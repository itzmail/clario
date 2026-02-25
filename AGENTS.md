# AGENTS.md - AI Development Guide for Clario

## 🎯 **Project Context**

**Clario** adalah aplikasi TUI (Terminal User Interface) system cleaning utility yang ditulis dalam Rust, mirip seperti CleanMyMac tetapi berbasis terminal. Project ini digunakan sebagai learning project untuk developer yang ingin belajar Rust dengan background Go/Java/JavaScript.

### **Target User Profile**
- **Developer**: Ismail Alam
- **Background**: Berpengalaman dengan Go, Java, JavaScript
- **Learning Goal**: Menguasai Rust programming melalui hands-on project
- **Approach**: Guided implementation dengan penjelasan detail setiap Rust concept

### **Project Vision**
- Smart file scanning dan cleanup (cache, temp, logs, duplicates)
- Complete application uninstallation dengan related files  
- TUI file manager dengan preview dan expand/collapse
- Cross-platform support (prioritas: macOS → Linux → Windows)

---

## 📂 **Project Architecture & Status**

### **Planned Structure** (dari PLAN.md)
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
├── PLAN.md
└── AGENTS.md (this file)
```

### **Current Implementation Status**

**✅ COMPLETED:**
- Basic project structure initialized
- Comprehensive project plan in PLAN.md
- AGENTS.md untuk AI guidance

**🔄 IN PROGRESS:**
- Setup Cargo.toml dengan dependencies
- Todo list management system active

**❌ PENDING:**
- Module structure creation  
- Basic TUI implementation
- Core business logic
- File system operations
- UI components

---

## 📋 **Recent Updates: January 2026**

### **Enhancements to Input Handling in `app.rs`**

#### **Output: Application Fails with Initialization Error**
Error observed: `Application error: Failed to initialize input reader` due to missing raw mode setup for terminal input handling.

#### **Fix Details**
To fix the issue, adjustments were made to the terminal initialization sequence in `App::run` as follows:
1. Enabled raw mode before entering the alternate screen.
2. Disabled raw mode post-exit to restore system integrity.

**Updated Code**:
```rust
pub async fn run(&mut self) -> anyhow::Result<()> {
    let mut stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?; // Enable raw mode
    execute!(stdout, EnterAlternateScreen)?; // Set up alternate screen
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    use crossterm::event::{poll, read, Event, KeyCode};
    use std::time::Duration;

    let result = (|| -> anyhow::Result<()> {
        while !self.should_quit {
            terminal.draw(|f| {
                match self.mode {
                    AppMode::Dashboard => draw_dashboard::<CrosstermBackend<std::io::Stdout>>(f),
                    _ => {}
                }
            })?;

            if poll(Duration::from_millis(100))? {
                if let Event::Key(key) = read()? {
                    match key.code {
                        KeyCode::Char('q') => self.should_quit = true,
                        KeyCode::Char('f') => self.mode = AppMode::FileManager,
                        KeyCode::Char('s') => self.mode = AppMode::Settings,
                        KeyCode::Char('d') => self.mode = AppMode::Dashboard,
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    })();

    crossterm::terminal::disable_raw_mode()?; // Disable raw mode
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?; // Restore terminal
    result
}
```

#### **Explanation**
- **Raw Mode Management**  
  `enable_raw_mode` ensures input buffering is disabled, allowing real-time input handling.
- **Alternate Screen**  
  `EnterAlternateScreen` provides a clear, isolated TUI environment.
- **Event Polling**  
  Used `poll` with a timeout and processed events effectively, transitioning app modes based on user input (`q` to quit, `f/s/d` for switching modes).

#### Next Steps
- Re-run application (`cargo run`) to validate the fix.
- Extend rendering functions for `FileManager` and `Settings` modes in their respective files (`src/ui/file_manager.rs`, `src/ui/settings.rs`).

---



## 📂 **Project Architecture & Status**

### **Planned Structure** (dari PLAN.md)
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
├── PLAN.md
└── AGENTS.md (this file)
```

### **Current Implementation Status**

**✅ COMPLETED:**
- Basic project structure initialized
- Comprehensive project plan in PLAN.md
- AGENTS.md untuk AI guidance

**🔄 IN PROGRESS:**
- Setup Cargo.toml dengan dependencies
- Todo list management system active

**❌ PENDING:**
- Module structure creation  
- Basic TUI implementation
- Core business logic
- File system operations
- UI components

### **Dependencies Required** (sesuai PLAN.md)
```toml
[dependencies]
ratatui = "0.26"           # TUI framework
crossterm = "0.27"         # Terminal handling
tokio = { version = "1.0", features = ["full"] }  # Async runtime
walkdir = "2.4"            # Directory traversal
serde = { version = "1.0", features = ["derive"] }  # Serialization
anyhow = "1.0"             # Error handling
chrono = "0.4"             # Date/time handling
uuid = "1.0"               # File identification
```

---

## 🎓 **Learning Strategy**

### **Teaching Approach**
- **Detailed explanations** untuk setiap Rust concept baru
- **Analogi dengan Go/Java/JS** untuk familiar concepts
- **Guided implementation** - AI implements sambil explain step-by-step
- **Sequential learning** - satu concept selesai baru lanjut ke next

### **Rust Concepts Priority**
1. **Foundation**: Ownership, borrowing, lifetimes, Result/Option
2. **Syntax**: Pattern matching, traits, impl blocks, modules  
3. **Advanced**: Async/await, generics, macros, error handling
4. **Ecosystem**: Cargo, crates, testing, documentation

### **Comparison Framework**
Selalu relate Rust concepts ke bahasa yang familiar:

| Rust Concept | Go Equivalent | Java Equivalent | JavaScript Equivalent |
|--------------|---------------|-----------------|----------------------|
| `Result<T,E>` | `error` return | `Optional/Exceptions` | `Promise.catch()` |
| `Option<T>` | `nil` pointer | `Optional<T>` | `undefined/null` |
| `trait` | `interface` | `interface` | `duck typing` |
| `impl` | method receiver | `class methods` | `prototype methods` |
| `mod` | `package` | `package` | `module/import` |
| Ownership | GC | GC | GC |

---

## 📋 **Current Todo List**

### **Phase 1: Foundation Setup**
1. **✅ COMPLETED**: Setup Cargo.toml dengan dependencies
2. **✅ COMPLETED**: Create module structure sesuai architecture
3. **✅ COMPLETED**: Basic App struct untuk state management
4. **✅ COMPLETED**: Simple TUI event loop dengan ratatui

### **Phase 2: Core Implementation** 
5. **✅ COMPLETED**: Dashboard layout implementation
6. **✅ COMPLETED**: Keyboard navigation system
7. **🔄 IN_PROGRESS**: File scanning basic functionality

### **Phase 3: Advanced Features**
8. **⏳ PENDING**: Application detection logic
9. **⏳ PENDING**: Safety checking mechanisms  
10. **⏳ PENDING**: Cross-platform compatibility

---

## 🤖 **AI Agent Instructions**

### **Communication Style**
- **Bahasa Indonesia** untuk communication utama
- **English** untuk code comments dan technical terms
- **Emoji** hanya jika user request explicitly
- **Concise but comprehensive** explanations

### **Development Approach**
1. **Always use TodoWrite tool** untuk track progress
2. **Mark todos in_progress** saat mulai kerja
3. **Mark completed immediately** setelah selesai task
4. **One task in_progress** at a time
5. **Explain concepts** sebelum/saat implement code

### **Code Quality Standards**
- **Follow Rust best practices** (clippy, rustfmt)
- **Comprehensive error handling** dengan anyhow
- **Clear documentation** untuk public APIs
- **Modular design** sesuai planned architecture
- **Safety first** - prefer safe operations over performance

### **Learning Integration**
- **Explain WHY** not just HOW untuk setiap implementation
- **Show alternatives** dan explain trade-offs
- **Relate to familiar concepts** dari Go/Java/JS
- **Progressive complexity** - start simple, build up
- **Practice opportunities** - suggest exercises untuk reinforce learning

### **File Management**
- **Read files first** sebelum edit (required by tools)
- **Prefer editing** over creating new files
- **Follow planned structure** dari PLAN.md
- **Consistent naming** dan organization

### **Testing Strategy**
- **Cargo check** untuk verify compilation
- **Basic functionality tests** saat implement
- **Integration tests** untuk core features
- **Manual testing** untuk TUI interactions

---

## 🔍 **Key Implementation Checkpoints**

### **Milestone 1: Basic TUI (Week 1-2)**
- [ ] Dependencies setup complete
- [ ] Module structure created
- [ ] Basic ratatui app runs
- [ ] Simple dashboard layout
- [ ] Keyboard input handling

### **Milestone 2: Core Logic (Week 3-4)**  
- [ ] File system scanner working
- [ ] Basic safety checks implemented
- [ ] Data models defined
- [ ] Error handling consistent

### **Milestone 3: Features (Week 5-6)**
- [ ] Application detection logic
- [ ] Multi-select operations
- [ ] File preview functionality
- [ ] Cross-platform compatibility

### **Milestone 4: Polish (Week 7-8)**
- [ ] Performance optimization
- [ ] Complete error handling
- [ ] Documentation
- [ ] Testing coverage

---

## 💡 **Important Notes for AI Agents**

### **User Context**
- **Learning-focused**: Prioritize explanation over speed
- **Hands-on approach**: User wants to understand each step
- **Background knowledge**: Strong in Go/Java/JS, new to Rust
- **Goal**: Build practical skills through real project

### **Technical Preferences**  
- **Rust 2021 edition** untuk stability
- **macOS development** environment
- **Terminal-based** workflow preferred
- **Git integration** untuk version control

### **Common Pitfalls to Avoid**
- Don't assume Rust knowledge - explain ownership/borrowing
- Don't skip error handling explanations  
- Don't create files without reading first
- Don't batch todo completions - mark immediately
- Don't use echo/printf for communication - output directly

### **Success Metrics**
- User understands each Rust concept introduced
- Code compiles and runs correctly
- Architecture follows planned structure  
- Progress tracked via todo system
- Learning objectives achieved incrementally

---

## 🚀 **Quick Start for New AI Agent**

1. **Read current status**: Check todo list dengan `todoread`
2. **Understand context**: Review PLAN.md dan current codebase
3. **Ask clarifying questions** jika ada uncertainty
4. **Start with current in_progress task** atau pick next high priority
5. **Explain concepts** sambil implement
6. **Track progress** dengan todo updates
7. **Test hasil** dengan cargo check/run

---

**Last Updated**: January 2026
**Project Phase**: Foundation Setup (Phase 1)
**Next Priority**: Complete Cargo.toml setup dan create module structure