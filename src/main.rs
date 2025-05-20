mod hoi;
mod user_command;

use std::fs;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::hoi::{Hoi, HoiError};
use rand::seq::SliceRandom;
use rand::thread_rng;
use tabled::builder::Builder;
use tabled::settings::object::Columns;
use tabled::settings::{Alignment, Modify, Padding, Style};

/// Searches for a .hoi.yml configuration file in the current directory and its parents.
/// Returns the path to the first .hoi.yml file found, or None if no configuration file exists.
fn find_config_file() -> Option<PathBuf> {
    use std::env;

    let current_dir = env::current_dir().ok()?;
    let mut dir = current_dir.as_path();

    loop {
        let config_path = dir.join(".hoi.yml");
        if config_path.exists() {
            // On Windows, avoid canonicalize() as it can lead to path format issues
            #[cfg(not(windows))]
            {
                return config_path.canonicalize().ok().or(Some(config_path));
            }

            // For Windows, just return the path directly
            #[cfg(windows)]
            {
                return Some(config_path);
            }
        }

        if let Some(parent) = dir.parent() {
            dir = parent;
        } else {
            break;
        }
    }

    None
}

/// Checks for a global .hoi.global.yml configuration file in the user's home directory.
/// Returns the path to the global config file if it exists, or None if it doesn't.
fn find_global_config_file() -> Option<PathBuf> {
    if let Some(home_dir) = dirs_next::home_dir() {
        // Create the global config path
        let global_config = home_dir.join(".hoi").join(".hoi.global.yml");

        if global_config.exists() {
            return match global_config.canonicalize() {
                Ok(path) => Some(path),
                Err(_) => Some(global_config),
            };
        }
    }

    None
}

/// Loads environment variables from .env and .env.local files in the same directory as the .hoi.yml file.
/// If both files exist, .env is loaded first, and .env.local variables will override any variables
/// with the same name defined in .env.
///
/// # Arguments
/// * `config_dir` - The directory containing the .hoi.yml file
fn load_environment_files(config_dir: &Path) {
    let env_file = config_dir.join(".env");
    if env_file.exists() {
        let _ = dotenvy::from_path(&env_file);
    }

    let env_local_file = config_dir.join(".env.local");
    if env_local_file.exists() {
        let _ = dotenvy::from_path_override(&env_local_file);
    }
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
/// about Hoi. If the random selection fails for any reason, it falls
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
        "Hoi also looks for a global config at ~/.hoi/.hoi.global.yml.",
        "Global commands are available in all projects and mixed with local commands.",
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

/// Looks up a command by its alias.                                                                                                                                                                                                                                                                                                                                    
///                                                                                                                                                                                                                                                                                                                                                                     
/// This function searches through all commands in the Hoi configuration                                                                                                                                                                                                                                                                                                
/// and returns the name of the command that has the specified alias.                                                                                                                                                                                                                                                                                                   
///                                                                                                                                                                                                                                                                                                                                                                     
/// # Arguments                                                                                                                                                                                                                                                                                                                                                         
/// * `hoi` - The Hoi configuration struct containing command definitions                                                                                                                                                                                                                                                                                               
/// * `alias` - The alias to search for                                                                                                                                                                                                                                                                                                                                 
///                                                                                                                                                                                                                                                                                                                                                                     
/// # Returns                                                                                                                                                                                                                                                                                                                                                          
/// * `Option<&String>` - The name of the command with the matching alias, or None if no match found                                                                                                                                                                                                                                                                    
fn find_command_by_alias(hoi: &Hoi, alias: &str) -> Option<String> {
    for (name, command) in &hoi.commands {
        if let Some(a) = &command.alias {
            if a == alias {
                return Some(name.clone());
            }
        }
    }
    None
}

/// Displays the available commands in a nicely formatted table.
///
/// This function generates and displays a table of all available commands
/// defined in the Hoi configuration.
/// It also shows a greeting and a random "Did you know?" fact.
///
/// # Arguments
/// * `hoi` - The Hoi configuration struct containing commands to display
fn display_commands(hoi: &Hoi) {
    let mut builder = Builder::default();

    builder.push_record(["Command", "Alias", "Description"]);
    builder.push_record([
        "init",
        "",
        "Create a new .hoi.yml configuration file in the current directory.",
    ]);

    for (name, command) in &hoi.commands {
        builder.push_record([
            name,
            command.alias.as_deref().unwrap_or(""),
            &command.description,
        ]);
    }

    let mut table = builder.build();

    table
        .with(Style::blank())
        .with(Padding::new(1, 1, 0, 0))
        .with(Modify::new(Columns::new(..)).with(Alignment::left()));

    println!("Hoi Hoi!");
    println!("\nDid you know? {}", get_random_did_you_know());
    println!("\nUsage:");
    println!("  hoi [command|alias] (command options) (command arguments...)");

    if !hoi.description.is_empty() {
        println!("\n{}\n", hoi.description);
    } else {
        println!();
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
    let alias_or_command = match find_command_by_alias(hoi, command_name) {
        Some(alias) => alias.to_string(),
        None => command_name.parse().unwrap(),
    };

    match hoi.commands.get(&alias_or_command) {
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

/// Creates a new .hoi.yml file with a basic template in the current directory.
///
/// This function creates a new configuration file with some example commands
/// to help users get started. It will not overwrite an existing file.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Ok if file creation succeeded, or an error
///
/// # Errors
/// * Various I/O errors if file creation fails
fn create_init_config() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::fs::File;

    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".hoi.yml");

    if config_path.exists() {
        println!(
            "A .hoi.yml file already exists at {}",
            config_path.display()
        );
        return Ok(());
    }

    let mut file = File::create(&config_path)?;

    let template = r#"version: 1
description: "Custom commands"
commands:
  hello:
    cmd: echo "Hello from Hoi!"
    description: "A simple example command."
  multiline:
    cmd: |
      echo "This is a multi-line command"
      echo "Each line will be executed in sequence"
    alias: ml
    description: "An example of a multi-line command with an alias."
  help:
    cmd: |
      echo "To add new commands, edit the .hoi.yml file in this directory."
      echo "Run 'hoi' to see a list of all available commands."
    description: "Shows help information about using hoi."
"#;

    file.write_all(template.as_bytes())?;
    println!("Created new .hoi.yml file at {}", config_path.display());
    println!("Run 'hoi' to see your available commands.");

    Ok(())
}

/// The main entry point for the Hoi application.
///
/// This function coordinates the overall flow of the application:
/// 1. Finds and loads the Hoi configuration files (local and global)
/// 2. Loads environment variables from .env and .env.local files if they exist
///    (with .env.local values overriding .env values)
/// 3. Merges configurations, with local commands taking precedence
/// 4. Parses command-line arguments
/// 5. Either displays available commands or executes the specified command
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Ok if execution was successful or an error
///
/// # Errors
/// * Various errors can be returned if configuration loading or command execution fails
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;

    // Parse command line arguments early to check for 'init' command
    let mut args: Vec<String> = env::args().skip(1).collect();

    // Handle the 'init' command before looking for config files
    if !args.is_empty() && args[0] == "init" {
        return create_init_config();
    }

    // Find and load the local config file (project-specific)
    let local_config_path = find_config_file();

    // Find and load the global config file
    let global_config_path = find_global_config_file();

    // Start with empty default configuration
    let mut merged_hoi = Hoi::default();

    // Load and merge global config if it exists
    if let Some(global_path) = global_config_path {
        if let Ok(global_hoi) = load_config(&global_path) {
            if !global_hoi.entrypoint.is_empty() {
                merged_hoi.entrypoint = global_hoi.entrypoint;
            }

            if !global_hoi.description.is_empty() && merged_hoi.description.is_empty() {
                merged_hoi.description = global_hoi.description;
            }

            for (name, command) in global_hoi.commands {
                merged_hoi.commands.insert(name, command);
            }
        }
    }

    // Load and merge local config if it exists (overriding global settings)
    if let Some(local_path) = local_config_path {
        if let Some(config_dir) = local_path.parent() {
            load_environment_files(config_dir);
        }

        if let Ok(local_hoi) = load_config(&local_path) {
            // Override entrypoint if defined in local config
            if !local_hoi.entrypoint.is_empty() {
                merged_hoi.entrypoint = local_hoi.entrypoint;
            }

            // Override description if defined in local config
            if !local_hoi.description.is_empty() {
                merged_hoi.description = local_hoi.description;
            }

            // Add local commands (overriding any global commands with the same name)
            for (name, command) in local_hoi.commands {
                merged_hoi.commands.insert(name, command);
            }
        }
    }

    // Args were already parsed earlier to check for 'init' command
    if args.is_empty() {
        display_commands(&merged_hoi);
    } else {
        let command_name = args.remove(0);
        execute_command(&merged_hoi, &command_name, &args)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::{self};
    use std::path::PathBuf;
    use temp_env::with_var;
    use testdir::testdir;
    use utilities::copy_fixture;

    #[test]
    fn test_custom_entrypoint() {
        let temp_dir: PathBuf = testdir!();
        copy_fixture(".hoi.with_entrypoint.yml", &temp_dir, ".hoi.yml");
        let result = load_config(&temp_dir.join(".hoi.yml"));

        assert!(
            result.is_ok(),
            "Failed to load valid config: {:?}",
            result.err()
        );

        let hoi = result.unwrap();
        assert_eq!(hoi.entrypoint, vec!["sh", "-c", "$@"]);
    }

    #[test]
    fn test_find_config() {
        let temp_dir: PathBuf = testdir!();
        copy_fixture(".hoi.yml", &temp_dir, ".hoi.yml");
        let config_path = temp_dir.join(".hoi.yml");
        env::set_current_dir(&temp_dir).unwrap();

        let result = find_config_file();
        assert!(result.is_some(), "Failed to find config file");

        #[cfg(not(windows))]
        {
            let canonical_path = config_path.canonicalize().ok().unwrap();
            assert_eq!(result.unwrap(), canonical_path);
        }

        #[cfg(windows)]
        {
            let result_path = result.unwrap();
            // For Windows, just check that the file exists and has the right name
            assert!(result_path.exists(), "Result path does not exist");
            assert_eq!(
                result_path.file_name().unwrap(),
                config_path.file_name().unwrap()
            );
        }
    }

    #[test]
    fn test_find_global_config() {
        let temp_dir: PathBuf = testdir!();

        #[cfg(not(windows))]
        let (env_var, env_val) = ("HOME", &temp_dir);
        #[cfg(windows)]
        let (env_var, env_val) = ("USERPROFILE", &temp_dir);

        with_var(env_var, Some(env_val.to_str().unwrap()), || {
            let home_dir = dirs_next::home_dir().unwrap();
            let hoi_dir = home_dir.join(".hoi");
            fs::create_dir_all(&hoi_dir).unwrap();
            copy_fixture(".hoi.global.yml", &hoi_dir, ".hoi.global.yml");

            let result = find_global_config_file();
            assert!(result.is_some(), "Failed to find global config file");

            let result_path = result.unwrap();

            // Platform-specific path comparison
            #[cfg(not(windows))]
            {
                let canonical_path = hoi_dir.join(".hoi.global.yml").canonicalize().unwrap();
                assert_eq!(result_path, canonical_path);
            }

            #[cfg(windows)]
            {
                // For Windows, just check that the file exists and has the right name
                assert!(result_path.exists(), "Result path does not exist");
                assert_eq!(
                    result_path.file_name().unwrap(),
                    temp_dir.file_name().unwrap()
                );

                // Also verify the parent directory is correct
                assert_eq!(
                    result_path.parent().unwrap().file_name().unwrap(),
                    temp_dir.unwrap().parent().unwrap().file_name().unwrap()
                );
            }
        });
    }

    #[test]
    fn test_init_command() {
        let temp_dir: PathBuf = testdir!();
        env::set_current_dir(&temp_dir).unwrap();

        // Run the init config function
        let result = create_init_config();
        assert!(
            result.is_ok(),
            "Failed to create init config: {:?}",
            result.err()
        );

        // Verify the config file was created
        let config_path = temp_dir.join(".hoi.yml").canonicalize().ok().unwrap();
        assert!(config_path.exists(), "Config file was not created");

        // Test that running init again when file exists doesn't overwrite it
        let original_content = fs::read_to_string(&config_path).unwrap();

        // Modify the file slightly to detect if it gets overwritten
        let modified_content = original_content.replace("Custom commands", "Test commands");
        fs::write(&config_path, modified_content).unwrap();

        // Run init again
        let result = create_init_config();
        assert!(
            result.is_ok(),
            "Failed on second init run: {:?}",
            result.err()
        );

        // Verify content wasn't overwritten
        let final_content = fs::read_to_string(&config_path).unwrap();
        assert!(
            final_content.contains("Test commands"),
            "Config file was incorrectly overwritten"
        );
    }
}
