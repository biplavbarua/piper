use std::path::{Path, PathBuf};
use std::time::{SystemTime, Duration};
use walkdir::WalkDir;

pub struct ScanResult {
    pub path: PathBuf,
    pub size: u64,
}

pub fn scan_logs(root: &Path) -> Vec<ScanResult> {
    let mut results = Vec::new();
    let now = SystemTime::now();
    let one_day = Duration::from_secs(60 * 60 * 24);

    // Relaxed scanner for MVP
    let walker = WalkDir::new(root).into_iter();
    
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy();

        // 1. Check for Heavy Directories
        if entry.file_type().is_dir() {
            if file_name == "node_modules" || file_name == "target" || file_name == "venv" || file_name == ".venv" {
                // Calculate directory size (expensive but necessary)
                let size = get_dir_size(path);
                 results.push(ScanResult {
                    path: path.to_path_buf(),
                    size,
                });
                continue; 
            }
        }

        // 2. Check for Log Files
        if entry.file_type().is_file() {
            if let Some(ext) = path.extension() {
                if ext == "log" || ext == "txt" {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.len() > 1024 { // > 1KB
                            results.push(ScanResult {
                                path: path.to_path_buf(),
                                size: metadata.len(),
                            });
                        }
                    }
                }
            }
        }
    }
    results
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with('.') && s != ".venv") // Allow .venv
         .unwrap_or(false)
}

fn get_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}
