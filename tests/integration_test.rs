use std::env;
use std::fs::File;
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
    writeln!(file, "entrypoint:").unwrap();
    writeln!(file, "  - bash").unwrap();
    writeln!(file, "  - -e").unwrap();
    writeln!(file, "  - -c").unwrap();
    writeln!(file, "  - \"$@\"").unwrap();
    writeln!(file, "commands:").unwrap();
    writeln!(file, "  echo-test:").unwrap();
    writeln!(file, "    cmd: echo \"Integration test successful\"").unwrap();
    writeln!(file, "    usage: \"Test echo command\"").unwrap();
    writeln!(file, "    description: \"Prints a test success message\"").unwrap();
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
    assert!(stdout.contains("Test echo command"));
    assert!(stdout.contains("Prints a test success message"));
}

#[test]
fn test_hoi_execute_command() {
    let temp_dir = TempDir::new().unwrap();
    create_test_config(temp_dir.path());

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
}
