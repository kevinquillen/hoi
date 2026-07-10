use crate::user_command::UserCommand;
use indexmap::IndexMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HoiError {
    #[error("Unable to read configuration {path}: {source}")]
    ConfigIo { path: PathBuf, source: io::Error },
    #[error("Invalid YAML in configuration {path}: {source}")]
    ConfigYaml {
        path: PathBuf,
        source: serde_yaml::Error,
    },
    #[error("Invalid configuration {path}: {message}")]
    ConfigValidation { path: PathBuf, message: String },
    #[error("Command not found: {0}")]
    CommandNotFound(String),
    #[error("{0}")]
    Cli(String),
    #[error("No .hoi.yml file found in current directory or parent directories, and no global config at ~/.hoi/.hoi.global.yml\nRun `hoi init` to create one.")]
    ConfigNotFound,
    #[error("Unable to execute command: {0}")]
    CommandIo(#[from] io::Error),
}

#[derive(Deserialize, Debug)]
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

impl Default for Hoi {
    fn default() -> Self {
        Self {
            version: default_version(),
            description: String::new(),
            entrypoint: default_entrypoint(),
            commands: IndexMap::new(),
        }
    }
}

impl Hoi {
    pub(crate) fn validate(&self, path: PathBuf) -> Result<(), HoiError> {
        let invalid = |message: String| HoiError::ConfigValidation {
            path: path.clone(),
            message,
        };

        if self.version != "1" {
            return Err(invalid(format!(
                "unsupported version {:?}; supported version is \"1\"",
                self.version
            )));
        }
        if self.entrypoint.is_empty() || self.entrypoint.iter().all(|part| part.trim().is_empty()) {
            return Err(invalid("entrypoint must not be empty".to_string()));
        }
        if self.commands.is_empty() {
            return Err(invalid("at least one command must be defined".to_string()));
        }

        let mut aliases: HashMap<&str, &str> = HashMap::new();
        for (name, command) in &self.commands {
            if name.trim().is_empty() {
                return Err(invalid("command names must not be empty".to_string()));
            }
            if is_reserved(name) {
                return Err(invalid(format!("command name {name:?} is reserved")));
            }
            if command.cmd.trim().is_empty() {
                return Err(invalid(format!("command {name:?} has an empty cmd")));
            }
            if let Some(alias) = command.alias.as_deref() {
                if is_reserved(alias) {
                    return Err(invalid(format!("alias {alias:?} is reserved")));
                }
                if self.commands.contains_key(alias) {
                    return Err(invalid(format!(
                        "alias {alias:?} for command {name:?} conflicts with a command name"
                    )));
                }
                if let Some(existing) = aliases.insert(alias, name) {
                    return Err(invalid(format!(
                        "alias {alias:?} is used by both {existing:?} and {name:?}"
                    )));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn merge(&mut self, other: Hoi) {
        self.version = other.version;
        self.entrypoint = other.entrypoint;
        if !other.description.is_empty() {
            self.description = other.description;
        }
        self.commands.extend(other.commands);
    }
}

fn is_reserved(value: &str) -> bool {
    value.starts_with('-') || matches!(value, "init" | "list" | "config" | "help" | "version")
}

fn default_description() -> String {
    "Hoi is designed to help teams standardize their development workflows.".to_string()
}

fn default_version() -> String {
    "1".to_string()
}

fn default_entrypoint() -> Vec<String> {
    #[cfg(windows)]
    {
        vec!["cmd".to_string(), "/C".to_string()]
    }

    #[cfg(not(windows))]
    {
        vec![
            "bash".to_string(),
            "-e".to_string(),
            "-c".to_string(),
            "$@".to_string(),
            "hoi".to_string(),
        ]
    }
}
