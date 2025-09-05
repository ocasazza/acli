//! TUI binary entry point

use atui::run_tui;
use std::process;

fn main() {
    if let Err(e) = run_tui() {
        eprintln!("Error running TUI: {e}");
        process::exit(1);
    }
}
