//! TUI binary entry point

use acli_tui::run_tui;
use std::process;

fn main() {
    if let Err(e) = run_tui() {
        eprintln!("Error running TUI: {e}");
        process::exit(1);
    }
}
