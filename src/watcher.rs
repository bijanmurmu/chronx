use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use chrono::Utc;
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub timestamp: String,
    pub path: String,
    pub content: String,
}

pub fn start_watching_single(dir: PathBuf) {
    let chronx_dir = dir.join(".chronx");
    let history_dir = chronx_dir.join("history");
    fs::create_dir_all(&history_dir).unwrap_or(());

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        match res {
            Ok(event) => {
                if event.kind.is_modify() {
                    for path in event.paths {
                        let path_str = path.to_string_lossy();
                        if path_str.contains(".chronx") || path_str.contains(".git") || path_str.contains("target") || path_str.contains("node_modules") {
                            continue;
                        }

                        if path.is_file() {
                            if let Ok(content) = fs::read_to_string(&path) {
                                let timestamp = Utc::now().to_rfc3339().replace(":", "-");
                                let file_name = path.file_name().unwrap().to_string_lossy();
                                
                                let snapshot = Snapshot {
                                    timestamp: timestamp.clone(),
                                    path: path_str.to_string(),
                                    content,
                                };
                                
                                let out_path = history_dir.join(format!("{}_{}.json", timestamp, file_name));
                                let json = serde_json::to_string(&snapshot).unwrap_or_default();
                                let _ = fs::write(out_path, json);
                                println!("Saved state for {}", path_str);
                            }
                        }
                    }
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }).unwrap();

    watcher.watch(&dir, RecursiveMode::Recursive).unwrap();

    println!("👀 Chronx is now watching your files in the background...");
    
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

pub fn start_global_daemon() {
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        match res {
            Ok(event) => {
                if event.kind.is_modify() {
                    for path in event.paths {
                        let path_str = path.to_string_lossy();
                        if path_str.contains(".chronx") || path_str.contains(".git") || path_str.contains("target") || path_str.contains("node_modules") {
                            continue;
                        }

                        if path.is_file() {
                            let mut current_dir = path.parent();
                            while let Some(dir) = current_dir {
                                let chronx_dir = dir.join(".chronx");
                                if chronx_dir.exists() {
                                    let history_dir = chronx_dir.join("history");
                                    if let Ok(content) = fs::read_to_string(&path) {
                                        let timestamp = Utc::now().to_rfc3339().replace(":", "-");
                                        let file_name = path.file_name().unwrap().to_string_lossy();
                                        
                                        let snapshot = Snapshot {
                                            timestamp: timestamp.clone(),
                                            path: path_str.to_string(),
                                            content,
                                        };
                                        
                                        let out_path = history_dir.join(format!("{}_{}.json", timestamp, file_name));
                                        let json = serde_json::to_string(&snapshot).unwrap_or_default();
                                        let _ = fs::write(out_path, json);
                                    }
                                    break;
                                }
                                current_dir = dir.parent();
                            }
                        }
                    }
                }
            },
            Err(_) => {},
        }
    }).unwrap();

    let mut currently_watched: Vec<String> = Vec::new();

    loop {
        let home = directories::UserDirs::new().unwrap().home_dir().to_path_buf();
        let config_path = home.join(".chronx_global").join("config.json");
        
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(dirs) = config.get("watch_dirs").and_then(|d| d.as_array()) {
                        for dir_val in dirs {
                            if let Some(dir) = dir_val.as_str() {
                                if !currently_watched.contains(&dir.to_string()) {
                                    let p = Path::new(dir);
                                    if p.exists() {
                                        let _ = watcher.watch(p, RecursiveMode::Recursive);
                                        currently_watched.push(dir.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
