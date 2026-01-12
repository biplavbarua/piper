
use walkdir::{WalkDir, DirEntry};
use std::path::{Path, PathBuf};
use rayon::prelude::*;

pub struct Spyder {
    root: PathBuf,
}

impl Spyder {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    /// The "Middle-Out" Traversal Algorithm.
    /// Unlike standard depth-first or breadth-first searches which waste cycles on the edges,
    /// Spyder calculates the heuristic "centroid" of the file system and traverses outwards.
    /// (Actually, currently it just parallelizes the walk, which is efficiently "middle-out" in thread utility).
    pub fn crawl(&self) -> Vec<PathBuf> {
        let entries: Vec<DirEntry> = WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();

        // Silicon Valley "Middle-Out" Simulation: 
        // We process the list from the middle outwards for maximum efficiency?
        // In reality, we just parallel iterate which is faster.
        let mut paths: Vec<PathBuf> = entries.par_iter()
            .map(|e| e.path().to_path_buf())
            .collect();
            
        // Sorting by path length to simulate processing "dense" (middle) content first?
        // Just keeping it simple for now.
        paths
    }
}
