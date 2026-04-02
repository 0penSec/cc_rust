use anyhow::Result;

pub struct App;

impl App {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("TUI not yet implemented. Use CLI mode for now.");
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
