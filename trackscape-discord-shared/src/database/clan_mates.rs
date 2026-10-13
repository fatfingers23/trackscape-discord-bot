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

    /// Folds the `duplicate_id` row into `keep` in one transaction: its
    /// personal best records and collection log totals move to the kept
    /// row (keeping the better value where both rows have one), the
    /// duplicate row is deleted, and the kept row is updated to `keep`'s
    /// state — typically a rename onto the name the duplicate was holding,
    /// which can only succeed once the duplicate's row is gone. Used by
    /// the WOM sync when a member's new WOM name is already taken by an
    /// unlinked (chat-created) duplicate row.
    async fn merge_clan_mates(
        &self,
        keep: ClanMateModel,
        duplicate_id: Uuid,
    ) -> Result<ClanMateModel, anyhow::Error>;

    /// Number of active clan mates: in the WOM group, or a non-guest seen in
    /// clan chat within the last 90 days. Same predicate as
    /// [ClanMates::get_active_clan_mates_by_guild_id] and
    /// `GuildsDb::list_clans_paginated`.
    async fn get_clan_member_count(&self, guild_id: u64) -> Result<u64, Error>;

    /// Every clan mate row for the guild, including soft-removed ones. The
    /// WOM sync and name-change jobs depend on seeing all rows, so this is
    /// intentionally unfiltered; use
    /// [ClanMates::get_active_clan_mates_by_guild_id] for anything user facing.
    async fn get_clan_mates_by_guild_id(&self, guild_id: u64) -> Result<Vec<ClanMateModel>, Error>;

    /// Only the active clan mates, using the same predicate as
    /// [ClanMates::get_clan_member_count] so a list and its count always agree.
    async fn get_active_clan_mates_by_guild_id(
        &self,
        guild_id: u64,
    ) -> Result<Vec<ClanMateModel>, Error>;

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

    async fn merge_clan_mates(
        &self,
        mut keep: ClanMateModel,
        duplicate_id: Uuid,
    ) -> Result<ClanMateModel, anyhow::Error> {
        keep.player_name = keep.player_name.replace(" ", "\u{a0}");
        let mut tx = self.pool.begin().await?;

        // Personal bests: where both rows have a time for the same
        // activity, the better (lower) one wins; the rest simply move.
        // Order matters: the better time is copied onto the kept row
        // FIRST, so the delete of the duplicate's now-redundant rows sees
        // the updated value — otherwise the final move would violate the
        // (clan_mate_id, activity_id) unique constraint.
        sqlx::query!(
            r#"
            UPDATE personal_best_records kept
            SET time_in_seconds = dup.time_in_seconds,
                updated_at = dup.updated_at
            FROM personal_best_records dup
            WHERE kept.clan_mate_id = $1
              AND dup.clan_mate_id = $2
              AND kept.activity_id = dup.activity_id
              AND dup.time_in_seconds < kept.time_in_seconds
            "#,
            keep.id,
            duplicate_id
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            r#"
            DELETE FROM personal_best_records dup
            USING personal_best_records kept
            WHERE kept.clan_mate_id = $1
              AND dup.clan_mate_id = $2
              AND kept.activity_id = dup.activity_id
              AND kept.time_in_seconds <= dup.time_in_seconds
            "#,
            keep.id,
            duplicate_id
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            "UPDATE personal_best_records SET clan_mate_id = $1 WHERE clan_mate_id = $2",
            keep.id,
            duplicate_id
        )
        .execute(&mut *tx)
        .await?;

        // Collection log totals: keep the higher total per guild, move the rest.
        sqlx::query!(
            r#"
            UPDATE clan_mate_collection_log_totals kept
            SET total = GREATEST(kept.total, dup.total),
                created_at = LEAST(kept.created_at, dup.created_at)
            FROM clan_mate_collection_log_totals dup
            WHERE kept.player_id = $1
              AND dup.player_id = $2
              AND kept.guild_id = dup.guild_id
            "#,
            keep.id,
            duplicate_id
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            r#"
            UPDATE clan_mate_collection_log_totals dup
            SET player_id = $1
            WHERE dup.player_id = $2
              AND NOT EXISTS (
                  SELECT 1 FROM clan_mate_collection_log_totals kept
                  WHERE kept.player_id = $1 AND kept.guild_id = dup.guild_id
              )
            "#,
            keep.id,
            duplicate_id
        )
        .execute(&mut *tx)
        .await?;

        // The duplicate's names and sightings were folded into `keep` by
        // the caller; with its dependent rows re-pointed, deleting it
        // cascades nothing.
        sqlx::query!("DELETE FROM clan_mates WHERE id = $1", duplicate_id)
            .execute(&mut *tx)
            .await?;

        // The name is free now, so the kept row can take it.
        sqlx::query!(
            r#"
            UPDATE clan_mates
            SET player_name = $2, wom_player_id = $3, previous_names = $4, "rank" = $5, created_at = $6, in_wom = $7, wom_left_at = $8, last_seen_in_chat_at = $9
            WHERE id = $1
            "#,
            keep.id,
            keep.player_name,
            keep.wom_player_id.map(|id| id as i64),
            &keep.previous_names,
            keep.rank,
            keep.created_at,
            keep.in_wom,
            keep.wom_left_at,
            keep.last_seen_in_chat_at,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(keep)
    }

    async fn get_clan_member_count(&self, guild_id: u64) -> Result<u64, Error> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as count FROM clan_mates cm
            WHERE cm.guild_id = $1
                AND (cm.in_wom = true OR (cm."rank" IS DISTINCT FROM 'Guest' AND cm.last_seen_in_chat_at > now() - interval '90 days'))
            "#,
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

    async fn get_active_clan_mates_by_guild_id(
        &self,
        guild_id: u64,
    ) -> Result<Vec<ClanMateModel>, Error> {
        let rows = sqlx::query_as!(
            ClanMateRow,
            r#"
            SELECT id, guild_id, player_name, wom_player_id, previous_names, "rank", created_at, in_wom, wom_left_at, last_seen_in_chat_at
            FROM clan_mates cm
            WHERE cm.guild_id = $1
                AND (cm.in_wom = true OR (cm."rank" IS DISTINCT FROM 'Guest' AND cm.last_seen_in_chat_at > now() - interval '90 days'))
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
