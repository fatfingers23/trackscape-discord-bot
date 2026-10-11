use crate::database::BroadcastsDb;
use crate::osrs_broadcast_handler::BroadcastMessageToDiscord;
use chrono::{DateTime, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BroadcastModel {
    pub id: Uuid,
    pub guild_id: u64,
    pub broadcast: BroadcastMessageToDiscord,
    pub created_at: DateTime<Utc>,
}

/// Private row struct matching the Postgres column types exactly
/// (Discord snowflakes are bigint, the embedded broadcast is jsonb).
struct BroadcastRow {
    id: Uuid,
    guild_id: i64,
    broadcast: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl BroadcastRow {
    fn into_model(self) -> Option<BroadcastModel> {
        match serde_json::from_value::<BroadcastMessageToDiscord>(self.broadcast) {
            Ok(broadcast) => Some(BroadcastModel {
                id: self.id,
                guild_id: self.guild_id as u64,
                broadcast,
                created_at: self.created_at,
            }),
            Err(e) => {
                error!("Found a corrupt broadcast row {}: {:?}", self.id, e);
                None
            }
        }
    }
}

impl BroadcastsDb {
    pub const COLLECTION_NAME: &'static str = "broadcasts";
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_broadcast(
        &self,
        guild_id: u64,
        broadcast: BroadcastMessageToDiscord,
    ) -> Result<(), anyhow::Error> {
        let model = BroadcastModel {
            id: Uuid::new_v4(),
            guild_id,
            broadcast,
            created_at: Utc::now(),
        };
        sqlx::query!(
            r#"
            INSERT INTO broadcasts (id, guild_id, broadcast, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
            model.id,
            model.guild_id as i64,
            serde_json::to_value(&model.broadcast).unwrap_or_default(),
            model.created_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_latest_broadcasts(
        &self,
        guild_id: u64,
        limit: i64,
    ) -> Result<Vec<BroadcastModel>, anyhow::Error> {
        let rows = sqlx::query_as!(
            BroadcastRow,
            r#"
            SELECT id, guild_id, broadcast, created_at
            FROM broadcasts
            WHERE guild_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            guild_id as i64,
            limit
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
