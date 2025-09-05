//! Signal handling for graceful application shutdown

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::signal::unix::{signal, SignalKind};

/// Signal handler for managing application shutdown
pub struct SignalHandler {
    quit_flag: Arc<AtomicBool>,
}

impl SignalHandler {
    /// Create a new signal handler with the given quit flag
    pub fn new(quit_flag: Arc<AtomicBool>) -> Self {
        Self { quit_flag }
    }

    /// Set up SIGINT (Ctrl+C) signal handler
    pub async fn setup_signal_handler(&self) -> tokio::task::JoinHandle<()> {
        let quit_flag = Arc::clone(&self.quit_flag);

        tokio::spawn(async move {
            let mut sigint = signal(SignalKind::interrupt())
                .expect("Failed to create SIGINT signal handler");

            sigint.recv().await;
            quit_flag.store(true, Ordering::Relaxed);
        })
    }

    /// Check if quit signal has been received
    pub fn should_quit(&self) -> bool {
        self.quit_flag.load(Ordering::Relaxed)
    }
}
