use super::PersonalBestRecordsDb;
use super::clan_mates::ClanMateModel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonalBestRecordsModel {
    pub id: Uuid,
    pub clan_mate_id: Uuid,
    pub activity_id: Uuid,
    pub guild_id: u64,
    pub time_in_seconds: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub clan_mate: Option<ClanMateModel>,
}

impl PersonalBestRecordsModel {
    pub const COLLECTION_NAME: &'static str = "personal_best_records";
}

/// Row for the leaderboard query that joins clan mates in.
struct PersonalBestRecordsLeaderboardRow {
    id: Uuid,
    clan_mate_id: Uuid,
    activity_id: Uuid,
    guild_id: i64,
    time_in_seconds: f64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    cm_id: Uuid,
    cm_guild_id: i64,
    cm_player_name: String,
    cm_wom_player_id: Option<i64>,
    cm_previous_names: Vec<String>,
    cm_rank: Option<String>,
    cm_created_at: DateTime<Utc>,
    cm_in_wom: bool,
    cm_wom_left_at: Option<DateTime<Utc>>,
    cm_last_seen_in_chat_at: Option<DateTime<Utc>>,
}

impl From<PersonalBestRecordsLeaderboardRow> for PersonalBestRecordsModel {
    fn from(row: PersonalBestRecordsLeaderboardRow) -> Self {
        let clan_mate = ClanMateModel {
            id: row.cm_id,
            guild_id: row.cm_guild_id as u64,
            player_name: row.cm_player_name,
            wom_player_id: row.cm_wom_player_id.map(|id| id as u64),
            previous_names: row.cm_previous_names,
            rank: row.cm_rank,
            created_at: row.cm_created_at,
            in_wom: row.cm_in_wom,
            wom_left_at: row.cm_wom_left_at,
            last_seen_in_chat_at: row.cm_last_seen_in_chat_at,
        };
        Self {
            id: row.id,
            clan_mate_id: row.clan_mate_id,
            activity_id: row.activity_id,
            guild_id: row.guild_id as u64,
            time_in_seconds: row.time_in_seconds,
            created_at: row.created_at,
            updated_at: row.updated_at,
            clan_mate: Some(clan_mate),
        }
    }
}

impl PersonalBestRecordsDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_or_update_pb_record(
        &self,
        clan_mate_id: Uuid,
        activity_id: Uuid,
        guild_id: u64,
        time_in_seconds: f64,
    ) -> Result<(), anyhow::Error> {
        // Upserts the record, but only overwrites the stored time when the
        // new time is better (lower). Ties keep the existing row.
        sqlx::query!(
            r#"
            INSERT INTO personal_best_records
                (id, clan_mate_id, activity_id, guild_id, time_in_seconds, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, now(), now())
            ON CONFLICT (clan_mate_id, activity_id) DO UPDATE
                SET time_in_seconds = EXCLUDED.time_in_seconds, updated_at = now()
                WHERE personal_best_records.time_in_seconds > EXCLUDED.time_in_seconds
            "#,
            Uuid::new_v4(),
            clan_mate_id,
            activity_id,
            guild_id as i64,
            time_in_seconds,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_pb_records_leaderboard(
        &self,
        activity_id: Uuid,
        guild_id: u64,
    ) -> Result<Vec<PersonalBestRecordsModel>, anyhow::Error> {
        let rows = sqlx::query_as!(
            PersonalBestRecordsLeaderboardRow,
            r#"
            SELECT r.id, r.clan_mate_id, r.activity_id, r.guild_id, r.time_in_seconds,
                   r.created_at, r.updated_at,
                   c.id as cm_id, c.guild_id as cm_guild_id, c.player_name as cm_player_name,
                   c.wom_player_id as cm_wom_player_id, c.previous_names as cm_previous_names,
                   c."rank" as cm_rank, c.created_at as cm_created_at,
                   c.in_wom as cm_in_wom, c.wom_left_at as cm_wom_left_at,
                   c.last_seen_in_chat_at as cm_last_seen_in_chat_at
            FROM personal_best_records r
            JOIN clan_mates c ON c.id = r.clan_mate_id
            WHERE r.guild_id = $1 AND r.activity_id = $2
            ORDER BY r.time_in_seconds ASC, r.updated_at ASC
            "#,
            guild_id as i64,
            activity_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|row| row.into()).collect())
    }
}
