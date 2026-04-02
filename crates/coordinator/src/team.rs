//! Team management

pub struct Team {
    pub name: String,
}

impl Team {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
