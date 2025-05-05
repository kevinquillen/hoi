use crate::user_command::UserCommand;
use indexmap::IndexMap;
use serde::Deserialize;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HoiError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("YAML parsing error: {0}")]
    YamlParsing(#[from] serde_yaml::Error),
    #[error("Command not found: {0}")]
    CommandNotFound(String),
    #[error("No .hoi.yml file found in current directory or parent directories, and no global config was found at ~/.hoi/.hoi.global.yml.")]
    ConfigNotFound,
    #[error("No commands defined in .hoi.yml file. You need at least one command defined.")]
    NoCommandsDefined,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Hoi {
    #[serde(default = "default_version")]
    pub(crate) version: String,

    #[serde(default = "default_description")]
    pub(crate) description: String,

    #[serde(default = "default_entrypoint")]
    pub(crate) entrypoint: Vec<String>,

    #[serde(default)]
    pub(crate) commands: IndexMap<String, UserCommand>,
}

impl Default for crate::hoi::Hoi {
    fn default() -> Self {
        Self {
            version: String::new(),
            description: String::new(),
            entrypoint: Vec::new(),
            commands: IndexMap::new(),
        }
    }
}

/// Returns the default description string for Hoi configuration.
/// This is used when no description is specified in the configuration file.
fn default_description() -> String {
    "Hoi is designed to help teams standardize their development workflows.".to_string()
}

/// Returns the default version string for Hoi configuration.
/// This is used when no version is specified in the configuration file.
fn default_version() -> String {
    "1".to_string()
}

/// Returns the default entrypoint to use.
/// This is used when no entrypoint is specified in the configuration file.
fn default_entrypoint() -> Vec<String> {
    vec![
        "bash".to_string(),
        "-e".to_string(),
        "-c".to_string(),
        "$@".to_string(),
    ]
}
