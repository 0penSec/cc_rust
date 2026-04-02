//! Agent swarm management

use crate::agent::Agent;
use std::collections::HashMap;

pub struct Swarm {
    agents: HashMap<String, Agent>,
}

impl Swarm {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.insert(agent.name.clone(), agent);
    }
}

impl Default for Swarm {
    fn default() -> Self {
        Self::new()
    }
}
