use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
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

    // Create both local and global configs
    create_test_config(temp_dir.path());
    create_global_test_config(temp_dir.path());

    // Set the HOME env var to our temp dir for testing
    let original_home = env::var("HOME").ok();
    env::set_var("HOME", temp_dir.path());

    // First build the binary
    Command::new("cargo")
        .args(["build"])
        .status()
        .expect("Failed to build hoi binary");

    let binary_path = get_binary_path();

    let output = Command::new(binary_path)
        .arg("echo-test")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed with status: {:?}",
        output.status
    );

    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("Running command echo-test..."));
    assert!(stdout.contains("Integration test successful"));

    let binary_path = get_binary_path();

    // Test an alias
    let output = Command::new(binary_path)
        .arg("ge")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed with status: {:?}",
        output.status
    );

    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("Running command ge..."));
    assert!(stdout.contains("Global command successful"));

    // Build the binary
    Command::new("cargo")
        .args(["build"])
        .status()
        .expect("Failed to build hoi binary");

    let binary_path = get_binary_path();

    // Test listing commands - should show global commands
    let list_output = Command::new(&binary_path)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(
        list_output.status.success(),
        "List command failed with status: {:?}",
        list_output.status
    );

    let list_stdout = str::from_utf8(&list_output.stdout).unwrap();
    assert!(list_stdout.contains("global-echo"));

    // Test executing global command
    let exec_output = Command::new(&binary_path)
        .arg("global-echo")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(
        exec_output.status.success(),
        "Execute command failed with status: {:?}",
        exec_output.status
    );

    let exec_stdout = str::from_utf8(&exec_output.stdout).unwrap();
    assert!(exec_stdout.contains("Running command global-echo..."));
    assert!(exec_stdout.contains("Global command successful"));

    // Test executing local command
    let local_exec_output = Command::new(&binary_path)
        .arg("echo-test")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(
        local_exec_output.status.success(),
        "Execute local command failed with status: {:?}",
        local_exec_output.status
    );

    // Test executing global command
    let global_exec_output = Command::new(&binary_path)
        .arg("global-echo")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(
        global_exec_output.status.success(),
        "Execute global command failed with status: {:?}",
        global_exec_output.status
    );

    // Restore original HOME env var if it existed
    if let Some(home) = original_home {
        env::set_var("HOME", home);
    } else {
        env::remove_var("HOME");
    }
}
