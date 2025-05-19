use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct UserCommand {
    pub(crate) cmd: String,

    #[serde(default, deserialize_with = "trimmed")]
    pub(crate) alias: Option<String>,

    #[serde(default)]
    pub(crate) description: String,
}

fn trimmed<'a, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'a>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
}
