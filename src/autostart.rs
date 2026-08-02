use std::env;
#[allow(unused_imports)]
use std::fs;
#[allow(unused_imports)]
use std::path::PathBuf;
use std::process::Command;

pub fn enable_autostart() {
    let exe_path = env::current_exe().unwrap();
    let exe_str = exe_path.to_string_lossy();

    #[cfg(target_os = "windows")]
    {
        Command::new("reg")
            .args([
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "ChronxDaemon",
                "/t",
                "REG_SZ",
                "/d",
                &format!("\"{}\" daemon", exe_str),
                "/f",
            ])
            .output()
            .expect("Failed to add registry key");
    }

    #[cfg(target_os = "macos")]
    {
        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.chronx.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
            exe_str
        );
        let home = std::env::var("HOME").unwrap();
        let plist_path = PathBuf::from(home).join("Library/LaunchAgents/com.chronx.daemon.plist");
        fs::write(&plist_path, plist_content).unwrap();
        Command::new("launchctl")
            .args(["load", plist_path.to_str().unwrap()])
            .output()
            .unwrap();
    }

    #[cfg(target_os = "linux")]
    {
        let desktop_content = format!(
            r#"[Desktop Entry]
Type=Application
Exec={} daemon
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
Name=Chronx
Comment=Chronx Background Daemon
"#,
            exe_str
        );
        let home = std::env::var("HOME").unwrap();
        let autostart_dir = PathBuf::from(home).join(".config/autostart");
        fs::create_dir_all(&autostart_dir).unwrap();
        let desktop_path = autostart_dir.join("chronx.desktop");
        fs::write(desktop_path, desktop_content).unwrap();
    }
}

pub fn disable_autostart() {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("reg")
            .args([
                "delete",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "ChronxDaemon",
                "/f",
            ])
            .output();
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap();
        let plist_path = PathBuf::from(home).join("Library/LaunchAgents/com.chronx.daemon.plist");
        if plist_path.exists() {
            let _ = Command::new("launchctl")
                .args(["unload", plist_path.to_str().unwrap()])
                .output();
            let _ = fs::remove_file(plist_path);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap();
        let desktop_path = PathBuf::from(home).join(".config/autostart/chronx.desktop");
        if desktop_path.exists() {
            let _ = fs::remove_file(desktop_path);
        }
    }
}
