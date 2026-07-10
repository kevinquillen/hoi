use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use testdir::testdir;
use utilities::copy_fixture;

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_hoi"))
}

fn run_hoi(args: &[&str], cwd: &Path, home: &Path) -> Output {
    let mut command = Command::new(binary());
    command.args(args).current_dir(cwd);
    #[cfg(not(windows))]
    command.env("HOME", home);
    #[cfg(windows)]
    command.env("USERPROFILE", home);
    command.output().expect("failed to execute hoi")
}

fn output_text(output: &Output) -> (String, String) {
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn lists_and_executes_local_and_global_commands() {
    let root: PathBuf = testdir!();
    let home = root.join("home");
    fs::create_dir_all(home.join(".hoi")).unwrap();
    copy_fixture(".hoi.yml", &root, ".hoi.yml");
    copy_fixture(".hoi.global.yml", &home.join(".hoi"), ".hoi.global.yml");

    let output = run_hoi(&["list"], &root, &home);
    assert!(output.status.success());
    let (stdout, _) = output_text(&output);
    assert!(stdout.contains("Integration test config"));
    assert!(stdout.contains("echo-test"));
    assert!(stdout.contains("global-echo"));

    let output = run_hoi(&["ge"], &root, &home);
    assert!(output.status.success());
    assert!(output_text(&output).0.contains("Global command successful"));
}

#[test]
fn loads_environment_files() {
    let root: PathBuf = testdir!();
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    copy_fixture(".hoi.yml", &root, ".hoi.yml");
    copy_fixture(".env", &root, ".env");
    copy_fixture(".env.local", &root, ".env.local");

    let output = run_hoi(&["echo-env"], &root, &home);
    assert!(output.status.success());
    let stdout = output_text(&output).0;
    assert!(stdout.contains("ENV_VAR=env_value"));
    assert!(stdout.contains("LOCAL_VAR=local_value"));
    assert!(stdout.contains("OVERRIDE_VAR=local_value"));
}

#[test]
fn propagates_child_exit_code() {
    let root: PathBuf = testdir!();
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    #[cfg(not(windows))]
    let command = "exit 7";
    #[cfg(windows)]
    let command = "exit /B 7";
    fs::write(
        root.join(".hoi.yml"),
        format!("version: 1\ncommands:\n  fail:\n    cmd: {command}\n"),
    )
    .unwrap();

    let output = run_hoi(&["fail"], &root, &home);
    assert_eq!(output.status.code(), Some(7));
}

#[cfg(not(windows))]
#[test]
fn forwards_each_command_argument_once() {
    let root: PathBuf = testdir!();
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    fs::write(
        root.join(".hoi.yml"),
        "version: 1\ncommands:\n  args:\n    cmd: 'printf ''%s|%s'' \"$1\" \"$2\"'\n",
    )
    .unwrap();

    let output = run_hoi(&["args", "alpha", "beta"], &root, &home);
    assert!(output.status.success());
    assert!(output_text(&output).0.ends_with("alpha|beta"));
}

#[test]
fn reports_malformed_config_with_its_path() {
    let root: PathBuf = testdir!();
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    fs::write(root.join(".hoi.yml"), "commands: [not valid").unwrap();

    let output = run_hoi(&["list"], &root, &home);
    assert!(!output.status.success());
    let stderr = output_text(&output).1;
    assert!(stderr.contains("Invalid YAML"));
    assert!(stderr.contains(".hoi.yml"));
}

#[test]
fn unknown_command_is_a_failure() {
    let root: PathBuf = testdir!();
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    copy_fixture(".hoi.yml", &root, ".hoi.yml");

    let output = run_hoi(&["does-not-exist"], &root, &home);
    assert!(!output.status.success());
    assert!(output_text(&output).1.contains("Command not found"));
}

#[test]
fn missing_config_is_successful_and_suggests_init() {
    let root: PathBuf = testdir!();
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();

    let output = run_hoi(&[], &root, &home);
    assert!(output.status.success());
    let stdout = output_text(&output).0;
    assert!(stdout.contains("No .hoi.yml file found"));
    assert!(stdout.contains("hoi init"));

    let command = run_hoi(&["missing"], &root, &home);
    assert!(!command.status.success());
    assert!(output_text(&command).1.contains("hoi init"));

    let check = run_hoi(&["config", "--check"], &root, &home);
    assert!(!check.status.success());
}

#[test]
fn supports_help_version_and_config_inspection() {
    let root: PathBuf = testdir!();
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    copy_fixture(".hoi.yml", &root, ".hoi.yml");

    let help = run_hoi(&["--help"], &root, &home);
    assert!(help.status.success());
    assert!(output_text(&help).0.contains("hoi config --check"));

    let version = run_hoi(&["--version"], &root, &home);
    assert!(version.status.success());
    assert!(output_text(&version).0.contains(env!("CARGO_PKG_VERSION")));

    let check = run_hoi(&["config", "--check"], &root, &home);
    assert!(check.status.success());
    let stdout = output_text(&check).0;
    assert!(stdout.contains("Configuration is valid"));
    assert!(stdout.contains(".hoi.yml"));
}

#[test]
fn executes_from_discovered_project_root() {
    let root: PathBuf = testdir!();
    let child = root.join("src").join("nested");
    let home = root.join("home");
    fs::create_dir_all(&child).unwrap();
    fs::create_dir_all(&home).unwrap();
    #[cfg(not(windows))]
    let command = "pwd";
    #[cfg(windows)]
    let command = "cd";
    fs::write(
        root.join(".hoi.yml"),
        format!("version: 1\ncommands:\n  cwd:\n    cmd: {command}\n"),
    )
    .unwrap();

    let output = run_hoi(&["cwd"], &child, &home);
    assert!(output.status.success());
    let canonical_root = root.canonicalize().unwrap();
    assert!(output_text(&output)
        .0
        .to_lowercase()
        .contains(&canonical_root.display().to_string().to_lowercase()));
}

#[test]
fn init_respects_force() {
    let root: PathBuf = testdir!();
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();

    assert!(run_hoi(&["init"], &root, &home).status.success());
    let path = root.join(".hoi.yml");
    fs::write(&path, "custom").unwrap();
    assert!(run_hoi(&["init"], &root, &home).status.success());
    assert_eq!(fs::read_to_string(&path).unwrap(), "custom");
    assert!(run_hoi(&["init", "--force"], &root, &home).status.success());
    assert!(fs::read_to_string(path).unwrap().contains("version: 1"));
}

#[test]
fn init_can_create_global_config() {
    let root: PathBuf = testdir!();
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();

    let output = run_hoi(&["init", "--global"], &root, &home);
    assert!(output.status.success());
    assert!(home.join(".hoi").join(".hoi.global.yml").is_file());
}

#[test]
fn local_commands_override_global_commands() {
    let root: PathBuf = testdir!();
    let home = root.join("home");
    fs::create_dir_all(home.join(".hoi")).unwrap();
    fs::write(
        home.join(".hoi").join(".hoi.global.yml"),
        "version: 1\ncommands:\n  shared:\n    cmd: echo global\n",
    )
    .unwrap();
    fs::write(
        root.join(".hoi.yml"),
        "version: 1\ncommands:\n  shared:\n    cmd: echo local\n",
    )
    .unwrap();

    let output = run_hoi(&["shared"], &root, &home);
    assert!(output.status.success());
    let stdout = output_text(&output).0;
    assert!(stdout.contains("local"));
    assert!(!stdout.contains("global"));
}
