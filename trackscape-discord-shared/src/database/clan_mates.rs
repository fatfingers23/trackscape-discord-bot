use crate::database::ClanMatesDb;
use anyhow::Error;
use async_trait::async_trait;
use mockall::predicate::*;
use mockall::*;
use mongodb::bson::{doc, DateTime};
use mongodb::{bson, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClanMateModel {
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,
    pub guild_id: u64,
    pub player_name: String,
    pub wom_player_id: Option<u64>,
    pub previous_names: Vec<String>,
    pub created_at: DateTime,
}

impl ClanMateModel {
    pub const COLLECTION_NAME: &'static str = "clan_mates";

    pub fn new(guild_id: u64, player_name: String, wom_player_id: Option<u64>) -> Self {
        Self {
            id: bson::oid::ObjectId::new(),
            guild_id,
            wom_player_id,
            previous_names: Vec::new(),
            player_name,
            created_at: DateTime::now(),
        }
    }
}

#[automock]
#[async_trait]
pub trait ClanMates {
    fn new_instance(mongodb: Database) -> Self;

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
        player_name: String,
    ) -> Result<Option<ClanMateModel>, anyhow::Error>;

    async fn find_by_previous_name(
        &self,
        player_name: String,
    ) -> Result<Option<ClanMateModel>, anyhow::Error>;

    async fn get_clan_member_count(&self, guild_id: u64) -> Result<u64, Error>;
}

#[async_trait]
impl ClanMates for ClanMatesDb {
    fn new_instance(mongodb: Database) -> Self {
        Self { db: mongodb }
    }

    async fn find_or_create_clan_mate(
        &self,
        guild_id: u64,
        player_name: String,
    ) -> Result<ClanMateModel, Error> {
        let possible_clan_mate = self.find_by_current_name(player_name.clone()).await?;
        return Ok(match possible_clan_mate {
            None => {
                self.create_new_clan_mate(guild_id, player_name, None)
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
        let collection = self
            .db
            .collection::<ClanMateModel>(ClanMateModel::COLLECTION_NAME);
        let clan_mate = ClanMateModel::new(guild_id, player_name, wom_player_id);
        let _ = collection.insert_one(clan_mate.clone(), None).await?;
        Ok(clan_mate)
    }

    async fn find_by_current_name(
        &self,
        player_name: String,
    ) -> Result<Option<ClanMateModel>, anyhow::Error> {
        let collection = self
            .db
            .collection::<ClanMateModel>(ClanMateModel::COLLECTION_NAME);
        let filter = doc! {
            "player_name": bson::to_bson(&player_name).unwrap(),
        };
        let result = collection.find_one(filter, None).await?;
        Ok(result)
    }

    async fn find_by_previous_name(
        &self,
        player_name: String,
    ) -> Result<Option<ClanMateModel>, anyhow::Error> {
        let collection = self
            .db
            .collection::<ClanMateModel>(ClanMateModel::COLLECTION_NAME);
        let filter = doc! {
            "previous_names": bson::to_bson(&player_name).unwrap(),
        };
        let result = collection.find_one(filter, None).await?;
        Ok(result)
    }

    async fn get_clan_member_count(&self, guild_id: u64) -> Result<u64, Error> {
        let collection = self
            .db
            .collection::<ClanMateModel>(ClanMateModel::COLLECTION_NAME);
        let filter = doc! {
            "guild_id": bson::to_bson(&guild_id).unwrap(),
        };
        let result = collection.count_documents(filter, None).await?;
        Ok(result)
    }
}
