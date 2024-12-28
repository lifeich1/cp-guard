use anyhow::{bail, Context, Result};
use lazy_regex::regex_captures;
use log::{debug, error, info};
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{timeout_at, Instant};

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParseResult {
    pub name: String,
    pub group: String,
    pub url: String,
    pub interactive: Option<bool>,
    pub memory_limit: i64,
    pub time_limit: i64,
    pub tests: Vec<Testcase>,
    pub test_type: String,
    pub input: InputDesc,
    pub output: OutputDesc,
    pub languages: Option<LangSettings>,
    pub batch: BatchDesc,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Testcase {
    pub input: String,
    pub output: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct InputDesc {
    #[serde(rename = "type")]
    pub kind: String,
    pub file_name: Option<String>,
    pub pattern: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct OutputDesc {
    #[serde(rename = "type")]
    pub kind: String,
    pub file_name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct LangSettings {
    pub java: Option<JavaLangSetting>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JavaLangSetting {
    pub main_class: String,
    pub task_class: String,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchDesc {
    pub id: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct BatchDumpRes {
    batch: BatchDesc,
    group: String,
    code: Option<String>,
}

fn write_file(dir: &Path, id: usize, ext: &'static str, text: &str) -> Result<()> {
    let name = dir.join(id.to_string()).with_extension(ext);
    fs::write(&name, text).with_context(|| format!("error write {name:?}"))
}

/// Dump parse result to competition directory.
/// # Errors
/// Throw error breaks dumping process.
pub fn dump_to_cp_dir(
    result: &ParseResult,
    topdir: &str,
    tx: &mpsc::Sender<BatchDumpRes>,
) -> Result<()> {
    let dump = dump_to_cp_dir_impl(result, topdir);
    tx.try_send(BatchDumpRes {
        batch: result.batch.clone(),
        group: result.group.clone(),
        code: dump.as_ref().cloned().ok(),
    })
    .inspect_err(|e| {
        error!(
            "notify queue for [{}]({}) result {:?}: {:?}",
            result.name, result.url, &dump, e
        );
    })
    .ok();
    dump?;
    Ok(())
}

/// return subdir(code)
fn dump_to_cp_dir_impl(result: &ParseResult, topdir: &str) -> Result<String> {
    let subdir = dstdir(result)?;
    let dir = Path::new(topdir).join(&subdir);
    fs::create_dir_all(&dir).with_context(|| format!("mkdir -p {dir:?}"))?;
    for (test, id) in result.tests.iter().zip(1..) {
        write_file(&dir, id, "in", &test.input)?;
        write_file(&dir, id, "ans", &test.output)?;
    }
    let metapath = dir.join("meta.json");
    let meta = fs::File::create(&metapath).with_context(|| format!("error create {metapath:?}"))?;
    serde_json::to_writer(meta, &result)?;
    info!("dump [{}]({}) done.", result.name, result.url);
    Ok(subdir)
}

#[derive(Default, Debug)]
struct NotifyProxyCtx {
    batch: Option<BatchDumpRes>,
    ok_cnt: usize,
    err_cnt: usize,
    code_set: BTreeSet<String>,
}

pub async fn notify_proxy(mut rx: mpsc::Receiver<BatchDumpRes>) {
    let mut ctx = NotifyProxyCtx::default();
    let mut deadline = Instant::now();
    error!("notify_proxy enter loop ..");
    loop {
        let newbatch: Option<BatchDumpRes>;
        if ctx.batch.is_none() {
            newbatch = rx.recv().await;
            if newbatch.is_none() {
                error!("notify_proxy channel closed!");
                return;
            }
            deadline = Instant::now() + Duration::from_secs(5);
        } else {
            match timeout_at(deadline, rx.recv()).await {
                Err(_) | Ok(None) => newbatch = None,
                Ok(v) => newbatch = v,
            }
        }
        ctx.handle_new_batch(newbatch);
    }
}

impl NotifyProxyCtx {
    fn reset(&mut self) {
        self.ok_cnt = 0;
        self.err_cnt = 0;
        self.code_set.clear();
        self.batch = None;
    }

    fn notify(&self, hint: &'static str) {
        let mut text = String::with_capacity(256);
        let size: usize = self
            .batch
            .as_ref()
            .map_or(0, |b| b.batch.size)
            .try_into()
            .unwrap_or(0);
        text.push_str(&format!(
            "{}/{}/{} ok: ",
            self.ok_cnt,
            self.err_cnt,
            size.saturating_sub(self.ok_cnt + self.err_cnt)
        ));
        for code in &self.code_set {
            text.push_str(code);
            text.push(',');
        }

        let group = self
            .batch
            .as_ref()
            .map_or_else(|| "unknown".to_owned(), |b| b.group.clone());
        debug!("before send notify {self:?}");
        Notification::new()
            .appname(module_path!())
            .timeout(Duration::from_secs(5))
            .summary(&format!("{hint} - {group}"))
            .body(&text)
            .show()
            .inspect_err(|e| {
                if cfg!(test) && matches!(option_env!("XDG_CURRENT_DESKTOP"), Some("KDE")) {
                    panic!("error send notify {self:?}: {e:?}");
                } else {
                    error!("error send notify {self:?}: {e:?}");
                }
            })
            .ok();
        debug!("done send notify");
    }

    fn handle_new_batch(&mut self, newbatch: Option<BatchDumpRes>) {
        let Some(mut newbatch) = newbatch else {
            error!("batch deadline reach: {self:?}");
            self.notify("timeout");
            self.reset();
            return;
        };
        if let Some(last) = &self.batch {
            if last.batch.id != newbatch.batch.id {
                error!("unexpected diff batch id, last {self:?}, arrived {newbatch:?}");
                self.reset();
                return;
            }
        }
        if let Some(code) = newbatch.code.take() {
            self.code_set.insert(code);
            self.ok_cnt += 1;
        } else {
            self.err_cnt += 1;
        }
        let size = newbatch.batch.size.try_into().unwrap_or(1);
        self.batch = Some(newbatch);
        if self.ok_cnt + self.err_cnt == size {
            self.notify("done");
            self.reset();
        }
    }
}

fn dstdir(r: &ParseResult) -> Result<String> {
    let url = &r.url;
    if let Some((_, contest, prob)) =
        regex_captures!(r#"https://codeforces.com/contest/(\d+)/problem/(\w+)"#, url)
    {
        return Ok(format!("{contest}/{}", prob.to_lowercase()));
    }
    if let Some((_, contest, prob)) = regex_captures!(
        r#"https://codeforces.com/problemset/problem/(\d+)/(\w+)"#,
        url
    ) {
        return Ok(format!("{contest}/{}", prob.to_lowercase()));
    }
    if let Some((_, contest, prob)) =
        regex_captures!(r#"https://atcoder.jp/contests/\w+/tasks/(\w+)_(\w+)"#, url)
    {
        return Ok(format!("{contest}/{prob}"));
    }
    bail!("mismatch dstdir of url: {}", r.url);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dstdir_eq(url: &'static str, subdir: &'static str) {
        let res = dstdir(&ParseResult {
            url: url.to_owned(),
            ..Default::default()
        });
        println!("res: {res:?}");
        assert!(res.is_ok());
        assert_eq!(res.ok(), Some(subdir.to_owned()));
    }

    #[test]
    fn test_cf_url() {
        dstdir_eq("https://codeforces.com/contest/2051/problem/C", "2051/c");
        dstdir_eq("https://codeforces.com/problemset/problem/2041/N", "2041/n");
    }

    #[test]
    fn test_atcoder_url() {
        dstdir_eq(
            "https://atcoder.jp/contests/abc384/tasks/abc384_e",
            "abc384/e",
        );
    }

    #[test]
    fn test_handle_new_batch_one_prob() {
        let mut ctx = NotifyProxyCtx::default();
        ctx.handle_new_batch(Some(BatchDumpRes {
            batch: BatchDesc {
                id: "fake-batch-id".to_owned(),
                size: 1,
            },
            group: "test group".to_owned(),
            code: Some("test01/a".to_owned()),
        }));
    }

    #[test]
    fn test_handle_new_batch_two_prob() {
        let mut ctx = NotifyProxyCtx::default();
        let group = "test group 2".to_owned();
        let id = "fake-batch-id".to_owned();
        ctx.handle_new_batch(Some(BatchDumpRes {
            batch: BatchDesc {
                id: id.clone(),
                size: 2,
            },
            group: group.clone(),
            code: Some("test02/a".to_owned()),
        }));
        ctx.handle_new_batch(Some(BatchDumpRes {
            batch: BatchDesc { id, size: 2 },
            group,
            code: Some("test02/b".to_owned()),
        }));
    }

    #[test]
    fn test_handle_new_batch_two_prob_timeout() {
        let mut ctx = NotifyProxyCtx::default();
        let group = "test group 2".to_owned();
        let id = "fake-batch-id".to_owned();
        ctx.handle_new_batch(Some(BatchDumpRes {
            batch: BatchDesc { id, size: 2 },
            group,
            code: Some("test02/b".to_owned()),
        }));
        ctx.handle_new_batch(None);
    }
}
