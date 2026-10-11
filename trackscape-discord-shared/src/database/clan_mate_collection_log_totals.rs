use crate::database::ClanMateCollectionLogTotalsDb;
use anyhow::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::info;
use mockall::automock;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::clan_mates::ClanMateModel;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClanMateCollectionLogTotalModel {
    pub guild_id: u64,
    pub player_id: Uuid,
    pub total: i64,
    pub created_at: DateTime<Utc>,
    pub clan_mate: Option<ClanMateModel>,
}

impl ClanMateCollectionLogTotalModel {
    pub const COLLECTION_NAME: &'static str = "clan_mate_collection_log_totals";
    pub fn new(guild_id: u64, player_id: Uuid, total: i64) -> Self {
        Self {
            guild_id,
            player_id,
            total,
            created_at: Utc::now(),
            clan_mate: None,
        }
    }
}

/// Row for the guild totals query that joins clan mates in.
struct GuildTotalsRow {
    guild_id: i64,
    player_id: Uuid,
    total: i64,
    created_at: DateTime<Utc>,
    cm_id: Uuid,
    cm_guild_id: i64,
    cm_player_name: String,
    cm_wom_player_id: Option<i64>,
    cm_previous_names: Vec<String>,
    cm_rank: Option<String>,
    cm_created_at: DateTime<Utc>,
}

impl From<GuildTotalsRow> for ClanMateCollectionLogTotalModel {
    fn from(row: GuildTotalsRow) -> Self {
        let clan_mate = ClanMateModel {
            id: row.cm_id,
            guild_id: row.cm_guild_id as u64,
            player_name: row.cm_player_name,
            wom_player_id: row.cm_wom_player_id.map(|id| id as u64),
            previous_names: row.cm_previous_names,
            rank: row.cm_rank,
            created_at: row.cm_created_at,
        };
        Self {
            guild_id: row.guild_id as u64,
            player_id: row.player_id,
            total: row.total,
            created_at: row.created_at,
            clan_mate: Some(clan_mate),
        }
    }
}

impl ClanMateCollectionLogTotalsDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[automock]
#[async_trait]
pub trait ClanMateCollectionLogTotals {
    async fn update_or_create(
        &self,
        guild_id: u64,
        player_id: Uuid,
        total: i64,
    ) -> Result<(), anyhow::Error>;

    async fn get_guild_totals(
        &self,
        guild_id: u64,
    ) -> Result<Vec<ClanMateCollectionLogTotalModel>, anyhow::Error>;
}

#[async_trait]
impl ClanMateCollectionLogTotals for ClanMateCollectionLogTotalsDb {
    async fn update_or_create(
        &self,
        guild_id: u64,
        player_id: Uuid,
        total: i64,
    ) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"
            INSERT INTO clan_mate_collection_log_totals (guild_id, player_id, total, created_at)
            VALUES ($1, $2, $3, now())
            ON CONFLICT (guild_id, player_id) DO UPDATE SET total = EXCLUDED.total
            "#,
            guild_id as i64,
            player_id,
            total,
        )
        .execute(&self.pool)
        .await?;
        info!("Upserted collection log total, rows affected: {:?}", result);
        Ok(())
    }

    async fn get_guild_totals(
        &self,
        guild_id: u64,
    ) -> Result<Vec<ClanMateCollectionLogTotalModel>, anyhow::Error> {
        let rows = sqlx::query_as!(
            GuildTotalsRow,
            r#"
            SELECT t.guild_id, t.player_id, t.total, t.created_at,
                   c.id as cm_id, c.guild_id as cm_guild_id, c.player_name as cm_player_name,
                   c.wom_player_id as cm_wom_player_id, c.previous_names as cm_previous_names,
                   c."rank" as cm_rank, c.created_at as cm_created_at
            FROM clan_mate_collection_log_totals t
            JOIN clan_mates c ON c.id = t.player_id
            WHERE t.guild_id = $1
            ORDER BY t.total DESC
            "#,
            guild_id as i64
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|row| row.into()).collect())
    }
}
