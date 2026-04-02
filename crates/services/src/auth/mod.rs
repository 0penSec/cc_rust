//! Authentication management

pub struct AuthManager;

impl AuthManager {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}
