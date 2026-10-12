use crate::models::global_enums::{Metric, Period};
use crate::models::global_types::{PlayerId, RecordId};
use crate::models::player::Player;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// [Record Types & Entities](https://docs.wiseoldman.net/records-api/record-type-definitions)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub id: RecordId,
    pub player_id: PlayerId,
    pub period: Period,
    pub metric: Metric,
    pub value: f64,
    pub updated_at: DateTime<Utc>,
}

/// [Record Leaderboard Entry](https://docs.wiseoldman.net/records-api/record-type-definitions#object-record-leaderboard-entry)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordLeaderboardEntry {
    pub id: RecordId,
    pub player_id: PlayerId,
    pub period: Period,
    pub metric: Metric,
    pub value: f64,
    pub updated_at: DateTime<Utc>,
    pub player: Player,
}
