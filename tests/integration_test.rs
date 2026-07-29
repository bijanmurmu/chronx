use std::process::Command;

#[test]
fn test_cli_help_menu() {
    // Ensure the CLI parses correctly without panicking
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute cargo run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Continuous, invisible Undo button"));
    assert!(stdout.contains("watch"));
    assert!(stdout.contains("log"));
}

#[test]
fn test_global_config_path_creation() {
    // This is a basic integration check to ensure directories crate works on Windows
    let home = directories::UserDirs::new().expect("Failed to get user dirs").home_dir().to_path_buf();
    assert!(home.exists(), "Home directory should exist");
}
