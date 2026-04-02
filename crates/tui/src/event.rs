//! Event handling

#[derive(Debug, Clone)]
pub enum Event {
    Tick,
    Key(crossterm::event::KeyEvent),
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
