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
        println!("Added {} to global watch list.", dir);
    } else {
        println!("Directory is already being tracked.");
    }
}

fn run_install_daemon() {
    autostart::enable_autostart();
    
    #[cfg(target_os = "windows")]
    {
        OsCommand::new(env::current_exe().unwrap())
            .arg("daemon")
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .expect("Failed to start daemon in background");
        println!("Daemon started in background.");
    }
    #[cfg(not(target_os = "windows"))]
    {
        println!("Please reboot or start the daemon manually to begin.");
    }
}

fn run_init() {
    let current_dir = env::current_dir().unwrap().to_string_lossy().to_string();
    add_dir_to_config(current_dir);
    let chronx_dir = std::path::Path::new(".chronx");
    let history_dir = chronx_dir.join("history");
    fs::create_dir_all(&history_dir).unwrap_or(());
    println!("Initialized .chronx tracking for this directory.");
}

fn run_log() {
    let history_dir = std::path::Path::new(".chronx/history");
    if history_dir.exists() {
        let mut entries: Vec<_> = std::fs::read_dir(history_dir).unwrap().filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.metadata().unwrap().modified().unwrap()));
        
        let mut display_items = Vec::new();
        let mut snapshots = Vec::new();
        
        for entry in entries.iter().take(50) {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(snapshot) = serde_json::from_str::<watcher::Snapshot>(&content) {
                    display_items.push(format!("[{}] {} - {}", snapshot.timestamp, snapshot.event_type.to_uppercase(), snapshot.path));
                    snapshots.push(snapshot);
                }
            }
        }
        
        if display_items.is_empty() {
            println!("No history found.");
            return;
        }
        
        loop {
            println!("Here is the timeline of your recent changes:");
            if let Ok(selection) = dialoguer::Select::new()
                .with_prompt("Select a snapshot to preview/recover (Use arrows, Enter to select, Esc to quit)")
                .items(&display_items)
                .clear(true)
                .interact_on_opt(&console::Term::stdout())
            {
                if let Some(selection) = selection {
                    let selected = &snapshots[selection];
                    
                    if selected.event_type == "remove" {
                        println!("\n[Preview] This is a deletion event. The file content is empty.\n");
                    } else {
                        let current_content = fs::read_to_string(&selected.path).unwrap_or_default();
                        let diff = similar::TextDiff::from_lines(&current_content, &selected.content);
                        
                        println!("\n--- Diff Preview: Changes that will be applied ---");
                        for change in diff.iter_all_changes() {
                            let (sign, style) = match change.tag() {
                                similar::ChangeTag::Delete => ("-", console::Style::new().red()),
                                similar::ChangeTag::Insert => ("+", console::Style::new().green()),
                                similar::ChangeTag::Equal => (" ", console::Style::new().dim()),
                            };
                            print!("{}{}", style.apply_to(sign), style.apply_to(change));
                        }
                        println!("--------------------------------------------------\n");
                    }

                    if dialoguer::Confirm::new()
                        .with_prompt(format!("Recover {} to this state?", selected.path))
                        .default(false)
                        .interact_on(&console::Term::stdout())
                        .unwrap_or(false)
                    {
                        if selected.event_type == "remove" {
                            println!("Cannot recover a deletion directly. Select an older creation or modification event.");
                        } else {
                            fs::write(&selected.path, &selected.content).unwrap();
                            println!("Recovered {} to its state from {}", selected.path, selected.timestamp);
                            break;
                        }
                    } else {
                        println!("Preview cancelled.");
                    }
                } else {
                    // User pressed Esc or q
                    break;
                }
            } else {
                break;
            }
        }
    } else {
        println!("No history found.");
    }
}

fn run_squash(message: &Option<String>) {
    let current_branch = String::from_utf8_lossy(&OsCommand::new("git").args(["branch", "--show-current"]).output().unwrap().stdout).trim().to_string();
    
    let target_commit: String;
    
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
                println!("Successfully squashed and committed as: \"{}\"", msg);
            } else {
                println!("Failed to automatically commit.");
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

fn pause_for_return() {
    println!("\nPress Enter to return to the menu...");
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf);
}

fn print_help_guide() {
    println!("{}", console::style("\n📖 How to use Chronx").bold().cyan());
    println!("Chronx is an invisible undo button for your file system. Here is how to use it:\n");
    println!("{} Navigate to any folder in your terminal and select {}", console::style("1.").bold().yellow(), console::style("[ SETUP ]").bold());
    println!("   This creates a hidden `.chronx` folder to store your history.\n");
    println!("{} Select {} to begin tracking changes.", console::style("2.").bold().yellow(), console::style("[ DAEMON ]").bold());
    println!("   (Leave this terminal open, or run the global daemon, and edit your files normally).\n");
    println!("{} Whenever you want to undo a mistake, open Chronx and select {}.", console::style("3.").bold().yellow(), console::style("[ RECOVER ]").bold());
    println!("   You'll see a timeline of every save and can preview exactly what changed before recovering.\n");
    println!("{}", console::style("Pro-Tip:").bold().magenta());
    println!("Run `cargo install --path .` in the Chronx source code folder to install it globally.");
    println!("Then you can just type `chronx` in any terminal on your computer!");
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
                run_install_daemon();
            }
            Commands::Daemon => {
                watcher::start_global_daemon();
            }
            Commands::Init => {
                run_init();
            }
            Commands::Log => {
                run_log();
            }
            Commands::Squash { message } => {
                run_squash(message);
            }
        },
        None => loop {
            console::Term::stdout().clear_screen().unwrap_or(());
            let options = vec![
                "[ RECOVER ]  View History & Recover",
                "[  SETUP  ]  Start Tracking Current Directory",
                "[ DAEMON  ]  Start Foreground Watcher",
                "[   GIT   ]  Squash Git Commits",
                "[ SYSTEM  ]  Install Global Background Daemon",
                "[  HELP   ]  How to use Chronx",
                "[  EXIT   ]  Exit"
            ];
            
            let status_text = if is_daemon_running() {
                console::style("RUNNING (Watching in background)").bold().green()
            } else {
                console::style("STOPPED (Not watching)").bold().dim()
            };
            
            println!("{}", console::style("Chronx v1.0.1\n═════════════════════════════════════").bold().cyan());
            println!("Status: {}\n", status_text);
            
            if let Ok(Some(selection)) = dialoguer::Select::new()
                .with_prompt("What would you like to do? (Esc to quit)")
                .items(&options)
                .default(0)
                .clear(true)
                .interact_on_opt(&console::Term::stdout())
            {
                match selection {
                    0 => run_log(),
                    1 => {
                        run_init();
                        pause_for_return();
                    },
                    2 => watcher::start_watching_single(env::current_dir().unwrap()),
                    3 => {
                        run_squash(&None);
                        pause_for_return();
                    },
                    4 => {
                        run_install_daemon();
                        pause_for_return();
                    },
                    5 => {
                        print_help_guide();
                        pause_for_return();
                    },
                    _ => {
                        println!("Goodbye!");
                        break;
                    }
                }
            } else {
                // User pressed Esc or closed the prompt
                println!("Goodbye!");
                break;
            }
        }
    }
}
