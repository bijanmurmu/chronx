use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_cli_help_menu() {
    let bin_path = env!("CARGO_BIN_EXE_chronx");
    let output = Command::new(bin_path)
        .args(["--help"])
        .output()
        .expect("Failed to execute cargo run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Continuous, invisible Undo button"));
    assert!(stdout.contains("watch"));
    assert!(stdout.contains("init"));
}

#[test]
fn test_global_config_path_creation() {
    let home = directories::UserDirs::new().expect("Failed to get user dirs").home_dir().to_path_buf();
    assert!(home.exists(), "Home directory should exist");
}

#[test]
fn test_cli_init_creates_directories() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let bin_path = env!("CARGO_BIN_EXE_chronx");
    let output = Command::new(bin_path)
        .args(["init"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute cargo run");

    assert!(output.status.success());
    
    let chronx_dir = temp_path.join(".chronx");
    let history_dir = chronx_dir.join("history");
    
    assert!(chronx_dir.exists(), ".chronx folder was not created");
    assert!(history_dir.exists(), ".chronx/history folder was not created");
}

#[test]
fn test_cli_squash_no_repo() {
    let temp_dir = TempDir::new().unwrap();
    
    let bin_path = env!("CARGO_BIN_EXE_chronx");
    let output = Command::new(bin_path)
        .args(["squash"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute cargo run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Could not determine where to squash to") || !output.status.success());
}
