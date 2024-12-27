use anyhow::{bail, Context, Result};
use lazy_regex::regex_captures;
use log::info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BatchDesc {
    pub id: String,
    pub size: u64,
}

fn write_file(dir: &Path, id: usize, ext: &'static str, text: &str) -> Result<()> {
    let name = dir.join(id.to_string()).with_extension(ext);
    fs::write(&name, text).with_context(|| format!("error write {name:?}"))
}

/// Dump parse result to competition directory.
/// # Errors
/// Throw error breaks dumping process.
pub fn dump_to_cp_dir(result: &ParseResult, topdir: &str) -> Result<()> {
    let subdir = dstdir(result)?;
    let dir = Path::new(topdir).join(subdir);
    fs::create_dir_all(&dir).with_context(|| format!("mkdir -p {dir:?}"))?;
    for (test, id) in result.tests.iter().zip(1..) {
        write_file(&dir, id, "in", &test.input)?;
        write_file(&dir, id, "out", &test.output)?;
    }
    let metapath = dir.join("meta.json");
    let meta = fs::File::create(&metapath).with_context(|| format!("error create {metapath:?}"))?;
    serde_json::to_writer(meta, &result)?;
    info!("dump [{}]({}) done.", result.name, result.url);
    Ok(())
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
}
