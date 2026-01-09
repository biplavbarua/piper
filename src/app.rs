use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use std::path::PathBuf;
use ratatui::widgets::ListState;
use crossterm::event::KeyCode;
use rayon::prelude::*;

use crate::scanner;
use crate::compressor::{self, CompressionStats};

pub struct FileItem {
    pub path: String,
    pub original_size: u64,
    pub compressed_size: Option<u64>,
    pub status: FileStatus,
}

#[derive(PartialEq)]
pub enum FileStatus {
    Found,
    Compressing,
    Done,
    Error,
    Deleted,
    Restored,
}

pub enum AppMessage {
    ScanComplete(Vec<FileItem>),
    CompressionProgress(usize, Result<CompressionStats, String>),
    CompressionDone,
    RestorationDone(usize, bool), // index, success
}

pub struct App {
    pub items: Vec<FileItem>,
    pub list_state: ListState,
    pub weissman_score: f64,
    pub total_savings: u64,
    pub is_scanning: bool,
    pub is_compressing: bool,
    pub is_restoring: bool,
    pub show_details: bool,
    pub spinner_state: u8,
    pub rx: Option<Receiver<AppMessage>>,
}

impl App {
    pub fn new() -> App {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        App {
            items: Vec::new(),
            list_state,
            weissman_score: 5.2,
            total_savings: 0,
            is_scanning: false,
            is_compressing: false,
            is_restoring: false,
            show_details: false,
            spinner_state: 0,
            rx: None,
        }
    }

    pub fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Down | KeyCode::Char('j') => self.next(),
            KeyCode::Up | KeyCode::Char('k') => self.previous(),
            KeyCode::Char('s') => self.start_scan(),
            KeyCode::Char('c') => self.start_compression(),
            // Safety: Block operations during active work
            KeyCode::Char('d') if !self.is_compressing && !self.is_restoring => self.delete_item(),
            KeyCode::Char('e') if !self.is_compressing && !self.is_restoring => self.restore_item(),
            KeyCode::Enter => self.toggle_details(),
            _ => {}
        }
    }

    pub fn toggle_details(&mut self) {
        if !self.items.is_empty() {
             self.show_details = !self.show_details;
        }
    }

    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.items.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn tick(&mut self) {
        if self.is_scanning || self.is_compressing {
            self.spinner_state = (self.spinner_state + 1) % 4;
            
            // Check for results
            let mut messages = Vec::new();
            if let Some(rx) = &self.rx {
                while let Ok(msg) = rx.try_recv() {
                    messages.push(msg);
                }
            }

            for msg in messages {
                match msg {
                    AppMessage::ScanComplete(items) => {
                        self.items = items;
                        self.is_scanning = false;
                        self.rx = None;
                        if !self.items.is_empty() {
                            self.list_state.select(Some(0));
                        }
                    }
                    AppMessage::CompressionProgress(idx, result) => {
                        if idx < self.items.len() {
                            match result {
                                Ok(stats) => {
                                    self.items[idx].status = FileStatus::Done;
                                    self.items[idx].compressed_size = Some(stats.compressed_size);
                                    if stats.original_size > stats.compressed_size {
                                        self.total_savings += stats.original_size - stats.compressed_size;
                                    }
                                },
                                Err(_) => {
                                    self.items[idx].status = FileStatus::Error;
                                }
                            }
                            self.calculate_score();
                        }
                    }
                    AppMessage::CompressionDone => {
                        self.is_compressing = false;
                        self.rx = None;
                    }
                    AppMessage::RestorationDone(idx, success) => {
                        if idx < self.items.len() && success {
                            self.items[idx].status = FileStatus::Restored;
                            // Revert Stats
                            if let Some(compressed) = self.items[idx].compressed_size {
                                if self.items[idx].original_size > compressed {
                                    self.total_savings = self.total_savings.saturating_sub(self.items[idx].original_size - compressed);
                                }
                            }
                            self.items[idx].compressed_size = None;
                            self.calculate_score();
                        } else if idx < self.items.len() {
                             self.items[idx].status = FileStatus::Error;
                        }
                        self.is_restoring = false;
                        self.rx = None;
                    }
                }
            }
        }
    }

    fn calculate_score(&mut self) {
        let total_original = self.items.iter().map(|i| i.original_size).sum::<u64>() as f64;
        let total_compressed = self.items.iter().map(|i| i.compressed_size.unwrap_or(i.original_size)).sum::<u64>() as f64;
        
        if total_compressed > 0.0 {
            let ratio = total_original / total_compressed;
            self.weissman_score = ratio * 2.6; 
        } else {
            self.weissman_score = 0.0;
        }
    }

    fn start_scan(&mut self) {
        if self.is_scanning || self.is_compressing { return; }
        self.is_scanning = true;
        self.items.clear(); 
        self.weissman_score = 0.0;
        self.total_savings = 0;

        let (tx, rx): (Sender<AppMessage>, Receiver<AppMessage>) = mpsc::channel();
        self.rx = Some(rx);

        thread::spawn(move || {
            let mut results = Vec::new();
            if let Some(mut dev_dir) = dirs::home_dir() {
                dev_dir.push("Developer");
                let scan_res = scanner::scan_logs(&dev_dir);
                
                for res in scan_res {
                    results.push(FileItem {
                        path: res.path.to_string_lossy().to_string(),
                        original_size: res.size,
                        compressed_size: None,
                        status: FileStatus::Found,
                    });
                }
            }
            let _ = tx.send(AppMessage::ScanComplete(results));
        });
    }

    fn start_compression(&mut self) {
        if self.is_scanning || self.is_compressing { return; }
        self.is_compressing = true;

        let (tx, rx): (Sender<AppMessage>, Receiver<AppMessage>) = mpsc::channel();
        self.rx = Some(rx);

        // Collect files to compress (indices and paths) to pass to thread
        // We verify status is Found to avoid re-compressing
        let targets: Vec<(usize, PathBuf)> = self.items.iter().enumerate()
            .filter(|(_, item)| item.status == FileStatus::Found)
            .map(|(i, item)| (i, PathBuf::from(&item.path)))
            .collect();

        // Mark them as compressing in UI immediately
        for (i, _) in &targets {
            self.items[*i].status = FileStatus::Compressing;
        }

        thread::spawn(move || {
            // Parallel Compression using Rayon
            targets.into_par_iter().for_each_with(tx.clone(), |s, (idx, path)| {
                let res = compressor::compress_file(&path).map_err(|e| e.to_string());
                let _ = s.send(AppMessage::CompressionProgress(idx, res));
            });
            
            let _ = tx.send(AppMessage::CompressionDone);
        });
    }

    fn delete_item(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if i < self.items.len() {
                let path = PathBuf::from(&self.items[i].path);
                if path.exists() {
                    let res = if path.is_dir() {
                        std::fs::remove_dir_all(&path)
                    } else {
                        std::fs::remove_file(&path)
                    };

                    match res {
                        Ok(_) => {
                            self.items[i].status = FileStatus::Deleted;
                            self.calculate_score(); // Recalc score ignoring this? Or keeping it?
                            // Logic: If deleted, it counts as 100% savings? 
                            // Or just remove from score calculation?
                            // Current calc_score uses all items.
                            // If deleted, compressed size?
                            // Let's say deleted means size becomes 0.
                            self.items[i].compressed_size = Some(0);
                            self.total_savings += self.items[i].original_size;
                            self.calculate_score();
                        }
                        Err(_) => {
                            self.items[i].status = FileStatus::Error;
                        }
                    }
                }
            }
        }
    }


    fn restore_item(&mut self) {
        if self.is_scanning || self.is_compressing || self.is_restoring { return; }

        if let Some(i) = self.list_state.selected() {
            if i < self.items.len() {
                // Restoration only makes sense for Compressed (Done) items
                if self.items[i].status == FileStatus::Done {
                    self.is_restoring = true;
                    // Optimistic update
                    self.items[i].status = FileStatus::Compressing; // Reuse spinner

                    let (tx, rx): (Sender<AppMessage>, Receiver<AppMessage>) = mpsc::channel();
                    self.rx = Some(rx);

                    let path = PathBuf::from(&self.items[i].path);

                    thread::spawn(move || {
                        // Decompress
                        let zst_path = path.with_extension(format!("{}.zst", path.extension().unwrap_or_default().to_string_lossy()));
                        
                        let success = match compressor::decompress_file(&zst_path) {
                            Ok(_) => true,
                            Err(_) => false,
                        };
                        let _ = tx.send(AppMessage::RestorationDone(i, success));
                    });
                }
            }
        }
    }
}
