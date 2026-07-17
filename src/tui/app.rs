use crate::core::analyze_presets::{self, Preset};
use crate::models::file_info::FileInfo;
use crate::tui::scan::{spawn_scan, ScanEvent, ScanHandle};
use std::collections::HashSet;
use std::path::PathBuf;

/// Live counters shown while a background scan is still running.
pub struct ScanStatus {
    pub files: usize,
    pub dirs: usize,
    pub bytes: u64,
    pub done: bool,
}

impl ScanStatus {
    fn starting() -> Self {
        Self { files: 0, dirs: 0, bytes: 0, done: false }
    }
}

/// One row on the category screen: a preset location plus its (possibly
/// still-loading) recursive size.
pub struct CategoryRow {
    pub label: &'static str,
    pub path: PathBuf,
    pub size_bytes: Option<u64>,
    /// Running total while `size_bytes` is still `None` — lets the row show a
    /// live-growing size instead of a static "pending..." during its scan.
    pub running_bytes: u64,
}

pub struct CategoriesScreen {
    pub rows: Vec<CategoryRow>,
    pub selected: usize,
    /// Index of the row currently being sized, and its scan handle.
    scanning: Option<(usize, ScanHandle)>,
    next_to_scan: usize,
}

pub struct BrowserScreen {
    pub root: PathBuf,
    /// Path stack for Esc-to-go-up; empty means "back to categories".
    pub breadcrumb: Vec<PathBuf>,
    pub entries: Vec<FileInfo>,
    pub selected: usize,
    pub multi_selected: HashSet<usize>,
    pub status: ScanStatus,
    pub scan: Option<ScanHandle>,
    pub overlay: Overlay,
}

/// Modal overlays drawn on top of the browser screen.
pub enum Overlay {
    None,
    ConfirmDelete { targets: Vec<PathBuf>, total_bytes: u64, blocked: Vec<PathBuf> },
    Preview { title: String, lines: Vec<String> },
    Message(String),
}

pub enum Screen {
    Categories(CategoriesScreen),
    Browser(BrowserScreen),
}

pub struct App {
    pub screen: Screen,
    pub should_quit: bool,
}

impl App {
    /// Start on the category screen (Mole-style entry point).
    pub fn new_categories() -> Self {
        let rows = analyze_presets::presets()
            .into_iter()
            .map(|Preset { label, path }| CategoryRow { label, path, size_bytes: None, running_bytes: 0 })
            .collect();
        let mut screen = CategoriesScreen { rows, selected: 0, scanning: None, next_to_scan: 0 };
        screen.start_next_scan();
        Self { screen: Screen::Categories(screen), should_quit: false }
    }

    /// Skip straight to the browser screen for `path` (`clario analyze <path>`).
    pub fn new_browser(path: PathBuf) -> Self {
        let mut screen = BrowserScreen::new(path, Vec::new());
        screen.rescan();
        Self { screen: Screen::Browser(screen), should_quit: false }
    }

    /// Drain any pending scan events for the active screen. Called once per
    /// frame before rendering so progress counters stay current.
    pub fn poll_scans(&mut self) {
        match &mut self.screen {
            Screen::Categories(s) => s.poll(),
            Screen::Browser(s) => s.poll(),
        }
    }

    pub fn enter_browser_from_category(&mut self, idx: usize) {
        if let Screen::Categories(cats) = &self.screen {
            if let Some(row) = cats.rows.get(idx) {
                let mut screen = BrowserScreen::new(row.path.clone(), Vec::new());
                screen.rescan();
                self.screen = Screen::Browser(screen);
            }
        }
    }
}

impl CategoriesScreen {
    /// Scan rows one at a time (not all in parallel) — a dozen recursive
    /// `Library`/`Applications`-sized scans running concurrently would thrash
    /// disk I/O for no benefit; sequential keeps each row's ETA readable.
    fn start_next_scan(&mut self) {
        if self.scanning.is_some() {
            return;
        }
        if self.next_to_scan >= self.rows.len() {
            return;
        }
        let idx = self.next_to_scan;
        self.next_to_scan += 1;
        let handle = spawn_scan(&self.rows[idx].path);
        self.scanning = Some((idx, handle));
    }

    fn poll(&mut self) {
        let Some((idx, handle)) = &self.scanning else { return };
        let mut finished = false;
        while let Some(event) = handle.try_recv() {
            match event {
                ScanEvent::Entry(info) => self.rows[*idx].running_bytes += info.size_bytes,
                ScanEvent::Done(items) => {
                    let total: u64 = items.iter().map(|f| f.size_bytes).sum();
                    self.rows[*idx].size_bytes = Some(total);
                    finished = true;
                }
            }
        }
        if finished {
            self.scanning = None;
            self.start_next_scan();
        }
    }
}

impl BrowserScreen {
    fn new(root: PathBuf, breadcrumb: Vec<PathBuf>) -> Self {
        Self {
            root,
            breadcrumb,
            entries: Vec::new(),
            selected: 0,
            multi_selected: HashSet::new(),
            status: ScanStatus::starting(),
            scan: None,
            overlay: Overlay::None,
        }
    }

    pub fn rescan(&mut self) {
        self.entries.clear();
        self.selected = 0;
        self.multi_selected.clear();
        self.status = ScanStatus::starting();
        self.scan = Some(spawn_scan(&self.root)); // dropping the old handle cancels it
    }

    fn poll(&mut self) {
        let Some(scan) = &self.scan else { return };
        while let Some(event) = scan.try_recv() {
            match event {
                ScanEvent::Entry(info) => {
                    self.status.bytes += info.size_bytes;
                    if info.is_dir {
                        self.status.dirs += 1;
                    } else {
                        self.status.files += 1;
                    }
                    self.entries.push(info);
                    self.entries.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
                }
                ScanEvent::Done(items) => {
                    self.entries = items;
                    self.status.done = true;
                }
            }
        }
    }

    pub fn enter_selected(&mut self) {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                let new_root = entry.path.clone();
                let mut old_root = new_root.clone();
                std::mem::swap(&mut old_root, &mut self.root);
                self.breadcrumb.push(old_root);
                self.root = new_root;
                self.rescan();
            }
        }
    }

    /// Returns true if we went up a level; false if the caller should pop
    /// back to the category screen instead.
    pub fn go_up(&mut self) -> bool {
        match self.breadcrumb.pop() {
            Some(parent) => {
                self.root = parent;
                self.rescan();
                true
            }
            None => false,
        }
    }
}
