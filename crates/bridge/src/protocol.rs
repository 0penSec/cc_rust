//! Bridge protocol definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    Ping,
    Pong,
    Execute { command: String },
    Result { success: bool, output: String },
}
