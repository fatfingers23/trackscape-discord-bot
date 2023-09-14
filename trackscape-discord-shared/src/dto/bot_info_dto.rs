use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct DiscordServerCount {
    pub server_count: i64,
}
