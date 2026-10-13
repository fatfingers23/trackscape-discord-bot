use crate::database::GuildsDb;
use crate::database::enum_serde;
use crate::helpers::hash_string;
use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::{
    BroadcastType, DiaryTier, QuestDifficulty,
};
use anyhow::Result;
use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use log::error;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisteredGuildModel {
    pub id: Uuid,
    pub guild_id: u64,
    //Channel to send broadcast messages
    pub clan_name: Option<String>,
    pub broadcast_channel: Option<u64>,
    //Channel to send clan chats messages
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
    pub created_at: Option<DateTime<Utc>>,
    pub custom_drop_broadcast_filter: Option<HashMap<BroadcastType, Vec<String>>>,
    pub collection_log_max_percentage: Option<f64>,
}

/// A clan as it appears in the public clans list: just the fields the
/// dashboard renders, plus the active member count from the JOIN in
/// [GuildsDb::list_clans_paginated].
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClanListEntry {
    pub id: Uuid,
    pub clan_name: String,
    pub registered_members: i64,
}

impl RegisteredGuildModel {
    pub const COLLECTION_NAME: &'static str = "guilds";
    pub fn new(guild_id: u64) -> Self {
        let verification_code = Self::generate_code();
        let hashed_verification_code = hash_string(verification_code.clone());
        Self {
            id: Uuid::new_v4(),
            guild_id,
            clan_name: None,
            broadcast_channel: None,
            clan_chat_channel: None,
            leagues_broadcast_channel: None,
            drop_price_threshold: None,
            disallowed_broadcast_types: Vec::new(),
            verification_code,
            hashed_verification_code,
            min_quest_difficulty: None,
            min_diary_tier: None,
            pk_value_threshold: None,
            wom_id: None,
            created_at: Some(Utc::now()),
            custom_drop_broadcast_filter: Some(HashMap::new()),
            collection_log_max_percentage: None,
        }
    }

    fn generate_code() -> String {
        let mut code = String::new();
        let mut rng = rand::thread_rng();

        for i in 0..9 {
            code.push_str(&rng.gen_range(0..10).to_string());

            if i == 2 || i == 5 {
                code.push('-');
            }
        }
        code
    }
}

/// Private row struct that matches the Postgres column types exactly
/// (Discord snowflakes are bigint, enums are text, the filter is jsonb).
/// Converted into the public model with a `From` impl.
struct GuildRow {
    id: Uuid,
    guild_id: i64,
    clan_name: Option<String>,
    broadcast_channel: Option<i64>,
    clan_chat_channel: Option<i64>,
    leagues_broadcast_channel: Option<i64>,
    drop_price_threshold: Option<i64>,
    disallowed_broadcast_types: Vec<String>,
    verification_code: String,
    hashed_verification_code: String,
    min_quest_difficulty: Option<String>,
    min_diary_tier: Option<String>,
    pk_value_threshold: Option<i64>,
    wom_id: Option<i64>,
    created_at: Option<DateTime<Utc>>,
    custom_drop_broadcast_filter: Option<serde_json::Value>,
    collection_log_max_percentage: Option<f64>,
}

impl From<GuildRow> for RegisteredGuildModel {
    fn from(row: GuildRow) -> Self {
        Self {
            id: row.id,
            guild_id: row.guild_id as u64,
            clan_name: row.clan_name,
            broadcast_channel: row.broadcast_channel.map(|c| c as u64),
            clan_chat_channel: row.clan_chat_channel.map(|c| c as u64),
            leagues_broadcast_channel: row.leagues_broadcast_channel.map(|c| c as u64),
            drop_price_threshold: row.drop_price_threshold,
            disallowed_broadcast_types: enum_serde::enum_vec_from_strings(
                row.disallowed_broadcast_types,
            ),
            verification_code: row.verification_code,
            hashed_verification_code: row.hashed_verification_code,
            min_quest_difficulty: row
                .min_quest_difficulty
                .as_deref()
                .and_then(enum_serde::enum_from_string),
            min_diary_tier: row
                .min_diary_tier
                .as_deref()
                .and_then(enum_serde::enum_from_string),
            pk_value_threshold: row.pk_value_threshold,
            wom_id: row.wom_id,
            created_at: row.created_at,
            custom_drop_broadcast_filter: enum_serde::custom_filter_from_json(
                row.custom_drop_broadcast_filter,
            ),
            collection_log_max_percentage: row.collection_log_max_percentage,
        }
    }
}

impl GuildsDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_if_new_guild(&self, guild_id: u64) {
        let saved_guild_query = self.get_by_guild_id(guild_id).await;
        match saved_guild_query {
            Ok(saved_guild) => {
                if saved_guild.is_none() {
                    self.save_new_guild(guild_id).await;
                }
            }
            Err(e) => {
                error!(
                    "Failed to check if guild {} already exists: {:?}",
                    guild_id, e
                );
            }
        }
    }

    pub async fn save_new_guild(&self, guild_id: u64) {
        let mut guild = RegisteredGuildModel::new(guild_id);
        let check_for_unique_code = self
            .recursive_check_for_unique_code(guild.verification_code.clone())
            .await;
        match check_for_unique_code {
            Ok(code) => {
                guild.verification_code = code.clone();
                guild.hashed_verification_code = hash_string(code.clone());
                let disallowed_broadcast_types =
                    enum_serde::enum_vec_to_strings(&guild.disallowed_broadcast_types);
                let custom_drop_broadcast_filter = guild
                    .custom_drop_broadcast_filter
                    .as_ref()
                    .map(enum_serde::custom_filter_to_json);
                let result = sqlx::query!(
                    r#"
                    INSERT INTO guilds (id, guild_id, clan_name, broadcast_channel, clan_chat_channel,
                                        leagues_broadcast_channel, drop_price_threshold,
                                        disallowed_broadcast_types, verification_code,
                                        hashed_verification_code, min_quest_difficulty, min_diary_tier,
                                        pk_value_threshold, wom_id, created_at,
                                        custom_drop_broadcast_filter, collection_log_max_percentage)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
                    "#,
                    guild.id,
                    guild.guild_id as i64,
                    guild.clan_name,
                    guild.broadcast_channel.map(|c| c as i64),
                    guild.clan_chat_channel.map(|c| c as i64),
                    guild.leagues_broadcast_channel.map(|c| c as i64),
                    guild.drop_price_threshold,
                    &disallowed_broadcast_types,
                    guild.verification_code,
                    guild.hashed_verification_code,
                    guild.min_quest_difficulty.as_ref().map(enum_serde::enum_to_string),
                    guild.min_diary_tier.as_ref().map(enum_serde::enum_to_string),
                    guild.pk_value_threshold,
                    guild.wom_id,
                    guild.created_at,
                    custom_drop_broadcast_filter,
                    guild.collection_log_max_percentage,
                )
                .execute(&self.pool)
                .await;
                if let Err(e) = result {
                    error!("Failed to insert document for a new guild: {:?}", e);
                }
            }
            Err(e) => {
                error!("Failed to find a unique verification code: {:?}", e);
            }
        }
    }

    #[async_recursion]
    pub async fn recursive_check_for_unique_code(
        &self,
        code: String,
    ) -> Result<String, anyhow::Error> {
        let hashed_code = hash_string(code.clone());
        let result = sqlx::query!(
            "SELECT id FROM guilds WHERE hashed_verification_code = $1",
            hashed_code
        )
        .fetch_optional(&self.pool)
        .await?;
        match result {
            Some(_) => {
                let new_code = RegisteredGuildModel::generate_code();
                self.recursive_check_for_unique_code(new_code).await
            }
            None => Ok(code),
        }
    }

    pub async fn reset_verification_code(&self, guild_id: u64) -> Result<String, anyhow::Error> {
        let saved_guild_query = self.get_by_guild_id(guild_id).await;
        match saved_guild_query {
            Ok(saved_guild) => match saved_guild {
                Some(mut guild) => {
                    let check_for_unique_code = self
                        .recursive_check_for_unique_code(guild.verification_code.clone())
                        .await;
                    match check_for_unique_code {
                        Ok(new_code) => {
                            guild.verification_code = new_code.clone();
                            guild.hashed_verification_code = hash_string(new_code.clone());
                            self.update_guild(guild).await;
                            Ok(new_code)
                        }
                        Err(_) => Err(anyhow::Error::msg("Failed to reset verification code.")),
                    }
                }
                None => Err(anyhow::Error::msg(
                    "Could not find a clan with that guild id.",
                )),
            },
            Err(_) => Err(anyhow::Error::msg(
                "Failed to get the saved guild to  reset verification code.",
            )),
        }
    }

    pub async fn get_guild_by_code(
        &self,
        code: String,
    ) -> Result<Option<RegisteredGuildModel>, anyhow::Error> {
        let hashed_code = hash_string(code);
        let row = sqlx::query_as!(
            GuildRow,
            r#"
            SELECT id, guild_id, clan_name, broadcast_channel, clan_chat_channel,
                   leagues_broadcast_channel, drop_price_threshold,
                   disallowed_broadcast_types, verification_code, hashed_verification_code,
                   min_quest_difficulty, min_diary_tier, pk_value_threshold, wom_id, created_at,
                   custom_drop_broadcast_filter,
                   collection_log_max_percentage
            FROM guilds
            WHERE hashed_verification_code = $1
            "#,
            hashed_code
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|row| row.into()))
    }

    pub async fn get_by_guild_id(
        &self,
        id: u64,
    ) -> Result<Option<RegisteredGuildModel>, sqlx::Error> {
        let row = sqlx::query_as!(
            GuildRow,
            r#"
            SELECT id, guild_id, clan_name, broadcast_channel, clan_chat_channel,
                   leagues_broadcast_channel, drop_price_threshold,
                   disallowed_broadcast_types, verification_code, hashed_verification_code,
                   min_quest_difficulty, min_diary_tier, pk_value_threshold, wom_id, created_at,
                   custom_drop_broadcast_filter,
                   collection_log_max_percentage
            FROM guilds
            WHERE guild_id = $1
            "#,
            id as i64
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|row| row.into()))
    }

    pub async fn update_guild(&self, guild: RegisteredGuildModel) {
        let disallowed_broadcast_types =
            enum_serde::enum_vec_to_strings(&guild.disallowed_broadcast_types);
        let custom_drop_broadcast_filter = guild
            .custom_drop_broadcast_filter
            .as_ref()
            .map(enum_serde::custom_filter_to_json);
        let result = sqlx::query!(
            r#"
            UPDATE guilds
            SET clan_name = $2, broadcast_channel = $3, clan_chat_channel = $4,
                leagues_broadcast_channel = $5, drop_price_threshold = $6,
                disallowed_broadcast_types = $7, verification_code = $8,
                hashed_verification_code = $9, min_quest_difficulty = $10, min_diary_tier = $11,
                pk_value_threshold = $12, wom_id = $13, created_at = $14,
                custom_drop_broadcast_filter = $15, collection_log_max_percentage = $16
            WHERE guild_id = $1
            "#,
            guild.guild_id as i64,
            guild.clan_name,
            guild.broadcast_channel.map(|c| c as i64),
            guild.clan_chat_channel.map(|c| c as i64),
            guild.leagues_broadcast_channel.map(|c| c as i64),
            guild.drop_price_threshold,
            &disallowed_broadcast_types,
            guild.verification_code,
            guild.hashed_verification_code,
            guild
                .min_quest_difficulty
                .as_ref()
                .map(enum_serde::enum_to_string),
            guild
                .min_diary_tier
                .as_ref()
                .map(enum_serde::enum_to_string),
            guild.pk_value_threshold,
            guild.wom_id,
            guild.created_at,
            custom_drop_broadcast_filter,
            guild.collection_log_max_percentage,
        )
        .execute(&self.pool)
        .await;
        if let Err(e) = result {
            error!("Failed to update document for the Discord guild: {:?}", e);
        }
    }

    pub async fn delete_guild(&self, guild_id: u64) {
        let result = sqlx::query!("DELETE FROM guilds WHERE guild_id = $1", guild_id as i64)
            .execute(&self.pool)
            .await;
        if let Err(e) = result {
            error!("Failed to delete document for the Discord guild: {:?}", e);
        }
    }

    /// Total number of clans, optionally narrowed by a case-insensitive name
    /// search. Matches the filter used by [GuildsDb::list_clans_paginated].
    pub async fn count_clans(&self, search: Option<&str>) -> Result<i64, anyhow::Error> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as count FROM guilds
            WHERE clan_name IS NOT NULL AND clan_name <> ''
                AND ($1::text IS NULL OR clan_name ILIKE '%' || $1 || '%')
            "#,
            search
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(count.unwrap_or(0))
    }

    /// One page of clans with their active member counts folded in (a LEFT
    /// JOIN instead of the old per-clan COUNT round-trips). The member-count
    /// predicate (left_at IS NULL and not a Guest) mirrors
    /// `ClanMates::get_clan_member_count`.
    pub async fn list_clans_paginated(
        &self,
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ClanListEntry>, anyhow::Error> {
        let rows = sqlx::query_as!(
            ClanListEntry,
            r#"
            SELECT g.id, g.clan_name AS "clan_name!", COUNT(cm.guild_id) AS "registered_members!"
            FROM guilds g
            LEFT JOIN clan_mates cm ON cm.guild_id = g.guild_id
                AND cm.left_at IS NULL AND cm."rank" IS DISTINCT FROM 'Guest'
            WHERE g.clan_name IS NOT NULL AND g.clan_name <> ''
                AND ($1::text IS NULL OR g.clan_name ILIKE '%' || $1 || '%')
            GROUP BY g.id, g.clan_name
            ORDER BY g.clan_name ASC
            LIMIT $2 OFFSET $3
            "#,
            search,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn list_clans(&self) -> Result<Vec<RegisteredGuildModel>, anyhow::Error> {
        let rows = sqlx::query_as!(
            GuildRow,
            r#"
            SELECT id, guild_id, clan_name, broadcast_channel, clan_chat_channel,
                   leagues_broadcast_channel, drop_price_threshold,
                   disallowed_broadcast_types, verification_code, hashed_verification_code,
                   min_quest_difficulty, min_diary_tier, pk_value_threshold, wom_id, created_at,
                   custom_drop_broadcast_filter,
                   collection_log_max_percentage
            FROM guilds
            WHERE clan_name IS NOT NULL AND clan_name <> ''
            ORDER BY clan_name ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<RegisteredGuildModel>, sqlx::Error> {
        let row = sqlx::query_as!(
            GuildRow,
            r#"
            SELECT id, guild_id, clan_name, broadcast_channel, clan_chat_channel,
                   leagues_broadcast_channel, drop_price_threshold,
                   disallowed_broadcast_types, verification_code, hashed_verification_code,
                   min_quest_difficulty, min_diary_tier, pk_value_threshold, wom_id, created_at,
                   custom_drop_broadcast_filter,
                   collection_log_max_percentage
            FROM guilds
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|row| row.into()))
    }
}
