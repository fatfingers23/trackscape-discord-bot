use crate::database::GuildsDb;
use crate::helpers::hash_string;
use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::{
    BroadcastType, DiaryTier, QuestDifficulty,
};
use anyhow::Result;
use async_recursion::async_recursion;
use futures::TryStreamExt;
use mockall::predicate::*;
use mongodb::bson::{doc, DateTime};
use mongodb::options::FindOptions;
use mongodb::{bson, Database};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::string::ToString;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisteredGuildModel {
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,
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
    pub created_at: Option<DateTime>,
}

impl RegisteredGuildModel {
    pub const COLLECTION_NAME: &'static str = "guilds";
    pub fn new(guild_id: u64) -> Self {
        let verification_code = Self::generate_code();
        let hashed_verification_code = hash_string(verification_code.clone());
        Self {
            id: bson::oid::ObjectId::new(),
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
            created_at: DateTime::now().into(),
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

impl GuildsDb {
    pub fn new(mongodb: Database) -> Self {
        Self { db: mongodb }
    }

    pub async fn create_if_new_guild(&self, guild_id: u64) {
        let saved_guild_query = self.get_by_guild_id(guild_id).await;
        match saved_guild_query {
            Ok(saved_guild) => {
                if saved_guild.is_none() {
                    self.save_new_guild(guild_id).await;
                }
            }
            Err(_) => {}
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
                let collection = self.db.collection(RegisteredGuildModel::COLLECTION_NAME);
                collection
                    .insert_one(guild, None)
                    .await
                    .expect("Failed to insert document for a new guild.");
            }
            Err(_) => {}
        }
    }

    #[async_recursion]
    pub async fn recursive_check_for_unique_code(
        &self,
        code: String,
    ) -> Result<String, anyhow::Error> {
        let collection = self
            .db
            .collection::<RegisteredGuildModel>(RegisteredGuildModel::COLLECTION_NAME);
        let hashed_code = hash_string(code.clone());
        let filter = doc! {"hashed_verification_code": hashed_code};
        let result = collection
            .find_one(filter, None)
            .await
            .expect("Was an anyhow::Error checking for unique code.");
        match result {
            Some(_) => {
                let new_code = RegisteredGuildModel::generate_code();
                self.recursive_check_for_unique_code(new_code).await
            }
            None => Ok(code),
        }
    }

    pub async fn get_guild_by_code_and_clan_name(
        &self,
        code: String,
        clan_name: String,
    ) -> Result<Option<RegisteredGuildModel>, anyhow::Error> {
        let collection = self
            .db
            .collection::<RegisteredGuildModel>(RegisteredGuildModel::COLLECTION_NAME);
        let hashed_code = hash_string(code);
        let filter = doc! {"hashed_verification_code": hashed_code, "clan_name": clan_name};
        Ok(collection
            .find_one(filter, None)
            .await
            .expect("Failed to find document for the Discord guild."))
    }

    pub async fn get_guild_by_code(
        &self,
        code: String,
    ) -> Result<Option<RegisteredGuildModel>, anyhow::Error> {
        let collection = self
            .db
            .collection::<RegisteredGuildModel>(RegisteredGuildModel::COLLECTION_NAME);
        let hashed_code = hash_string(code);
        let filter = doc! {"hashed_verification_code": hashed_code};
        Ok(collection
            .find_one(filter, None)
            .await
            .expect("Failed to find document for the Discord guild."))
    }

    pub async fn get_by_guild_id(
        &self,
        id: u64,
    ) -> Result<Option<RegisteredGuildModel>, mongodb::error::Error> {
        let collection = self
            .db
            .collection::<RegisteredGuildModel>(RegisteredGuildModel::COLLECTION_NAME);
        let filter = doc! { "guild_id": bson::to_bson(&id).unwrap()};
        let result = collection.find_one(filter.clone(), None).await;
        return match result {
            Ok(possible_guild) => Ok(possible_guild),
            Err(e) => Err(e),
        };
    }

    pub async fn update_guild(&self, guild: RegisteredGuildModel) {
        let collection = self.db.collection(RegisteredGuildModel::COLLECTION_NAME);
        let filter = doc! { "guild_id": bson::to_bson(&guild.guild_id).unwrap()};
        collection
            .replace_one(filter, guild, None)
            .await
            .expect("Failed to update document for the Discord guild.");
    }

    pub async fn delete_guild(&self, guild_id: u64) {
        let collection = self
            .db
            .collection::<RegisteredGuildModel>(RegisteredGuildModel::COLLECTION_NAME);
        let filter = doc! { "guild_id": bson::to_bson(&guild_id).unwrap()};
        collection
            .delete_one(filter, None)
            .await
            .expect("Failed to delete document for the Discord guild.");
    }

    pub async fn list_clans(&self) -> Result<Vec<RegisteredGuildModel>, anyhow::Error> {
        let collection = self
            .db
            .collection::<RegisteredGuildModel>(RegisteredGuildModel::COLLECTION_NAME);
        let opts = FindOptions::builder().sort(doc! { "clan_name": 1 }).build();
        let filter = doc! {"clan_name": {"$ne": ""}};
        let result = collection.find(filter, opts).await;

        return match result {
            Ok(cursor) => Ok(cursor.try_collect().await.unwrap()),
            Err(e) => Err(anyhow::Error::new(e)),
        };
    }

    pub async fn get_by_id(
        &self,
        id: bson::oid::ObjectId,
    ) -> Result<Option<RegisteredGuildModel>, mongodb::error::Error> {
        let collection = self
            .db
            .collection::<RegisteredGuildModel>(RegisteredGuildModel::COLLECTION_NAME);
        let filter = doc! { "_id": bson::to_bson(&id).unwrap()};
        let result = collection.find_one(filter.clone(), None).await;
        return match result {
            Ok(possible_guild) => Ok(possible_guild),
            Err(e) => Err(e),
        };
    }
}
