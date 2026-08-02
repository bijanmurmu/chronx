use notify::{RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use chrono::Utc;
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub timestamp: String,
    pub path: String,
    pub content: String,
    pub event_type: String,
}

pub fn start_watching_single(dir: PathBuf) {
    let chronx_dir = dir.join(".chronx");
    let history_dir = chronx_dir.join("history");
    fs::create_dir_all(&history_dir).unwrap_or(());

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        match res {
            Ok(event) => {
                let is_modify = event.kind.is_modify();
                let is_create = event.kind.is_create();
                let is_remove = event.kind.is_remove();
                
                if is_modify || is_create || is_remove {
                    for path in event.paths {
                        let path_str = path.to_string_lossy();
                        if path_str.contains(".chronx") || path_str.contains(".git") || path_str.contains("target") || path_str.contains("node_modules") {
                            continue;
                        }

                        let event_type = if is_create {
                            "create"
                        } else if is_remove {
                            "remove"
                        } else {
                            "modify"
                        };

                        if path.is_file() || is_remove {
                            let content = if is_remove {
                                String::from("<DELETED>")
                            } else {
                                fs::read_to_string(&path).unwrap_or_default()
                            };

                            let timestamp = Utc::now().to_rfc3339().replace(":", "-");
                            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                            
                            let snapshot = Snapshot {
                                timestamp: timestamp.clone(),
                                path: path_str.to_string(),
                                content,
                                event_type: event_type.to_string(),
                            };
                            
                            let out_path = history_dir.join(format!("{}_{}_{}.json", timestamp, event_type, file_name));
                            let json = serde_json::to_string(&snapshot).unwrap_or_default();
                            let _ = fs::write(out_path, json);
                            println!("Saved state for {} ({})", path_str, event_type);
                        }
                    }
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }).unwrap();

    watcher.watch(&dir, RecursiveMode::Recursive).unwrap();

    println!("Chronx is now watching your files in the foreground!");
    println!("Press Enter to stop watching and return to the main menu...");
    
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf);
}

pub fn start_watching_single_thread(dir: PathBuf) {
    std::thread::spawn(move || {
        let chronx_dir = dir.join(".chronx");
        let history_dir = chronx_dir.join("history");
        fs::create_dir_all(&history_dir).unwrap_or(());

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            match res {
                Ok(event) => {
                    let is_modify = event.kind.is_modify();
                    let is_create = event.kind.is_create();
                    let is_remove = event.kind.is_remove();
                    
                    if is_modify || is_create || is_remove {
                        for path in event.paths {
                            let path_str = path.to_string_lossy();
                            if path_str.contains(".chronx") || path_str.contains(".git") || path_str.contains("target") || path_str.contains("node_modules") {
                                continue;
                            }

                            let event_type = if is_create {
                                "create"
                            } else if is_remove {
                                "remove"
                            } else {
                                "modify"
                            };

                            if path.is_file() || is_remove {
                                let content = if is_remove {
                                    String::from("<DELETED>")
                                } else {
                                    fs::read_to_string(&path).unwrap_or_default()
                                };

                                let timestamp = Utc::now().to_rfc3339().replace(":", "-");
                                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                                
                                let snapshot = Snapshot {
                                    timestamp: timestamp.clone(),
                                    path: path_str.to_string(),
                                    content,
                                    event_type: event_type.to_string(),
                                };
                                
                                let out_path = history_dir.join(format!("{}_{}_{}.json", timestamp, event_type, file_name));
                                let json = serde_json::to_string(&snapshot).unwrap_or_default();
                                let _ = fs::write(out_path, json);
                            }
                        }
                    }
                },
                Err(_) => {},
            }
        }).unwrap();

        watcher.watch(&dir, RecursiveMode::Recursive).unwrap();

        // Loop forever so the watcher is not dropped
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

pub fn start_global_daemon() {
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        match res {
            Ok(event) => {
                let is_modify = event.kind.is_modify();
                let is_create = event.kind.is_create();
                let is_remove = event.kind.is_remove();
                
                if is_modify || is_create || is_remove {
                    for path in event.paths {
                        let path_str = path.to_string_lossy();
                        if path_str.contains(".chronx") || path_str.contains(".git") || path_str.contains("target") || path_str.contains("node_modules") {
                            continue;
                        }

                        let event_type = if is_create {
                            "create"
                        } else if is_remove {
                            "remove"
                        } else {
                            "modify"
                        };

                        if path.is_file() || is_remove {
                            let mut current_dir = path.parent();
                            while let Some(dir) = current_dir {
                                let chronx_dir = dir.join(".chronx");
                                if chronx_dir.exists() {
                                    let history_dir = chronx_dir.join("history");
                                    
                                    let content = if is_remove {
                                        String::from("<DELETED>")
                                    } else {
                                        fs::read_to_string(&path).unwrap_or_default()
                                    };

                                    let timestamp = Utc::now().to_rfc3339().replace(":", "-");
                                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                                    
                                    let snapshot = Snapshot {
                                        timestamp: timestamp.clone(),
                                        path: path_str.to_string(),
                                        content,
                                        event_type: event_type.to_string(),
                                    };
                                    
                                    let out_path = history_dir.join(format!("{}_{}_{}.json", timestamp, event_type, file_name));
                                    let json = serde_json::to_string(&snapshot).unwrap_or_default();
                                    let _ = fs::write(out_path, json);
                                    
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = Snapshot {
            timestamp: "2026-07-29T10:00:00".to_string(),
            path: "/fake/path/file.txt".to_string(),
            content: "Hello chronx!".to_string(),
            event_type: "modify".to_string(),
        };

        let serialized = serde_json::to_string(&snapshot).unwrap();
        assert!(serialized.contains("Hello chronx!"));
        assert!(serialized.contains("modify"));
        
        let deserialized: Snapshot = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.timestamp, "2026-07-29T10:00:00");
        assert_eq!(deserialized.path, "/fake/path/file.txt");
        assert_eq!(deserialized.content, "Hello chronx!");
        assert_eq!(deserialized.event_type, "modify");
    }

    #[test]
    fn test_snapshot_deletion_state() {
        let snapshot = Snapshot {
            timestamp: "test".to_string(),
            path: "test.txt".to_string(),
            content: "<DELETED>".to_string(),
            event_type: "remove".to_string(),
        };
        assert_eq!(snapshot.content, "<DELETED>");
        assert_eq!(snapshot.event_type, "remove");
    }
}
