//! Event handling for the TUI application

use crossterm::event::{self, Event, KeyEvent};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Handles terminal events and forwards them to the application
pub struct EventHandler {
    /// How often to check for events
    tick_rate: Duration,
    /// Last time we processed a tick
    last_tick: Instant,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new() -> Self {
        Self {
            tick_rate: Duration::from_millis(250),
            last_tick: Instant::now(),
        }
    }

    /// Start the event loop that reads terminal events
    pub async fn start_event_loop(&mut self, tx: mpsc::UnboundedSender<Event>) {
        loop {
            let timeout = self
                .tick_rate
                .checked_sub(self.last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            // Check for crossterm events
            if event::poll(timeout).unwrap_or(false) {
                match event::read() {
                    Ok(event) => {
                        if tx.send(event).is_err() {
                            // Channel closed, exit loop
                            break;
                        }
                    }
                    Err(_) => {
                        // Error reading event, continue
                        continue;
                    }
                }
            }

            // Send tick event if enough time has passed
            if self.last_tick.elapsed() >= self.tick_rate {
                if tx.send(Event::Key(KeyEvent::from(event::KeyCode::Null))).is_err() {
                    break;
                }
                self.last_tick = Instant::now();
            }
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
