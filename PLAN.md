# Clario - System Cleaning Utility

## 🎯 **Project Overview**

Clario adalah aplikasi clean my mac-like yang berbasis TUI dan dibuat menggunakan Rust. Aplikasi ini menggabungkan konsep clean/clarity dengan interface yang modern dan fun.

### **Target Features**
- 🧹 Smart file scanning dan cleanup (cache, temp, logs, duplicates)
- 🗑️ Complete application uninstallation dengan related files
- 📁 TUI file manager dengan preview dan expand/collapse
- ⚡ Cross-platform support (macOS, Linux, Windows)

---

## 🏗️ **Architecture & Design**

### **Project Structure**
```
clario/
├── src/
│   ├── main.rs                 # Entry point
│   ├── app.rs                  # Application state & main loop
│   ├── ui/                     # TUI components
│   │   ├── mod.rs
│   │   ├── dashboard.rs        # Main dashboard
│   │   ├── file_manager.rs     # File browser widget
│   │   ├── app_list.rs         # Application list
│   │   └── preview.rs          # File preview panel
│   ├── core/                   # Business logic
│   │   ├── mod.rs
│   │   ├── file_scanner.rs     # File system scanner
│   │   ├── app_detector.rs     # Application detector
│   │   ├── safety_checker.rs   # Safe deletion logic
│   │   └── uninstaller.rs      # App uninstaller
│   ├── models/                 # Data structures
│   │   ├── mod.rs
│   │   ├── file_info.rs
│   │   ├── app_info.rs
│   │   └── scan_result.rs
│   └── utils/                  # Utilities
│       ├── mod.rs
│       ├── platform.rs         # Platform-specific code
│       └── file_utils.rs
├── Cargo.toml
├── README.md
└── PLAN.md
```

### **Dependencies**
```toml
[dependencies]
ratatui = "0.26"           # TUI framework
crossterm = "0.27"         # Terminal handling
tokio = { version = "1.0", features = ["full"] }  # Async runtime
walkdir = "2.4"            # Directory traversal
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"             # Error handling
chrono = "0.4"             # Date/time handling
uuid = "1.0"               # File identification
```

---

## 🎨 **UI Design: Dashboard-First Approach**

### **Main Dashboard Layout**
```
┌─────────────────────────────────────────────────────────────┐
│ 🧹 Clario v1.0                                              │
├─────────────────────────────────────────────────────────────┤
│ 💻 System: MacBook Pro M2 🟢  Storage: 156.3GB/256GB (61%)  │
│ 📊 Last Clean: 2 days ago  📁 Files Deleted: 142           │
│ 🗑️ Space Freed: 8.1GB  ⚡ Performance Score: 85/100        │
│                                                             │
│ 📈 Current Issues:                                          │
│ ⚠️  Browser cache > 1GB | ⚠️ 3 apps unused > 3 months      │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Quick Actions: [c] Clean Cache [u] Uninstall Unused    │ │
│ └─────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│ 📋 Main Menu                                               │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 🧹 1. Clean Files                                        │ │
│ │     Smart cleanup - 2.1GB ready to delete               │ │
│ │                                                         │ │
│ │ 🗑️ 2. Uninstall Applications                             │ │
│ │     15 apps, 3 unused, 524MB total size                 │ │
│ │                                                         │ │
│ │ ⚙️ 3. Settings                                          │ │
│ │     Configure cleanup rules and safety options          │ │
│ │                                                         │ │
│ │ 📊 4. Statistics                                         │ │
│ │     View cleanup history and system trends              │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ [1-4] Select Menu | [q] Quit | [?] Help | [↑↓] Navigate   │
└─────────────────────────────────────────────────────────────┘
```

### **User Flow**
```
Main Dashboard → Select Action → Detail Page → Execution
```

---

## 🚀 **Core Features Implementation**

### **1. Smart File Scanner**
- Scan direktori sistem untuk file tidak berguna
- Kategori: cache, temp, logs, duplicates, large files
- Metadata: size, modified date, author, file type
- AI-powered safety scoring untuk menentukan aman dihapus

### **2. TUI File Manager**
- Layout 3-panel: folder tree, file list, file preview
- Navigasi keyboard (vim-like): hjkl, Enter, Backspace
- Expand/collapse dengan `l` atau `→`
- Sort by name, size, date modified
- Multi-select dengan Space

### **3. Application Uninstaller**
- Deteksi aplikasi macOS di `/Applications` dan `~/Applications`
- Scan related files di:
  - `~/Library/Application Support/`
  - `~/Library/Preferences/` (plist files)
  - `~/Library/Caches/`
  - `~/Library/Logs/`
  - `~/Library/Containers/`
- Safe uninstall dengan konfirmasi

### **4. Safety Features**
- Whitelist file penting (system files)
- Backup sebelum delete (opsional)
- Preview mode sebelum eksekusi
- Undo functionality (trash system)

---

## ⚡ **Implementation Strategy**

### **Phase 1: Foundation (Week 1-2)**
- [x] Setup project structure dan dependencies
- [ ] Basic TUI dengan ratatui
- [ ] Dashboard layout implementation
- [ ] Basic navigation system
- [ ] File system scanner dasar

### **Phase 2: Core Features (Week 3-4)**
- [ ] Smart file detection dan categorization
- [ ] Safety checker logic
- [ ] Multi-select dan batch operations
- [ ] File preview functionality
- [ ] Menu system dengan dynamic items

### **Phase 3: App Management (Week 5-6)**
- [ ] Application detector untuk macOS
- [ ] Related files scanner
- [ ] Safe uninstaller
- [ ] Cross-platform compatibility layer

### **Phase 4: Polish & Optimization (Week 7-8)**
- [ ] Performance optimization
- [ ] Error handling & recovery
- [ ] Configuration system
- [ ] Documentation dan testing
- [ ] Package dan distribution

---

## 🔒 **Safety Considerations**

- **Read-only mode by default** - Explicit confirmation untuk delete operations
- **System file protection** - Whitelist untuk file kritik sistem
- **Audit trail** - Log semua operasi untuk transparency
- **Rollback capability** - Undo functionality untuk safety
- **Multi-layer confirmation** - Untuk destructive actions

---

## 🎯 **Success Metrics**

- **Performance**: Scan performance <5s untuk 10GB data
- **Memory**: Memory usage <50MB untuk large directories
- **Safety**: Safety accuracy >99% untuk file identification
- **UX**: Intuitive keyboard navigation, <3 steps untuk common tasks
- **Compatibility**: Works seamlessly pada macOS, Linux, Windows

---

## 📝 **Next Steps**

1. **Setup development environment** dengan semua dependencies
2. **Implement basic TUI structure** dengan ratatui
3. **Create dashboard layout** dengan summary dan menu system
4. **Build file scanner foundation** untuk basic functionality
5. **Add navigation dan interaction** untuk user experience

---

## 🎨 **Branding Notes**

**Nama**: Clario
**Konsep**: Clean + Clarity 
**Vibe**: Modern, professional, fun
**Target**: Power users yang ingin system optimization tool yang reliable dan user-friendly