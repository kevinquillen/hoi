use std::env;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str;
use temp_env::with_var;
use testdir::testdir;

fn get_binary_path() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    Path::new(&manifest_dir)
        .join("target")
        .join(profile)
        .join("hoi")
}

#[test]
fn test_hoi_list_commands() {
    let temp_dir: PathBuf = testdir!();
    copy_fixture(".hoi.yml", &temp_dir, ".hoi.yml");

    Command::new("cargo")
        .args(["build"])
        .status()
        .expect("Failed to build hoi binary");

    let binary_path = get_binary_path();
    let output = Command::new(binary_path)
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed with status: {:?}",
        output.status
    );

    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("Hoi Hoi!"));
    assert!(stdout.contains("Integration test config"));
    assert!(stdout.contains("echo-test"));
    assert!(stdout.contains("Prints a test success message"));
}

#[test]
fn test_hoi_execute_command() {
    let temp_dir: PathBuf = testdir!();

    #[cfg(windows)]
    let (env_var, env_val) = ("USERPROFILE", &temp_dir);
    #[cfg(not(windows))]
    let (env_var, env_val) = ("HOME", &temp_dir);

    with_var(env_var, Some(env_val.to_str().unwrap()), || {
        // Set up config inside isolated "home"
        copy_fixture(".hoi.yml", &temp_dir, ".hoi.yml");

        let home_dir = dirs_next::home_dir().unwrap();
        let hoi_dir = home_dir.join(".hoi");
        fs::create_dir_all(&hoi_dir).unwrap();

        copy_fixture(".hoi.global.yml", &hoi_dir, ".hoi.global.yml");

        // Build binary
        Command::new("cargo")
            .args(["build"])
            .status()
            .expect("Failed to build hoi binary");

        let binary_path = get_binary_path();

        // Run local command
        let output = run_hoi_command(&binary_path, &["echo-test"], &temp_dir);
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Integration test successful"));

        // Run alias for global command
        let output = run_hoi_command(&binary_path, &["ge"], &temp_dir);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Global command successful"));

        // List commands (should include global)
        let output = run_hoi_command(&binary_path, &[], &temp_dir);
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("global-echo"));

        // Run global command explicitly
        let output = run_hoi_command(&binary_path, &["global-echo"], &temp_dir);
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Global command successful"));
    });
}

#[test]
fn test_hoi_with_env_files() {
    let temp_dir: PathBuf = testdir!();

    #[cfg(windows)]
    let (env_var, env_val) = ("USERPROFILE", &temp_dir);
    #[cfg(not(windows))]
    let (env_var, env_val) = ("HOME", &temp_dir);

    with_var(env_var, Some(env_val.to_str().unwrap()), || {
        copy_fixture(".hoi.yml", &temp_dir, ".hoi.yml");
        copy_fixture(".env", &temp_dir, ".env");

        // Build the binary
        Command::new("cargo")
            .args(["build"])
            .status()
            .expect("Failed to build hoi binary");

        let binary_path = get_binary_path();

        let output = Command::new(&binary_path)
            .arg("echo-env")
            .current_dir(&temp_dir)
            .env(env_var, env_val)
            .output()
            .expect("Failed to execute command with .env");

        let stdout = str::from_utf8(&output.stdout).unwrap();
        assert!(stdout.contains("ENV_VAR=env_value"));
        assert!(!stdout.contains("LOCAL_VAR=local_value"));
        assert!(stdout.contains("OVERRIDE_VAR=env_value"));

        copy_fixture(".env.local", &temp_dir, ".env.local");

        let output = Command::new(&binary_path)
            .arg("echo-env")
            .current_dir(&temp_dir)
            .env(env_var, env_val)
            .output()
            .expect("Failed to execute command with .env.local");

        let stdout = str::from_utf8(&output.stdout).unwrap();
        assert!(stdout.contains("ENV_VAR=env_value"));
        assert!(stdout.contains("LOCAL_VAR=local_value"));
        assert!(stdout.contains("OVERRIDE_VAR=local_value"));
    });
}

/// Helper to spawn the hoi binary with proper mocked HOME
fn run_hoi_command(binary: &Path, args: &[&str], cwd: &Path) -> Output {
    let mut cmd = Command::new(binary);
    cmd.args(args).current_dir(cwd);

    #[cfg(not(windows))]
    cmd.env("HOME", cwd);
    #[cfg(windows)]
    cmd.env("USERPROFILE", cwd);

    cmd.output().expect("Failed to execute hoi binary")
}

fn copy_fixture(name: &str, target_dir: &Path, target_filename: &str) {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_path = Path::new(&manifest_dir)
        .join("tests")
        .join("fixtures")
        .join(name);

    let contents = std::fs::read_to_string(&src_path).expect("Failed to read fixture");

    #[cfg(windows)]
    let contents = contents
        .replace("$ENV_VAR", "%ENV_VAR%")
        .replace("$LOCAL_VAR", "%LOCAL_VAR%")
        .replace("$OVERRIDE_VAR", "%OVERRIDE_VAR%");

    std::fs::write(target_dir.join(target_filename), contents)
        .expect("Failed to write test config");
}
