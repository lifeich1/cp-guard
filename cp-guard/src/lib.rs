use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParseResult {
    name: String,
    group: String,
    url: String,
    interactive: Option<bool>,
    memory_limit: i64,
    time_limit: i64,
    tests: Vec<Testcase>,
    test_type: String,
    input: InputDesc,
    output: OutputDesc,
    languages: Option<LangSettings>,
    batch: BatchDesc,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Testcase {
    input: String,
    output: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputDesc {
    #[serde(rename = "type")]
    kind: String,
    file_name: Option<String>,
    pattern: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputDesc {
    #[serde(rename = "type")]
    kind: String,
    file_name: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LangSettings {
    java: Option<JavaLangSetting>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JavaLangSetting {
    main_class: String,
    task_class: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BatchDesc {
    id: String,
    size: u64,
}
