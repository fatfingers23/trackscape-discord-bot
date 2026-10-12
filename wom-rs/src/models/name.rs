use crate::models::global_types::PlayerId;
use crate::models::player::Player;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// [Name Change Status](https://docs.wiseoldman.net/names-api/name-type-definitions#enum-name-change-status)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NameChangeStatus {
    Pending,
    Approved,
    Denied,
}

/// [Name Change](https://docs.wiseoldman.net/names-api/name-type-definitions#object-name-change)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NameChange {
    pub id: i64,
    pub player_id: PlayerId,
    pub old_name: String,
    pub new_name: String,
    pub status: NameChangeStatus,
    /// There was just too many different types and not sure how much of this would really be used. Opted for serde_json::Value
    /// [Name Change Review Context](https://docs.wiseoldman.net/names-api/name-type-definitions#name-change-review-context)
    pub review_context: Option<serde_json::Value>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub player: Option<Player>,
}
