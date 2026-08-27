mod cli;
mod hoi;
mod user_command;

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use crate::cli::{Cli, CommandArg};
use crate::hoi::{Hoi, HoiError};
use clap::Parser;
use rand::seq::SliceRandom;
use rand::thread_rng;
use tabled::builder::Builder;
use tabled::settings::object::Columns;
use tabled::settings::{Alignment, Modify, Padding, Style};

#[derive(Debug)]
struct ConfigPaths {
    local: Option<PathBuf>,
    global: Option<PathBuf>,
}

fn canonical_or_original(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        env::var_os("USERPROFILE")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(dirs::home_dir)
    }
    #[cfg(not(windows))]
    {
        dirs::home_dir()
    }
}

fn find_config_file_from(start: &Path) -> Option<PathBuf> {
    for dir in start.ancestors() {
        let path = dir.join(".hoi.yml");
        if path.is_file() {
            return Some(canonical_or_original(path));
        }
    }
    None
}

fn find_global_config_file_from(home: Option<&Path>) -> Option<PathBuf> {
    let path = home?.join(".hoi").join(".hoi.global.yml");
    path.is_file().then(|| canonical_or_original(path))
}

fn discover_config_paths(start: &Path, home: Option<&Path>) -> ConfigPaths {
    ConfigPaths {
        local: find_config_file_from(start),
        global: find_global_config_file_from(home),
    }
}

fn load_environment_files(config_dir: &Path) {
    let env_file = config_dir.join(".env");
    if env_file.is_file() {
        if let Err(error) = dotenvy::from_path(&env_file) {
            eprintln!("Warning: failed to load {}: {error}", env_file.display());
        }
    }
    let env_local_file = config_dir.join(".env.local");
    if env_local_file.is_file() {
        if let Err(error) = dotenvy::from_path_override(&env_local_file) {
            eprintln!(
                "Warning: failed to load {}: {error}",
                env_local_file.display()
            );
        }
    }
}

fn load_config(path: &Path) -> Result<Hoi, HoiError> {
    let contents = fs::read_to_string(path).map_err(|source| HoiError::ConfigIo {
        path: path.to_path_buf(),
        source,
    })?;
    let hoi: Hoi = serde_yaml_ng::from_str(&contents).map_err(|source| HoiError::ConfigYaml {
        path: path.to_path_buf(),
        source,
    })?;
    hoi.validate(path.to_path_buf())?;
    Ok(hoi)
}

fn load_merged_config(paths: &ConfigPaths) -> Result<Hoi, HoiError> {
    let mut merged = Hoi::default();
    if let Some(path) = &paths.global {
        merged.merge(load_config(path)?);
    }
    if let Some(path) = &paths.local {
        merged.merge(load_config(path)?);
    }
    let source = paths
        .local
        .as_ref()
        .or(paths.global.as_ref())
        .expect("configuration paths checked before loading");
    merged.validate(source.clone())?;
    Ok(merged)
}

fn get_random_did_you_know() -> &'static str {
    let facts = [
        "In Dutch, 'hoi' is an informal way to say 'hi'.",
        "Hoi configuration files use YAML format.",
        "Hoi searches current and parent directories for .hoi.yml.",
        "Global commands can be defined in ~/.hoi/.hoi.global.yml.",
        "Local commands override global commands with the same name.",
        "Commands can have short aliases.",
        "Multi-line YAML commands run as one shell script.",
    ];
    facts
        .choose(&mut thread_rng())
        .copied()
        .unwrap_or("Hoi is a command-line tool.")
}

fn display_commands(hoi: &Hoi) {
    let mut builder = Builder::default();
    builder.push_record(["Command", "Alias", "Description"]);
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
    println!("\nUsage:\n  hoi [command|alias] [arguments...]");
    if !hoi.description.is_empty() {
        println!("\n{}", hoi.description);
    }
    println!("\n{table}\n");
}

fn execute_command(
    hoi: &Hoi,
    command_name: &str,
    args: &[String],
    working_dir: &Path,
) -> Result<ExitCode, HoiError> {
    let command = hoi
        .command_by_name_or_alias(command_name)
        .ok_or_else(|| hoi.unknown_command(command_name))?;
    println!("Running command {command_name}...");

    let mut process_args = Vec::with_capacity(hoi.entrypoint.len() + args.len() + 1);
    let mut placeholder_found = false;
    for arg in &hoi.entrypoint {
        if arg == "$@" {
            process_args.push(command.cmd.clone());
            placeholder_found = true;
        } else {
            process_args.push(arg.clone());
        }
    }
    if !placeholder_found {
        process_args.push(command.cmd.clone());
    }
    let entrypoint = process_args
        .first()
        .cloned()
        .ok_or_else(|| HoiError::Cli("command entrypoint is empty".to_string()))?;
    if !process_args.is_empty() {
        process_args.remove(0);
    }
    process_args.extend_from_slice(args);

    let status = Command::new(entrypoint)
        .args(process_args)
        .current_dir(working_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    Ok(match status.code() {
        Some(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        None => ExitCode::FAILURE,
    })
}

fn create_init_config(global: bool, force: bool) -> Result<(), HoiError> {
    let path = if global {
        let home = home_dir().ok_or_else(|| HoiError::ConfigValidation {
            path: PathBuf::from("~/.hoi/.hoi.global.yml"),
            message: "unable to determine the home directory".to_string(),
        })?;
        let dir = home.join(".hoi");
        fs::create_dir_all(&dir)?;
        dir.join(".hoi.global.yml")
    } else {
        env::current_dir()?.join(".hoi.yml")
    };

    if path.exists() && !force {
        println!("A configuration already exists at {}", path.display());
        println!("Use --force to replace it.");
        return Ok(());
    }
    let template = r#"version: 1
description: "Custom commands"
commands:
  hello:
    cmd: echo "Hello from Hoi!"
    alias: hi
    description: "A simple example command."
"#;
    let mut file = fs::File::create(&path)?;
    file.write_all(template.as_bytes())?;
    println!("Created configuration at {}", path.display());
    Ok(())
}

fn print_config_paths(paths: &ConfigPaths) {
    println!(
        "Local: {}",
        paths
            .local
            .as_deref()
            .map_or_else(|| "not found".to_string(), |p| p.display().to_string())
    );
    println!(
        "Global: {}",
        paths
            .global
            .as_deref()
            .map_or_else(|| "not found".to_string(), |p| p.display().to_string())
    );
}

fn require_config_flags(path: bool, check: bool) -> Result<(), HoiError> {
    match (path, check) {
        (true, false) | (false, true) => Ok(()),
        _ => Err(HoiError::Cli(
            "usage: hoi config <--path|--check>".to_string(),
        )),
    }
}

fn run() -> Result<ExitCode, HoiError> {
    let cli = Cli::parse();

    match &cli.command {
        Some(CommandArg::Init { global, force }) => {
            create_init_config(*global, *force)?;
            return Ok(ExitCode::SUCCESS);
        }
        Some(CommandArg::Config { path, check }) => {
            require_config_flags(*path, *check)?;
        }
        _ => {}
    }

    let current_dir = env::current_dir()?;
    let home = home_dir();
    let paths = discover_config_paths(&current_dir, home.as_deref());

    if matches!(
        cli.command,
        Some(CommandArg::Config {
            path: true,
            check: false
        })
    ) {
        print_config_paths(&paths);
        return Ok(ExitCode::SUCCESS);
    }

    let listing = matches!(cli.command, None | Some(CommandArg::List));
    if paths.local.is_none() && paths.global.is_none() {
        return if listing {
            println!("{}", HoiError::ConfigNotFound);
            Ok(ExitCode::SUCCESS)
        } else {
            Err(HoiError::ConfigNotFound)
        };
    }

    let hoi = load_merged_config(&paths)?;
    if let Some(dir) = paths.local.as_deref().and_then(Path::parent) {
        load_environment_files(dir);
    }

    match cli.command {
        Some(CommandArg::Config { check: true, .. }) | Some(CommandArg::Validate) => {
            print_config_paths(&paths);
            println!("Configuration is valid ({} commands).", hoi.commands.len());
            Ok(ExitCode::SUCCESS)
        }
        None | Some(CommandArg::List) => {
            display_commands(&hoi);
            Ok(ExitCode::SUCCESS)
        }
        Some(CommandArg::External(argv)) => {
            let (command, args) = argv
                .split_first()
                .ok_or_else(|| hoi.unknown_command("<missing>"))?;
            let working_dir = paths
                .local
                .as_deref()
                .and_then(Path::parent)
                .unwrap_or(&current_dir);
            execute_command(&hoi, command, args, working_dir)
        }
        Some(CommandArg::Init { .. }) | Some(CommandArg::Config { .. }) => unreachable!(),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testdir::testdir;
    use utilities::copy_fixture;

    #[test]
    fn finds_config_from_child_directory_without_changing_cwd() {
        let root: PathBuf = testdir!();
        let child = root.join("src").join("nested");
        fs::create_dir_all(&child).unwrap();
        copy_fixture(".hoi.yml", &root, ".hoi.yml");

        let found = find_config_file_from(&child).unwrap();
        assert_eq!(
            found.canonicalize().unwrap(),
            root.join(".hoi.yml").canonicalize().unwrap()
        );
    }

    #[test]
    fn rejects_duplicate_aliases() {
        let root: PathBuf = testdir!();
        let path = root.join(".hoi.yml");
        fs::write(&path, "version: 1\ncommands:\n  one:\n    cmd: echo one\n    alias: x\n  two:\n    cmd: echo two\n    alias: x\n").unwrap();
        assert!(matches!(
            load_config(&path),
            Err(HoiError::ConfigValidation { .. })
        ));
    }

    #[test]
    fn test_custom_entrypoint() {
        let temp_dir: PathBuf = testdir!();
        copy_fixture(".hoi.with_entrypoint.yml", &temp_dir, ".hoi.yml");
        let hoi = load_config(&temp_dir.join(".hoi.yml")).unwrap();
        assert_eq!(hoi.entrypoint, vec!["sh", "-c", "$@"]);
    }
}
