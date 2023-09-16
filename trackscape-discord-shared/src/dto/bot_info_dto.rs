use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DiscordServerCount {
    pub server_count: i64,
}
