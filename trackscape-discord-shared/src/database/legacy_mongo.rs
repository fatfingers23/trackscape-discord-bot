//! Legacy MongoDB document shapes for the one-off
//! `trackscape-discord-migrator` binary.
//!
//! Only compiled with the `legacy-mongo` feature. These structs mirror the
//! serde shape the old Mongo models had (`_id` renames, bson `ObjectId`,
//! bson `DateTime`) so the old collections can still be deserialized and
//! copied into Postgres.

use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::{
    BroadcastType, DiaryTier, DropItemBroadcast, QuestDifficulty,
};
use crate::osrs_broadcast_handler::BroadcastMessageToDiscord;
use mongodb::bson::DateTime as BsonDateTime;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegacyGuild {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub guild_id: u64,
    pub clan_name: Option<String>,
    pub broadcast_channel: Option<u64>,
    pub clan_chat_channel: Option<u64>,
    pub leagues_broadcast_channel: Option<u64>,
    pub drop_price_threshold: Option<i64>,
    pub disallowed_broadcast_types: Vec<BroadcastType>,
    pub verification_code: String,
    pub hashed_verification_code: String,
    pub min_quest_difficulty: Option<QuestDifficulty>,
    pub min_diary_tier: Option<DiaryTier>,
    pub pk_value_threshold: Option<i64>,
    pub wom_id: Option<i64>,
    pub created_at: Option<BsonDateTime>,
    pub custom_drop_broadcast_filter: Option<HashMap<BroadcastType, Vec<String>>>,
    pub collection_log_max_percentage: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegacyClanMate {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub guild_id: u64,
    pub player_name: String,
    pub wom_player_id: Option<u64>,
    pub previous_names: Vec<String>,
    pub rank: Option<String>,
    pub created_at: BsonDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegacyDropLog {
    #[serde(rename = "_id")]
    #[allow(dead_code)]
    pub id: ObjectId,
    pub guild_id: u64,
    pub drop_item: DropItemBroadcast,
    pub created_at: BsonDateTime,
}

/// The old broadcasts collection stored `created_at` either as a BSON
/// DateTime or as an RFC3339 string (the old model had custom serde helpers
/// for exactly this), so both forms are accepted.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum LegacyDateTime {
    DateTime(BsonDateTime),
    Rfc3339String(String),
}

impl LegacyDateTime {
    pub fn to_chrono(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            LegacyDateTime::DateTime(dt) => dt.to_chrono(),
            LegacyDateTime::Rfc3339String(s) => {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    // An unparseable timestamp on one row should not fail the
                    // whole migration; fall back to now.
                    .unwrap_or_else(|_| chrono::Utc::now())
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegacyBroadcast {
    #[serde(rename = "_id")]
    #[allow(dead_code)]
    pub id: ObjectId,
    pub guild_id: u64,
    pub broadcast: BroadcastMessageToDiscord,
    pub created_at: LegacyDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegacyPbActivity {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub activity_name: String,
    pub created_at: BsonDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegacyPbRecord {
    #[serde(rename = "_id")]
    #[allow(dead_code)]
    pub id: ObjectId,
    pub clan_mate_id: ObjectId,
    pub activity_id: ObjectId,
    pub guild_id: u64,
    pub time_in_seconds: f64,
    pub created_at: Option<BsonDateTime>,
    pub updated_at: Option<BsonDateTime>,
    #[serde(default)]
    pub clan_mate: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegacyClogTotal {
    pub guild_id: u64,
    pub player_id: ObjectId,
    pub total: i64,
    pub created_at: Option<BsonDateTime>,
}

/// Collection names, exactly as the old Mongo layer used them.
pub mod collections {
    pub const GUILDS: &str = "guilds";
    pub const CLAN_MATES: &str = "clan_mates";
    pub const DROP_LOGS: &str = "drop_logs";
    pub const BROADCASTS: &str = "broadcasts";
    pub const PB_ACTIVITIES: &str = "personal_best_activities";
    pub const PB_RECORDS: &str = "personal_best_records";
    pub const CLOG_TOTALS: &str = "clan_mate_collection_log_totals";
}
