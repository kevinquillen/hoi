use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserCommand {
    pub(crate) cmd: String,
    #[serde(default)]
    pub(crate) alias: String,
    #[serde(default)]
    pub(crate) description: String,
}