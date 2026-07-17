use crate::core::file_scanner::FileScanner;
use crate::models::file_info::FileInfo;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;

/// One update from a background scan thread, consumed by the UI event loop.
pub enum ScanEvent {
    /// One depth-1 entry finished: (size_bytes, is_dir) — bumps the live
    /// "files/dirs/bytes scanned" counters in ScanStatus.
    Entry(u64, bool),
    /// Scan finished; final sorted depth-1 breakdown.
    Done(Vec<FileInfo>),
}

/// A running (or just-finished) background scan. Dropping/replacing this
/// sets `cancel`, so a stale scan's remaining work winds down without the UI
/// waiting for it — the thread itself is detached and joins on its own time.
pub struct ScanHandle {
    pub rx: Receiver<ScanEvent>,
    cancel: Arc<AtomicBool>,
}

impl ScanHandle {
    /// Poll for the next available event without blocking the UI thread.
    pub fn try_recv(&self) -> Option<ScanEvent> {
        self.rx.try_recv().ok()
    }
}

impl Drop for ScanHandle {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

/// Start a depth-1 scan of `root` on a background thread. Returns immediately;
/// progress and the final result arrive via `ScanHandle::try_recv`.
pub fn spawn_scan(root: &Path) -> ScanHandle {
    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_thread = cancel.clone();
    let root = root.to_path_buf();

    std::thread::spawn(move || {
        let tx_entry = tx.clone();
        let items = FileScanner::scan_depth1(&root, &cancel_thread, |_name, size, is_dir| {
            tx_entry.send(ScanEvent::Entry(size, is_dir)).ok();
        });
        if !cancel_thread.load(Ordering::Relaxed) {
            tx.send(ScanEvent::Done(items)).ok();
        }
    });

    ScanHandle { rx, cancel }
}

/// One-shot preview scan: depth-1 breakdown of `root`, capped to the top `n`
/// entries by size. Blocking — only used for the momentary Preview overlay,
/// not the live browser screen.
pub fn preview_top_n(root: &Path, n: usize) -> Vec<FileInfo> {
    let cancel = AtomicBool::new(false);
    let mut items = FileScanner::scan_depth1(root, &cancel, |_, _, _| {});
    items.truncate(n);
    items
}
