mod watcher;
mod autostart;
mod tui;

use clap::{Parser, Subcommand};
use std::process::{Command as OsCommand, Stdio};
use std::path::PathBuf;
use std::{env, fs};
use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Parser)]
#[command(name = "chronx")]
#[command(about = "Continuous, invisible Undo button for your local file system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Watch a single directory immediately (Foreground)
    Watch,
    /// View the timeline of changes interactively
    Log,

    /// Squash all commits on the current branch down to a single clean commit
    Squash {
        /// Optional commit message. If provided, chronx will automatically commit the squashed changes for you.
        #[arg(short, long)]
        message: Option<String>,
    },
    /// Add the tool to system startup and start the global daemon
    InstallDaemon,
    /// Run the global daemon (used by OS on startup)
    Daemon,
    /// Tell the global daemon to start tracking the current directory
    Init,
}

#[derive(Serialize, Deserialize, Default)]
struct GlobalConfig {
    watch_dirs: Vec<String>,
}

fn get_global_config_path() -> PathBuf {
    let home = directories::UserDirs::new().unwrap().home_dir().to_path_buf();
    let chronx_dir = home.join(".chronx_global");
    fs::create_dir_all(&chronx_dir).unwrap();
    chronx_dir.join("config.json")
}

fn add_dir_to_config(dir: String) -> String {
    let config_path = get_global_config_path();
    let mut config: GlobalConfig = if config_path.exists() {
        let content = fs::read_to_string(&config_path).unwrap();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        GlobalConfig::default()
    };

    if !config.watch_dirs.contains(&dir) {
        config.watch_dirs.push(dir.clone());
        fs::write(config_path, serde_json::to_string(&config).unwrap()).unwrap();
        format!("Added {} to global watch list.\n", dir)
    } else {
        "Directory is already being tracked.\n".to_string()
    }
}

pub fn run_install_daemon() -> String {
    autostart::enable_autostart();
    
    #[cfg(target_os = "windows")]
    {
        OsCommand::new(env::current_exe().unwrap())
            .arg("daemon")
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start daemon in background");
        return "Daemon started in background.".to_string();
    }
    #[cfg(not(target_os = "windows"))]
    {
        return "Please reboot or start the daemon manually to begin.".to_string();
    }
}

pub fn run_uninstall_daemon() -> String {
    autostart::disable_autostart();
    
    #[cfg(target_os = "windows")]
    {
        let _ = OsCommand::new("powershell")
            .args(["-Command", "Get-WmiObject Win32_Process -Filter \"Name='chronx.exe' AND CommandLine LIKE '%daemon%'\" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }"])
            .output();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = OsCommand::new("pkill").arg("-f").arg("chronx daemon").output();
    }
    
    "Daemon stopped and removed from system startup.".to_string()
}

pub fn run_init() -> String {
    let current_dir = env::current_dir().unwrap().to_string_lossy().to_string();
    let mut out = add_dir_to_config(current_dir);
    let chronx_dir = std::path::Path::new(".chronx");
    let history_dir = chronx_dir.join("history");
    fs::create_dir_all(&history_dir).unwrap_or(());
    out.push_str("Initialized .chronx tracking for this directory.");
    out
}



pub fn run_squash(message: &Option<String>) -> String {
    let current_branch = String::from_utf8_lossy(&OsCommand::new("git").args(["branch", "--show-current"]).output().unwrap().stdout).trim().to_string();
    
    let target_commit: String;
    
    if current_branch == "main" || current_branch == "master" {
        let root_commit = OsCommand::new("git")
            .args(["rev-list", "--max-parents=0", "HEAD"])
            .output()
            .expect("Failed to get root commit");
        target_commit = String::from_utf8_lossy(&root_commit.stdout).trim().to_string();
    } else {
        let base_commit = OsCommand::new("git")
            .args(["merge-base", "HEAD", "main"])
            .output()
            .unwrap_or_else(|_| OsCommand::new("git").args(["merge-base", "HEAD", "master"]).output().unwrap());
        target_commit = String::from_utf8_lossy(&base_commit.stdout).trim().to_string();
    }

    if target_commit.is_empty() {
        return "Could not determine where to squash to.".to_string();
    }

    let head_commit = String::from_utf8_lossy(&OsCommand::new("git").args(["rev-parse", "HEAD"]).output().unwrap().stdout).trim().to_string();
    if target_commit == head_commit {
        return "Nothing to squash! You are already at the base commit.".to_string();
    }

    let mut out = format!("Squashing commits back to: {}\n", target_commit);
    
    let reset_status = OsCommand::new("git")
        .args(["reset", "--soft", &target_commit])
        .output()
        .expect("Failed to execute git reset");
        
    if reset_status.status.success() {
        if let Some(msg) = message {
            let commit_status = if current_branch == "main" || current_branch == "master" {
                OsCommand::new("git").args(["commit", "--amend", "-m", msg]).output().expect("Failed to commit")
            } else {
                OsCommand::new("git").args(["commit", "-m", msg]).output().expect("Failed to commit")
            };
            if commit_status.status.success() {
                out.push_str(&format!("Successfully squashed and committed as: \"{}\"", msg));
            } else {
                out.push_str("Failed to automatically commit.");
            }
        } else {
            if current_branch == "main" || current_branch == "master" {
                out.push_str("Successfully soft-reset to the first commit. Run `git commit --amend` to squash the entire repo into one single commit!");
            } else {
                out.push_str(&format!("Successfully soft-reset to {}. You can now run `git commit` to create a single clean commit!", target_commit));
            }
        }
    } else {
        out.push_str("Failed to reset git history.");
    }
    out
}



fn is_daemon_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = OsCommand::new("tasklist").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            return output_str.matches("chronx.exe").count() > 1;
        }
        false
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(output) = OsCommand::new("pgrep").arg("-x").arg("chronx").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            return output_str.lines().count() > 1;
        }
        false
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(cmd) => match cmd {
            Commands::Watch => {
                watcher::start_watching_single(env::current_dir().unwrap());
            }
            Commands::InstallDaemon => {
                let out = run_install_daemon();
                println!("{}", out);
            }
            Commands::Daemon => {
                watcher::start_global_daemon();
            }
            Commands::Init => {
                let out = run_init();
                println!("{}", out);
            }
            Commands::Log => {
                // run_log(); 
                println!("The text-based log has been upgraded. Run `chronx` without arguments to use the new interactive dashboard!");
            }
            Commands::Squash { message } => {
                let out = run_squash(message);
                println!("{}", out);
            }
        },
        None => loop {
            let running = is_daemon_running();
            let action = match tui::run_tui(running) {
                Ok(act) => act,
                Err(_) => break,
            };
            
            match action {
                tui::MenuAction::Quit => {
                    break;
                }
            }
        }
    }
}
