use crate::user_command::UserCommand;
use indexmap::IndexMap;
use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
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
        source: serde_yaml_ng::Error,
    },
    #[error("Invalid configuration {path}: {message}")]
    ConfigValidation { path: PathBuf, message: String },
    #[error("Command not found: {name}{hint}")]
    CommandNotFound { name: String, hint: String },
    #[error("{0}")]
    Cli(String),
    #[error("No .hoi.yml file found in current directory or parent directories, and no global config at ~/.hoi/.hoi.global.yml\nRun `hoi init` to create one.")]
    ConfigNotFound,
    #[error("Unable to execute command: {0}")]
    CommandIo(#[from] io::Error),
}

#[derive(Deserialize, Debug)]
pub struct Hoi {
    #[serde(default = "default_version", deserialize_with = "deserialize_version")]
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
        if !other.entrypoint.is_empty() {
            self.entrypoint = other.entrypoint;
        }
        if !other.description.is_empty() {
            self.description = other.description;
        }
        self.commands.extend(other.commands);
    }

    pub(crate) fn command_by_name_or_alias(&self, name: &str) -> Option<&UserCommand> {
        self.commands.get(name).or_else(|| {
            self.commands
                .values()
                .find(|command| command.alias.as_deref() == Some(name))
        })
    }

    pub(crate) fn unknown_command(&self, name: &str) -> HoiError {
        HoiError::CommandNotFound {
            name: name.to_string(),
            hint: self.did_you_mean(name),
        }
    }

    fn did_you_mean(&self, name: &str) -> String {
        let mut scored: Vec<(usize, &str)> = self
            .commands
            .iter()
            .flat_map(|(command_name, command)| {
                std::iter::once(command_name.as_str()).chain(command.alias.as_deref())
            })
            .filter_map(|candidate| {
                let distance = edit_distance(name, candidate);
                let prefix = candidate.starts_with(name) || name.starts_with(candidate);
                if distance <= 3 || prefix {
                    Some((distance, candidate))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by_key(|(distance, candidate)| (*distance, *candidate));
        scored.dedup_by_key(|(_, candidate)| *candidate);

        let suggestions: Vec<&str> = scored
            .into_iter()
            .take(3)
            .map(|(_, candidate)| candidate)
            .collect();

        if suggestions.is_empty() {
            String::new()
        } else {
            format!("\nDid you mean: {}?", suggestions.join(", "))
        }
    }
}

fn is_reserved(value: &str) -> bool {
    value.starts_with('-')
        || matches!(
            value,
            "init" | "list" | "config" | "help" | "version" | "validate"
        )
}

fn default_description() -> String {
    "Hoi is designed to help teams standardize their development workflows.".to_string()
}

fn default_version() -> String {
    "1".to_string()
}

fn deserialize_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct VersionVisitor;

    impl Visitor<'_> for VersionVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a version string or integer")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.trim().to_string())
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.trim().to_string())
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }
    }

    deserializer.deserialize_any(VersionVisitor)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0; b.len() + 1];

    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b.len()]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_entrypoint_is_not_empty() {
        let hoi = Hoi::default();
        assert!(!hoi.entrypoint.is_empty());
        assert_eq!(hoi.version, "1");
    }

    #[test]
    fn command_by_name_or_alias_prefers_name() {
        let yaml = r#"
version: 1
commands:
  build:
    cmd: echo build
    alias: b
    description: build
  other:
    cmd: echo other
    description: other
"#;
        let hoi: Hoi = serde_yaml_ng::from_str(yaml).unwrap();
        hoi.validate(PathBuf::from("test.yml")).unwrap();
        assert_eq!(
            hoi.command_by_name_or_alias("build").unwrap().cmd,
            "echo build"
        );
        assert_eq!(hoi.command_by_name_or_alias("b").unwrap().cmd, "echo build");
        assert!(hoi.command_by_name_or_alias("missing").is_none());
    }

    #[test]
    fn did_you_mean_suggests_close_names() {
        let yaml = r#"
version: 1
commands:
  fail-code:
    cmd: exit 42
    description: fail
"#;
        let hoi: Hoi = serde_yaml_ng::from_str(yaml).unwrap();
        match hoi.unknown_command("fial-code") {
            HoiError::CommandNotFound { name, hint } => {
                assert_eq!(name, "fial-code");
                assert!(hint.contains("fail-code"), "hint was {hint}");
            }
            other => panic!("expected CommandNotFound, got {other:?}"),
        }
    }

    #[test]
    fn rejects_duplicate_aliases() {
        let yaml = r#"
version: 1
commands:
  one:
    cmd: echo one
    alias: x
    description: one
  two:
    cmd: echo two
    alias: x
    description: two
"#;
        let hoi: Hoi = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(matches!(
            hoi.validate(PathBuf::from("test.yml")),
            Err(HoiError::ConfigValidation { .. })
        ));
    }

    #[test]
    fn merge_local_overrides_global_commands() {
        let mut global = Hoi::default();
        global.commands.insert(
            "hello".into(),
            UserCommand {
                cmd: "echo global".into(),
                alias: None,
                description: "global".into(),
            },
        );

        let mut local = Hoi {
            description: "local desc".into(),
            ..Hoi::default()
        };
        local.commands.insert(
            "hello".into(),
            UserCommand {
                cmd: "echo local".into(),
                alias: Some("h".into()),
                description: "local".into(),
            },
        );

        global.merge(local);
        assert_eq!(global.description, "local desc");
        assert_eq!(global.commands.get("hello").unwrap().cmd, "echo local");
        assert_eq!(
            global.commands.get("hello").unwrap().alias.as_deref(),
            Some("h")
        );
    }
}
