use super::*;

#[derive(Deserialize)]
pub(crate) struct StandardJsonInput {
    pub(crate) language: String,
    pub(crate) sources: HashMap<String, StandardJsonSource>,
    #[serde(default)]
    pub(crate) settings: Value,
}

#[derive(Deserialize)]
pub(crate) struct StandardJsonSource {
    pub(crate) content: Option<String>,
    pub(crate) urls: Option<Vec<String>>,
}
