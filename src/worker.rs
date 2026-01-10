//! Background worker for CPU-intensive tasks

use crate::config::Config;
use crate::files::{extract_symbols, find_all_files_recursive, Symbol};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

pub enum WorkerRequest {
    IndexSymbols {
        root_dir: Box<Path>,
        config: Box<Config>,
    },
    Shutdown,
}

pub enum WorkerResponse {
    SymbolsIndexed(Vec<Symbol>),
    IndexingProgress { processed: usize, total: usize },
}

pub struct BackgroundWorker {
    request_tx: Sender<WorkerRequest>,
    response_rx: Receiver<WorkerResponse>,
    _handle: JoinHandle<()>,
}

impl BackgroundWorker {
    pub fn spawn() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<WorkerRequest>();
        let (response_tx, response_rx) = mpsc::channel::<WorkerResponse>();

        let handle = thread::spawn(move || {
            worker_loop(&request_rx, &response_tx);
        });

        Self {
            request_tx,
            response_rx,
            _handle: handle,
        }
    }

    pub fn request_symbol_indexing(&self, root_dir: &Path, config: Config) {
        let _ = self.request_tx.send(WorkerRequest::IndexSymbols {
            root_dir: root_dir.into(),
            config: Box::new(config),
        });
    }

    pub fn poll_response(&self) -> Option<WorkerResponse> {
        self.response_rx.try_recv().ok()
    }

    pub fn shutdown(&self) {
        let _ = self.request_tx.send(WorkerRequest::Shutdown);
    }
}

impl Drop for BackgroundWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn worker_loop(request_rx: &Receiver<WorkerRequest>, response_tx: &Sender<WorkerResponse>) {
    while let Ok(request) = request_rx.recv() {
        match request {
            WorkerRequest::IndexSymbols { root_dir, config } => {
                index_symbols(&root_dir, &config, response_tx);
            }
            WorkerRequest::Shutdown => break,
        }
    }
}

fn index_symbols(root_dir: &Path, config: &Config, response_tx: &Sender<WorkerResponse>) {
    let Ok(all_files) = find_all_files_recursive(root_dir, config) else {
        let _ = response_tx.send(WorkerResponse::SymbolsIndexed(Vec::new()));
        return;
    };

    let source_files: Vec<_> = all_files.into_iter().filter(|f| f.is_file).collect();
    let total = source_files.len();
    let mut symbols = Vec::new();

    for (idx, file_entry) in source_files.iter().enumerate() {
        if let Ok(content) = std::fs::read_to_string(&file_entry.path) {
            let file_symbols = extract_symbols(&file_entry.path, &content);
            symbols.extend(file_symbols);
        }

        if idx % 50 == 0 || idx == total - 1 {
            let _ = response_tx.send(WorkerResponse::IndexingProgress {
                processed: idx + 1,
                total,
            });
        }
    }

    let _ = response_tx.send(WorkerResponse::SymbolsIndexed(symbols));
}
