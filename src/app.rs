use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use std::path::PathBuf;
use ratatui::widgets::ListState;
use crossterm::event::KeyCode;

use crate::scanner;
use crate::compressor;

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
}

pub enum AppMessage {
    ScanComplete(Vec<FileItem>),
}

pub struct App {
    pub items: Vec<FileItem>,
    pub list_state: ListState,
    pub weissman_score: f64,
    pub total_savings: u64,
    pub is_scanning: bool,
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
            KeyCode::Char('d') => self.delete_item(),
            _ => {}
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
        if self.is_scanning {
            self.spinner_state = (self.spinner_state + 1) % 4;
            
            // Check for results
            if let Some(rx) = &self.rx {
                if let Ok(msg) = rx.try_recv() {
                    match msg {
                        AppMessage::ScanComplete(items) => {
                            self.items = items;
                            self.is_scanning = false;
                            self.rx = None;
                            if !self.items.is_empty() {
                                self.list_state.select(Some(0));
                            }
                        }
                    }
                }
            }
        }
    }

    fn start_scan(&mut self) {
        if self.is_scanning { return; }
        self.is_scanning = true;
        self.items.clear(); 

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
        let mut savings = 0;
        for i in 0..self.items.len() {
             if self.items[i].status == FileStatus::Found {
                 self.items[i].status = FileStatus::Compressing;
                 let path = PathBuf::from(&self.items[i].path);
                 match compressor::compress_file(&path) {
                     Ok(stats) => {
                         self.items[i].status = FileStatus::Done;
                         self.items[i].compressed_size = Some(stats.compressed_size);
                         if stats.original_size > stats.compressed_size {
                             savings += stats.original_size - stats.compressed_size;
                         }
                     },
                     Err(_) => {
                         self.items[i].status = FileStatus::Error;
                     }
                 }
             }
        }
        self.total_savings += savings;
        
        // Weissman Score Formula
        let total_original = self.items.iter().map(|i| i.original_size).sum::<u64>() as f64;
        let total_compressed = self.items.iter().map(|i| i.compressed_size.unwrap_or(i.original_size)).sum::<u64>() as f64;
        
        if total_compressed > 0.0 {
            let ratio = total_original / total_compressed;
            self.weissman_score = ratio * 2.6; 
        } else {
            self.weissman_score = 0.0;
        }
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
                            self.total_savings += self.items[i].original_size; 
                        }
                        Err(_) => {
                            self.items[i].status = FileStatus::Error;
                        }
                    }
                }
            }
        }
    }
}
