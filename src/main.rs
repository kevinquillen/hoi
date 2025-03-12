#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn create_test_config(dir: &Path) -> PathBuf {
        let config_path = dir.join(".hoi.yml");
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "version: 1").unwrap();
        writeln!(file, "description: \"Test configuration\"").unwrap();
        writeln!(file, "entrypoint:").unwrap();
        writeln!(file, "  - bash").unwrap();
        writeln!(file, "  - -e").unwrap();
        writeln!(file, "  - -c").unwrap();
        writeln!(file, "  - \"$@\"").unwrap();
        writeln!(file, "commands:").unwrap();
        writeln!(file, "  test-command:").unwrap();
        writeln!(file, "    cmd: echo \"Test output\"").unwrap();
        writeln!(file, "    usage: \"Test command that prints output\"").unwrap();
        writeln!(file, "    description: \"A simple test command\"").unwrap();
        writeln!(file, "  multiple-command:").unwrap();
        writeln!(file, "    cmd: |").unwrap();
        writeln!(file, "      echo \"Line 1\"").unwrap();
        writeln!(file, "      echo \"Line 2\"").unwrap();
        writeln!(file, "    usage: \"Multi-line command example\"").unwrap();
        writeln!(
            file,
            "    description: \"A multi-line command for testing\""
        )
        .unwrap();
        config_path
    }

    #[test]
    fn test_load_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = create_test_config(temp_dir.path());
        let result = load_config(&config_path);
        assert!(
            result.is_ok(),
            "Failed to load valid config: {:?}",
            result.err()
        );

        let hoi = result.unwrap();
        assert_eq!(hoi.version, "1");
        assert_eq!(hoi.description, "Test configuration");
        assert_eq!(hoi.entrypoint, vec!["bash", "-e", "-c", "$@"]);
        assert_eq!(hoi.commands.len(), 2);

        // Verify commands are in insertion order
        let command_keys: Vec<_> = hoi.commands.keys().collect();
        assert_eq!(command_keys[0], "test-command");
        assert_eq!(command_keys[1], "multiple-command");

        let test_cmd = hoi.commands.get("test-command").unwrap();
        assert_eq!(test_cmd.cmd, "echo \"Test output\"");
        assert_eq!(test_cmd.usage, "Test command that prints output");
        assert_eq!(test_cmd.description, "A simple test command");

        let multi_cmd = hoi.commands.get("multiple-command").unwrap();
        assert!(multi_cmd.cmd.contains("Line 1"));
        assert!(multi_cmd.cmd.contains("Line 2"));
        assert_eq!(multi_cmd.description, "A multi-line command for testing");
    }

    #[test]
    fn test_find_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = create_test_config(temp_dir.path());
        // Override current directory for testing
        env::set_current_dir(temp_dir.path()).unwrap();

        let result = find_config_file();
        assert!(result.is_some(), "Failed to find config file");
        assert_eq!(result.unwrap(), config_path);
    }
}

use std::fs;
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use thiserror::Error;

use indexmap::IndexMap;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Deserialize;
use tabled::builder::Builder;
use tabled::settings::object::Columns;
use tabled::settings::{Alignment, Color, Modify, Padding, Style};

#[derive(Error, Debug)]
enum HoiError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("YAML parsing error: {0}")]
    YamlParsing(#[from] serde_yaml::Error),
    #[error("Command not found: {0}")]
    CommandNotFound(String),
    #[error("No .hoi.yml file found in current directory or parent directories")]
    ConfigNotFound,
    #[error("No commands defined in .hoi.yml file")]
    NoCommandsDefined,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Hoi {
    #[serde(default = "default_version")]
    version: String,

    #[serde(default)]
    description: String,

    entrypoint: Vec<String>,

    #[serde(default)]
    commands: IndexMap<String, UserCommand>,
}

impl Default for Hoi {
    fn default() -> Self {
        Self {
            version: default_version(),
            description: String::new(),
            entrypoint: Vec::new(),
            commands: IndexMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct UserCommand {
    cmd: String,
    usage: String,
    #[serde(default)]
    description: String,
}

/// Returns the default version string for Hoi configuration.
/// This is used when no version is specified in the configuration file.
fn default_version() -> String {
    "1".to_string()
}

/// Searches for a .hoi.yml configuration file in the current directory and its parents.
/// If not found in the directory hierarchy, also checks the user's home directory.
/// Returns the path to the first .hoi.yml file found, or None if no configuration file exists.
fn find_config_file() -> Option<PathBuf> {
    use std::env;

    let current_dir = env::current_dir().ok()?;
    let mut dir = current_dir.as_path();

    loop {
        let config_path = dir.join(".hoi.yml");
        if config_path.exists() {
            return Some(config_path);
        }

        if let Some(parent) = dir.parent() {
            dir = parent;
        } else {
            break;
        }
    }

    // Also check in the user's home directory
    if let Some(home_dir) = dirs_next::home_dir() {
        let home_config = home_dir.join(".hoi.yml");
        if home_config.exists() {
            return Some(home_config);
        }
    }

    None
}

/// Loads and parses the Hoi configuration file from the specified path.
///
/// # Arguments
/// * `path` - The path to the .hoi.yml configuration file
///
/// # Returns
/// * `Result<Hoi, HoiError>` - The parsed Hoi configuration struct or an error
///
/// # Errors
/// * `HoiError::Io` - If the file cannot be read
/// * `HoiError::YamlParsing` - If the YAML is invalid
/// * `HoiError::NoCommandsDefined` - If the configuration doesn't define any commands
fn load_config(path: &Path) -> Result<Hoi, HoiError> {
    let file = fs::File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let hoi: Hoi = serde_yaml::from_str(&contents)?;

    if hoi.commands.is_empty() {
        return Err(HoiError::NoCommandsDefined);
    }

    Ok(hoi)
}

/// Returns a random "Did you know?" fact about Hoi.
///
/// This function selects a random interesting fact from a predefined list of facts
/// about Hoi and its usage. If the random selection fails for any reason, it falls
/// back to a default fact.
///
/// # Returns
/// * A static string containing an interesting fact about Hoi
fn get_random_did_you_know() -> &'static str {
    let did_you_know_facts = [
        "In Dutch, 'hoi' is an informal way to say 'hi'.",
        "Hoi configuration files use YAML format.",
        "You can add custom commands to Hoi by editing your .hoi.yml file.",
        "Hoi searches for .hoi.yml in your current directory and up through parent directories.",
        "Hoi also checks your home directory for a global .hoi.global.yml file.",
        "You can add detailed descriptions to your commands in the .hoi.yml file.",
        "You can create multi-line commands using the pipe operator (|) in YAML.",
        "Hoi is designed to help teams standardize their development workflows.",
        "In Hawaiian, 'hoi hoi' means to entertain, amuse, charm, delight, encourage, or please.",
        "In Japanese, 'hoi hoi' is a way of describing an action or task that is done quickly and without much thought.",
        "In Korean, 'hoi hoi' is used when you do something like magic."
    ];

    let mut rng = thread_rng();
    did_you_know_facts
        .choose(&mut rng)
        .unwrap_or(&"Hoi is a command-line tool.")
}

/// Displays the available commands in a nicely formatted table.
///
/// This function generates and displays a table of all available commands
/// defined in the Hoi configuration, including their usage and descriptions.
/// It also shows a greeting, a random "Did you know?" fact, and usage instructions.
///
/// # Arguments
/// * `hoi` - The Hoi configuration struct containing commands to display
fn display_commands(hoi: &Hoi) {
    let mut builder = Builder::default();

    // Add header row
    builder.push_record(["Command", "Usage", "Description"]);

    // Add rows for each command
    for (name, command) in &hoi.commands {
        builder.push_record([name, &command.usage, &command.description]);
    }

    let mut table = builder.build();

    // Style the table with colors and borders
    table
        .with(Style::modern())
        .with(Padding::new(1, 1, 0, 0))
        .with(Modify::new(Columns::first()).with(Color::FG_BRIGHT_CYAN))
        .with(Modify::new(Columns::new(..)).with(Alignment::left()));

    println!("Hoi Hoi!");
    println!("\nDid you know? {}", get_random_did_you_know());
    println!("\nUsage:");
    println!("  hoi [command]");

    if !hoi.description.is_empty() {
        println!("\n{}\n", hoi.description);
    }
    println!("{}\n", table);
}

/// Executes a command defined in the Hoi configuration.
///
/// This function looks up the requested command in the Hoi configuration and executes it
/// using the specified entrypoint and any additional arguments. It handles special
/// placeholder substitution ($@) in the entrypoint and reports command execution status.
///
/// # Arguments
/// * `hoi` - The Hoi configuration struct containing command definitions
/// * `command_name` - The name of the command to execute
/// * `args` - Additional arguments to pass to the command
///
/// # Returns
/// * `Result<(), HoiError>` - Ok if the command executed successfully, or an error
///
/// # Errors
/// * `HoiError::CommandNotFound` - If the specified command is not defined in the configuration
/// * `HoiError::Io` - If there's an IO error executing the command
fn execute_command(hoi: &Hoi, command_name: &str, args: &[String]) -> Result<(), HoiError> {
    match hoi.commands.get(command_name) {
        Some(command) => {
            println!("Running command {}...", command_name);

            // Start with entrypoint
            let mut process_args: Vec<String> =
                Vec::with_capacity(hoi.entrypoint.len() + args.len() + 1);

            // Special handling for $@ in the entrypoint (replace with command)
            let mut placeholder_found = false;
            for arg in &hoi.entrypoint {
                if arg == "$@" {
                    process_args.push(command.cmd.to_string());
                    placeholder_found = true;
                } else {
                    process_args.push(arg.clone());
                }
            }

            // If $@ was not found in the entrypoint, just append the command
            if !placeholder_found {
                process_args.push(command.cmd.to_string());
            }

            let entrypoint = process_args.remove(0);

            // Add remaining command-line arguments
            process_args.extend_from_slice(args);

            let status = Command::new(entrypoint)
                .args(&process_args)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()?;

            if !status.success() {
                eprintln!(
                    "Command '{}' failed with exit code: {:?}",
                    command_name,
                    status.code()
                );
            }

            Ok(())
        }
        None => Err(HoiError::CommandNotFound(command_name.to_string())),
    }
}

/// The main entry point for the Hoi application.
///
/// This function coordinates the overall flow of the application:
/// 1. Finds and loads the Hoi configuration file
/// 2. Parses command-line arguments
/// 3. Either displays available commands or executes the specified command
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Ok if execution was successful or an error
///
/// # Errors
/// * Various errors can be returned if configuration loading or command execution fails
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;

    let config_path = find_config_file().ok_or(HoiError::ConfigNotFound)?;
    let hoi = load_config(&config_path)?;

    let mut args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        display_commands(&hoi);
    } else {
        let command_name = args.remove(0);
        execute_command(&hoi, &command_name, &args)?;
    }

    Ok(())
}
