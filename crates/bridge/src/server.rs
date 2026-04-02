//! Bridge server

pub struct BridgeServer;

impl BridgeServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        println!("Bridge server not yet implemented");
        Ok(())
    }
}

impl Default for BridgeServer {
    fn default() -> Self {
        Self::new()
    }
}
