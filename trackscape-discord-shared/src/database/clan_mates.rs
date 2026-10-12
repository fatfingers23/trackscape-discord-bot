use crate::database::ClanMatesDb;
use anyhow::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mockall::automock;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ClanMateModel {
    pub id: Uuid,
    pub guild_id: u64,
    pub player_name: String,
    pub wom_player_id: Option<u64>,
    pub previous_names: Vec<String>,
    pub rank: Option<String>,
    pub created_at: DateTime<Utc>,
    pub in_wom: bool,
    pub wom_left_at: Option<DateTime<Utc>>,
    pub last_seen_in_chat_at: Option<DateTime<Utc>>,
}

impl ClanMateModel {
    pub const COLLECTION_NAME: &'static str = "clan_mates";

    pub fn new(guild_id: u64, player_name: String, wom_player_id: Option<u64>) -> Self {
        Self {
            id: Uuid::new_v4(),
            guild_id,
            wom_player_id,
            previous_names: Vec::new(),
            player_name,
            rank: None,
            created_at: Utc::now(),
            in_wom: false,
            wom_left_at: None,
            last_seen_in_chat_at: None,
        }
    }
}

/// Private row struct matching the Postgres column types exactly
/// (Discord snowflakes are bigint).
struct ClanMateRow {
    id: Uuid,
    guild_id: i64,
    player_name: String,
    wom_player_id: Option<i64>,
    previous_names: Vec<String>,
    rank: Option<String>,
    created_at: DateTime<Utc>,
    in_wom: bool,
    wom_left_at: Option<DateTime<Utc>>,
    last_seen_in_chat_at: Option<DateTime<Utc>>,
}

impl From<ClanMateRow> for ClanMateModel {
    fn from(row: ClanMateRow) -> Self {
        Self {
            id: row.id,
            guild_id: row.guild_id as u64,
            player_name: row.player_name,
            wom_player_id: row.wom_player_id.map(|id| id as u64),
            previous_names: row.previous_names,
            rank: row.rank,
            created_at: row.created_at,
            in_wom: row.in_wom,
            wom_left_at: row.wom_left_at,
            last_seen_in_chat_at: row.last_seen_in_chat_at,
        }
    }
}

impl ClanMatesDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[automock]
#[async_trait]
pub trait ClanMates {
    async fn find_or_create_clan_mate(
        &self,
        guild_id: u64,
        player_name: String,
    ) -> Result<ClanMateModel, anyhow::Error>;

    async fn create_new_clan_mate(
        &self,
        guild_id: u64,
        player_name: String,
        wom_player_id: Option<u64>,
    ) -> Result<ClanMateModel, anyhow::Error>;

    async fn find_by_current_name(
        &self,
        guild_id: u64,
        player_name: String,
    ) -> Result<Option<ClanMateModel>, anyhow::Error>;

    async fn update_clan_mate(&self, model: ClanMateModel) -> Result<ClanMateModel, anyhow::Error>;

    async fn get_clan_member_count(&self, guild_id: u64) -> Result<u64, Error>;

    async fn get_clan_mates_by_guild_id(&self, guild_id: u64) -> Result<Vec<ClanMateModel>, Error>;

    async fn remove_clan_mate(&self, guild_id: u64, player_name: String) -> Result<(), Error>;

    async fn change_name(
        &self,
        guild_id: u64,
        old_name: String,
        new_name: String,
    ) -> Result<ClanMateModel, Error>;

    async fn touch_last_seen_in_chat(&self, clan_mate_id: Uuid) -> Result<(), Error>;
}

#[async_trait]
impl ClanMates for ClanMatesDb {
    async fn find_or_create_clan_mate(
        &self,
        guild_id: u64,
        player_name: String,
    ) -> Result<ClanMateModel, Error> {
        let possible_clan_mate = self
            .find_by_current_name(guild_id, player_name.clone())
            .await?;
        return Ok(match possible_clan_mate {
            None => {
                self.create_new_clan_mate(guild_id, player_name.replace(" ", "\u{a0}"), None)
                    .await?
            }
            Some(clan_mate) => clan_mate,
        });
    }

    async fn create_new_clan_mate(
        &self,
        guild_id: u64,
        player_name: String,
        wom_player_id: Option<u64>,
    ) -> Result<ClanMateModel, anyhow::Error> {
        let mut clan_mate =
            ClanMateModel::new(guild_id, player_name.replace(" ", "\u{a0}"), wom_player_id);
        clan_mate.in_wom = clan_mate.wom_player_id.is_some();
        sqlx::query!(
            r#"
            INSERT INTO clan_mates (id, guild_id, player_name, wom_player_id, previous_names, "rank", created_at, in_wom, wom_left_at, last_seen_in_chat_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            clan_mate.id,
            clan_mate.guild_id as i64,
            clan_mate.player_name,
            clan_mate.wom_player_id.map(|id| id as i64),
            &clan_mate.previous_names,
            clan_mate.rank,
            clan_mate.created_at,
            clan_mate.in_wom,
            clan_mate.wom_left_at,
            clan_mate.last_seen_in_chat_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(clan_mate)
    }

    async fn find_by_current_name(
        &self,
        guild_id: u64,
        player_name: String,
    ) -> Result<Option<ClanMateModel>, anyhow::Error> {
        let row = sqlx::query_as!(
            ClanMateRow,
            r#"
            SELECT id, guild_id, player_name, wom_player_id, previous_names, "rank", created_at, in_wom, wom_left_at, last_seen_in_chat_at
            FROM clan_mates
            WHERE guild_id = $1 AND player_name = $2
            "#,
            guild_id as i64,
            player_name.replace(" ", "\u{a0}")
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|row| row.into()))
    }

    async fn update_clan_mate(&self, mut model: ClanMateModel) -> Result<ClanMateModel, Error> {
        model.player_name = model.player_name.replace(" ", "\u{a0}");
        sqlx::query!(
            r#"
            UPDATE clan_mates
            SET player_name = $2, wom_player_id = $3, previous_names = $4, "rank" = $5, created_at = $6, in_wom = $7, wom_left_at = $8, last_seen_in_chat_at = $9
            WHERE id = $1
            "#,
            model.id,
            model.player_name,
            model.wom_player_id.map(|id| id as i64),
            &model.previous_names,
            model.rank,
            model.created_at,
            model.in_wom,
            model.wom_left_at,
            model.last_seen_in_chat_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(model)
    }

    async fn get_clan_member_count(&self, guild_id: u64) -> Result<u64, Error> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) as count FROM clan_mates WHERE guild_id = $1 AND (in_wom = true OR last_seen_in_chat_at > now() - interval '90 days')",
            guild_id as i64
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(count.unwrap_or(0) as u64)
    }

    async fn get_clan_mates_by_guild_id(&self, guild_id: u64) -> Result<Vec<ClanMateModel>, Error> {
        let rows = sqlx::query_as!(
            ClanMateRow,
            r#"
            SELECT id, guild_id, player_name, wom_player_id, previous_names, "rank", created_at, in_wom, wom_left_at, last_seen_in_chat_at
            FROM clan_mates
            WHERE guild_id = $1
            ORDER BY created_at ASC
            "#,
            guild_id as i64
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    async fn remove_clan_mate(&self, guild_id: u64, player_name: String) -> Result<(), Error> {
        let possible_player = self
            .find_by_current_name(guild_id, player_name.clone())
            .await?;
        if possible_player.is_none() {
            return Err(anyhow::anyhow!(format!(
                "Failed to find clan mate: {}",
                player_name
            )));
        }

        // The personal best records and collection log totals for this clan
        // mate are removed by the foreign keys' ON DELETE CASCADE.
        let result = sqlx::query!(
            r#"
            DELETE FROM clan_mates
            WHERE guild_id = $1 AND player_name = $2
            "#,
            guild_id as i64,
            player_name.replace(" ", "\u{a0}")
        )
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Failed to remove clan mate"));
        }
        Ok(())
    }

    async fn change_name(
        &self,
        guild_id: u64,
        old_name: String,
        new_name: String,
    ) -> Result<ClanMateModel, Error> {
        let clan_mate = self
            .find_by_current_name(guild_id, old_name.clone())
            .await?;
        if clan_mate.is_none() {
            return Err(anyhow::anyhow!("Failed to find clan mate"));
        }
        let mut clan_mate = clan_mate.unwrap();
        if clan_mate.guild_id != guild_id {
            return Err(anyhow::anyhow!("Clan mate is not in this clan!"));
        }
        clan_mate
            .previous_names
            .push(old_name.replace(" ", "\u{a0}"));
        clan_mate.player_name = new_name.replace(" ", "\u{a0}");
        self.update_clan_mate(clan_mate).await
    }

    async fn touch_last_seen_in_chat(&self, clan_mate_id: Uuid) -> Result<(), Error> {
        sqlx::query!(
            r#"
            UPDATE clan_mates SET last_seen_in_chat_at = now() WHERE id = $1
            "#,
            clan_mate_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub fn name_compare(name1: &str, name2: &str) -> bool {
    name_normalize(name1) == name_normalize(name2)
}

pub fn name_normalize(name: &str) -> String {
    name.replace(" ", "\u{a0}").to_lowercase()
}
