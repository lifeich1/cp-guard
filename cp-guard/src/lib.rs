use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Testcase {
    pub input: String,
    pub output: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputDesc {
    #[serde(rename = "type")]
    pub kind: String,
    pub file_name: Option<String>,
    pub pattern: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputDesc {
    #[serde(rename = "type")]
    pub kind: String,
    pub file_name: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LangSettings {
    pub java: Option<JavaLangSetting>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JavaLangSetting {
    pub main_class: String,
    pub task_class: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BatchDesc {
    pub id: String,
    pub size: u64,
}

fn dstdir(r: &ParseResult) -> Result<String> {
    // TODO cf & atcoder
    bail!("mismatch dstdir of url: {}", r.url);
}

fn write_file(dir: &PathBuf, id: usize, ext: &'static str, text: &str) -> Result<()> {
    let name = dir.join(id.to_string()).with_extension(ext);
    fs::write(&name, text).with_context(|| format!("error write {name:?}"))
}

/// Dump parse result to competition directory.
/// # Errors
/// Throw error breaks dumping process.
pub fn dump_to_cp_dir(result: ParseResult, topdir: &str) -> Result<()> {
    let subdir = dstdir(&result)?;
    let dir = Path::new(topdir).join(subdir);
    fs::create_dir_all(&dir).with_context(|| format!("mkdir -p {dir:?}"))?;
    for (test, id) in result.tests.into_iter().zip(1..) {
        write_file(&dir, id, "in", &test.input)?;
        write_file(&dir, id, "out", &test.output)?;
    }
    let meta = dir.join("meta.json");
    // TODO dump meta json
    Ok(())
}
