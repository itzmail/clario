use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Run `work` on a background thread while animating a spinner after `label` on the
/// current line. The spinner is cleared (not left as a stray frame) before the result
/// line prints, so the caller can `print!` its own outcome text right after this returns.
///
/// ponytail: plain OS thread + atomic flag, no async runtime hookup needed for a
/// one-shot blocking scan; upgrade to a shared progress-bar crate if multiple
/// concurrent spinners are ever needed at once.
pub fn spin<T: Send + 'static>(label: &str, work: impl FnOnce() -> T + Send + 'static) -> T {
    let is_tty = atty_stdout();
    print!("  → {}... ", label);
    io::stdout().flush().ok();

    if !is_tty {
        // Non-interactive output (piped/redirected): animation would just spam lines.
        return work();
    }

    let done = Arc::new(AtomicBool::new(false));
    let done_writer = done.clone();
    let label_owned = label.to_string();

    let handle = thread::spawn(move || {
        let mut i = 0usize;
        while !done_writer.load(Ordering::Relaxed) {
            print!("\r  {} {}...  ", FRAMES[i % FRAMES.len()], label_owned);
            io::stdout().flush().ok();
            i += 1;
            thread::sleep(Duration::from_millis(80));
        }
    });

    let result = work();
    done.store(true, Ordering::Relaxed);
    handle.join().ok();

    // Erase the spinner line, leave the cursor ready for the caller's result text.
    print!("\r  → {}... ", label);
    io::stdout().flush().ok();

    result
}

fn atty_stdout() -> bool {
    use std::io::IsTerminal;
    io::stdout().is_terminal()
}
