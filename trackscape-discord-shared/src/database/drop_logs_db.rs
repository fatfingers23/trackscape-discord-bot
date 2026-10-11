use crate::database::DropLogsDb;
use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::DropItemBroadcast;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::error;
use mockall::automock;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DropLogModel {
    pub id: Uuid,
    pub guild_id: u64,
    pub drop_item: DropItemBroadcast,
    pub created_at: DateTime<Utc>,
}

impl DropLogModel {
    pub const COLLECTION_NAME: &'static str = "drop_logs";

    pub fn new(drop_item: DropItemBroadcast, guild_id: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            guild_id,
            drop_item,
            created_at: Utc::now(),
        }
    }
}

/// Private row struct matching the Postgres column types exactly
/// (Discord snowflakes are bigint, the embedded drop document is jsonb).
struct DropLogRow {
    id: Uuid,
    guild_id: i64,
    drop_item: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl DropLogRow {
    fn into_model(self) -> Option<DropLogModel> {
        match serde_json::from_value::<DropItemBroadcast>(self.drop_item) {
            Ok(drop_item) => Some(DropLogModel {
                id: self.id,
                guild_id: self.guild_id as u64,
                drop_item,
                created_at: self.created_at,
            }),
            Err(e) => {
                error!("Found a corrupt drop log row {}: {:?}", self.id, e);
                None
            }
        }
    }
}

impl DropLogsDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[automock]
#[async_trait]
pub trait DropLogs {
    async fn new_drop_log(&self, drop_log: DropItemBroadcast, guild_id: u64);
    async fn get_drops_between_dates(
        &self,
        guild_id: u64,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> anyhow::Result<Vec<DropLogModel>>;
}

#[async_trait]
impl DropLogs for DropLogsDb {
    async fn new_drop_log(&self, drop_broadcast: DropItemBroadcast, guild_id: u64) {
        let new_drop_log = DropLogModel::new(drop_broadcast, guild_id);
        let result = sqlx::query!(
            r#"
            INSERT INTO drop_logs (id, guild_id, drop_item, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
            new_drop_log.id,
            new_drop_log.guild_id as i64,
            serde_json::to_value(&new_drop_log.drop_item).unwrap_or_default(),
            new_drop_log.created_at,
        )
        .execute(&self.pool)
        .await;
        if let Err(e) = result {
            error!("Failed to insert document for a new drop log: {:?}", e);
        }
    }

    async fn get_drops_between_dates(
        &self,
        guild_id: u64,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> anyhow::Result<Vec<DropLogModel>> {
        let rows = sqlx::query_as!(
            DropLogRow,
            r#"
            SELECT id, guild_id, drop_item, created_at
            FROM drop_logs
            WHERE guild_id = $1 AND created_at >= $2 AND created_at <= $3
            ORDER BY created_at ASC
            "#,
            guild_id as i64,
            start_date,
            end_date
        )
        .fetch_all(&self.pool)
        .await?;
        // Corrupt jsonb rows are logged and skipped instead of failing the
        // whole query.
        Ok(rows
            .into_iter()
            .filter_map(|row| row.into_model())
            .collect())
    }
}
