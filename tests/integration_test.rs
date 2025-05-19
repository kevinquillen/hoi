use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str;
use temp_env::with_var;
use tempfile::TempDir;

fn create_test_config(dir: &Path) {
    let config_path = dir.join(".hoi.yml");
    let mut file = File::create(&config_path).unwrap();
    writeln!(file, "version: 1").unwrap();
    writeln!(file, "description: \"Integration test config\"").unwrap();
    writeln!(file, "commands:").unwrap();
    writeln!(file, "  echo-test:").unwrap();
    writeln!(file, "    cmd: echo \"Integration test successful\"").unwrap();
    writeln!(file, "    description: \"Prints a test success message\"").unwrap();
}

fn create_env_config(dir: &Path, vars: &HashMap<&str, &str>) {
    let env_path = dir.join(".env");
    let mut file = File::create(&env_path).unwrap();
    for (key, value) in vars {
        writeln!(file, "{}={}", key, value).unwrap();
    }
}

fn create_env_local_config(dir: &Path, vars: &HashMap<&str, &str>) {
    let env_local_path = dir.join(".env.local");
    let mut file = File::create(&env_local_path).unwrap();
    for (key, value) in vars {
        writeln!(file, "{}={}", key, value).unwrap();
    }
}

fn create_test_config_with_env_commands(dir: &Path) {
    let config_path = dir.join(".hoi.yml");
    let mut file = File::create(&config_path).unwrap();
    writeln!(file, "version: 1").unwrap();
    writeln!(file, "description: \"Integration test config\"").unwrap();
    writeln!(file, "commands:").unwrap();
    writeln!(file, "  echo-env:").unwrap();

    // Write command using platform-specific syntax
    #[cfg(windows)]
    writeln!(
        file,
        "    cmd: echo ENV_VAR=%ENV_VAR% LOCAL_VAR=%LOCAL_VAR% OVERRIDE_VAR=%OVERRIDE_VAR%"
    )
    .unwrap();

    #[cfg(not(windows))]
    writeln!(
        file,
        "    cmd: echo \"ENV_VAR=$ENV_VAR LOCAL_VAR=$LOCAL_VAR OVERRIDE_VAR=$OVERRIDE_VAR\""
    )
    .unwrap();

    writeln!(file, "    description: \"Prints environment variables\"").unwrap();
}

fn create_global_test_config(dir: &Path) {
    let hoi_dir = dir.join(".hoi");
    fs::create_dir_all(&hoi_dir).unwrap();

    let config_path = hoi_dir.join(".hoi.global.yml");
    let mut file = File::create(&config_path).unwrap();
    writeln!(file, "version: 1").unwrap();
    writeln!(file, "description: \"Global integration test config\"").unwrap();
    writeln!(file, "commands:").unwrap();
    writeln!(file, "  global-echo:").unwrap();
    writeln!(file, "    cmd: echo \"Global command successful\"").unwrap();
    writeln!(file, "    alias: ge").unwrap();
    writeln!(
        file,
        "    description: \"Prints a global command success message\""
    )
    .unwrap();
}

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
    let temp_dir = TempDir::new().unwrap();
    create_test_config(temp_dir.path());
    // First build the binary
    Command::new("cargo")
        .args(["build"])
        .status()
        .expect("Failed to build hoi binary");

    let binary_path = get_binary_path();
    let output = Command::new(binary_path)
        .current_dir(temp_dir.path())
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
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    #[cfg(windows)]
    let (env_var, env_val) = ("USERPROFILE", temp_path);
    #[cfg(not(windows))]
    let (env_var, env_val) = ("HOME", temp_path);

    with_var(env_var, Some(env_val.to_str().unwrap()), || {
        // Set up config inside isolated "home"
        create_test_config(temp_path);
        create_global_test_config(temp_path);

        // Build binary
        Command::new("cargo")
            .args(["build"])
            .status()
            .expect("Failed to build hoi binary");

        let binary_path = get_binary_path();

        // Run local command
        let output = run_hoi_command(&binary_path, &["echo-test"], temp_path);
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Integration test successful"));

        // Run alias for global command
        let output = run_hoi_command(&binary_path, &["ge"], temp_path);
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Global command successful"));

        // List commands (should include global)
        let output = run_hoi_command(&binary_path, &[], temp_path);
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("global-echo"));

        // Run global command explicitly
        let output = run_hoi_command(&binary_path, &["global-echo"], temp_path);
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Global command successful"));
    });
}

#[test]
fn test_hoi_with_env_files() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    #[cfg(windows)]
    let (env_var, env_val) = ("USERPROFILE", temp_path);
    #[cfg(not(windows))]
    let (env_var, env_val) = ("HOME", temp_path);

    with_var(env_var, Some(env_val.to_str().unwrap()), || {
        create_test_config_with_env_commands(temp_path);

        // Build the binary
        Command::new("cargo")
            .args(["build"])
            .status()
            .expect("Failed to build hoi binary");

        let binary_path = get_binary_path();

        // Test 1: With .env file only
        let env_vars = HashMap::from([("ENV_VAR", "env_value"), ("OVERRIDE_VAR", "env_value")]);
        create_env_config(temp_path, &env_vars);

        let output = Command::new(&binary_path)
            .arg("echo-env")
            .current_dir(temp_path)
            .env(env_var, env_val)
            .output()
            .expect("Failed to execute command with .env");

        let stdout = str::from_utf8(&output.stdout).unwrap();
        assert!(stdout.contains("ENV_VAR=env_value"));
        assert!(!stdout.contains("LOCAL_VAR=local_value")); // not yet set
        assert!(stdout.contains("OVERRIDE_VAR=env_value"));

        // Remove .env
        fs::remove_file(temp_path.join(".env")).unwrap();

        // Test 2: With .env.local file only
        let env_local_vars = HashMap::from([
            ("LOCAL_VAR", "local_value"),
            ("OVERRIDE_VAR", "local_value"),
        ]);
        create_env_local_config(temp_path, &env_local_vars);

        let output = Command::new(&binary_path)
            .arg("echo-env")
            .current_dir(temp_path)
            .env(env_var, env_val)
            .output()
            .expect("Failed to execute command with .env.local");

        let stdout = str::from_utf8(&output.stdout).unwrap();
        assert!(!stdout.contains("ENV_VAR=env_value")); // should not be present
        assert!(stdout.contains("LOCAL_VAR=local_value"));
        assert!(stdout.contains("OVERRIDE_VAR=local_value")); // override

        // Test 3: With both .env and .env.local
        create_env_config(temp_path, &env_vars); // add .env back

        let output = Command::new(&binary_path)
            .arg("echo-env")
            .current_dir(temp_path)
            .env(env_var, env_val)
            .output()
            .expect("Failed to execute command with both .env and .env.local");

        let stdout = str::from_utf8(&output.stdout).unwrap();
        assert!(stdout.contains("ENV_VAR=env_value")); // from .env
        assert!(stdout.contains("LOCAL_VAR=local_value")); // from .env.local
        assert!(stdout.contains("OVERRIDE_VAR=local_value")); // .env.local overrides
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
