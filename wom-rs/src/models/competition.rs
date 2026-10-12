use crate::models::global_enums::Metric;
use crate::models::global_types::{CompetitionId, GroupId, PlayerId};
use crate::models::group::Group;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// [Competition Type](https://docs.wiseoldman.net/competitions-api/competition-type-definitions#enum-competition-type)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompetitionType {
    Classic,
    Team,
}

/// [Competition Status](https://docs.wiseoldman.net/competitions-api/competition-type-definitions#enum-competition-status)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompetitionStatus {
    Upcoming,
    Ongoing,
    Finished,
}

impl CompetitionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CompetitionStatus::Upcoming => "upcoming",
            CompetitionStatus::Ongoing => "ongoing",
            CompetitionStatus::Finished => "finished",
        }
    }
}

/// [Competition Type](https://docs.wiseoldman.net/competitions-api/competition-type-definitions#enum-competition-csv-table-type)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompetitionCSVTableType {
    Team,
    Teams,
    Participants,
}

/// [Competition Progress](https://docs.wiseoldman.net/competitions-api/competition-type-definitions#object-competition-progress)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionProgress {
    pub start: i64,
    pub end: i64,
    pub gained: i64,
}

/// [Competition Levels Progress](https://docs.wiseoldman.net/competitions-api/competition-type-definitions#object-competition-levels-progress)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionLevelsProgress {
    pub start: i64,
    pub end: i64,
    pub gained: i64,
}

/// [Competition](https://docs.wiseoldman.net/competitions-api/competition-type-definitions#object-competition)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Competition {
    pub id: i64,
    pub title: String,
    pub metric: Metric,
    #[serde(rename = "type")]
    pub competition_type: CompetitionType,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub group_id: Option<GroupId>,
    pub score: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub participant_count: i64,
    pub group: Option<Group>,
}

///[Participation](https://docs.wiseoldman.net/competitions-api/competition-type-definitions#object-participation)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Participation {
    pub player_id: PlayerId,
    pub competition_id: CompetitionId,
    pub team_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// [Player Participation](https://docs.wiseoldman.net/competitions-api/competition-type-definitions#object-player-participation)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerParticipation {
    pub player_id: PlayerId,
    pub competition_id: CompetitionId,
    pub team_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub competition: Competition,
}

/// [Player Competition Standing](https://docs.wiseoldman.net/competitions-api/competition-type-definitions#object-player-competition-standing)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerCompetitionStanding {
    pub player_id: PlayerId,
    pub competition_id: CompetitionId,
    pub team_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub competition: Competition,
    pub progress: CompetitionProgress,
    pub levels: Option<CompetitionLevelsProgress>,
    pub rank: i64,
}
