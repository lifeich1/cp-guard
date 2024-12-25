use serde::Deserialize;

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
