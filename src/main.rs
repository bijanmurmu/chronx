mod watcher;
mod autostart;

use clap::{Parser, Subcommand};
use std::process::Command as OsCommand;
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
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Watch a single directory immediately (Foreground)
    Watch,
    /// View the timeline of changes for the current directory
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

fn add_dir_to_config(dir: String) {
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
        println!("✅ Added {} to global watch list.", dir);
    } else {
        println!("Directory is already being tracked.");
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Watch => {
            watcher::start_watching_single(env::current_dir().unwrap());
        }
        Commands::InstallDaemon => {
            autostart::enable_autostart();
            
            #[cfg(target_os = "windows")]
            {
                OsCommand::new(env::current_exe().unwrap())
                    .arg("daemon")
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW
                    .spawn()
                    .expect("Failed to start daemon in background");
                println!("✅ Daemon started in background.");
            }
            #[cfg(not(target_os = "windows"))]
            {
                println!("Please reboot or start the daemon manually to begin.");
            }
        }
        Commands::Daemon => {
            watcher::start_global_daemon();
        }
        Commands::Init => {
            let current_dir = env::current_dir().unwrap().to_string_lossy().to_string();
            add_dir_to_config(current_dir);
            let chronx_dir = std::path::Path::new(".chronx");
            let history_dir = chronx_dir.join("history");
            fs::create_dir_all(&history_dir).unwrap_or(());
            println!("✅ Initialized .chronx tracking for this directory.");
        }
        Commands::Log => {
            println!("Here is the timeline of your recent changes:");
            let history_dir = std::path::Path::new(".chronx/history");
            if history_dir.exists() {
                let mut entries: Vec<_> = std::fs::read_dir(history_dir).unwrap().filter_map(|e| e.ok()).collect();
                entries.sort_by_key(|e| e.metadata().unwrap().modified().unwrap());
                for entry in entries.iter().rev().take(10) {
                    println!("- {}", entry.file_name().to_string_lossy());
                }
            } else {
                println!("No history found.");
            }
        }
        Commands::Squash { message } => {
            let current_branch = String::from_utf8_lossy(&OsCommand::new("git").args(["branch", "--show-current"]).output().unwrap().stdout).trim().to_string();
            
            let mut target_commit = String::new();
            
            if current_branch == "main" || current_branch == "master" {
                // If on main, squash everything down to the very first commit
                let root_commit = OsCommand::new("git")
                    .args(["rev-list", "--max-parents=0", "HEAD"])
                    .output()
                    .expect("Failed to get root commit");
                target_commit = String::from_utf8_lossy(&root_commit.stdout).trim().to_string();
            } else {
                // If on a feature branch, squash down to where it split from main
                let base_commit = OsCommand::new("git")
                    .args(["merge-base", "HEAD", "main"])
                    .output()
                    .unwrap_or_else(|_| OsCommand::new("git").args(["merge-base", "HEAD", "master"]).output().unwrap());
                target_commit = String::from_utf8_lossy(&base_commit.stdout).trim().to_string();
            }

            if target_commit.is_empty() {
                println!("Could not determine where to squash to.");
                return;
            }

            let head_commit = String::from_utf8_lossy(&OsCommand::new("git").args(["rev-parse", "HEAD"]).output().unwrap().stdout).trim().to_string();
            if target_commit == head_commit {
                println!("Nothing to squash! You are already at the base commit.");
                return;
            }

            println!("Squashing commits back to: {}", target_commit);
            
            let reset_status = OsCommand::new("git")
                .args(["reset", "--soft", &target_commit])
                .status()
                .expect("Failed to execute git reset");
                
            if reset_status.success() {
                if let Some(msg) = message {
                    let commit_status = if current_branch == "main" || current_branch == "master" {
                        OsCommand::new("git").args(["commit", "--amend", "-m", msg]).status().expect("Failed to commit")
                    } else {
                        OsCommand::new("git").args(["commit", "-m", msg]).status().expect("Failed to commit")
                    };
                    if commit_status.success() {
                        println!("✅ Successfully squashed and committed as: \"{}\"", msg);
                    } else {
                        println!("❌ Failed to automatically commit.");
                    }
                } else {
                    if current_branch == "main" || current_branch == "master" {
                        println!("Successfully soft-reset to the first commit. Run `git commit --amend` to squash the entire repo into one single commit!");
                    } else {
                        println!("Successfully soft-reset to {}. You can now run `git commit` to create a single clean commit!", target_commit);
                    }
                }
            } else {
                println!("Failed to reset git history.");
            }
        }
    }
}
