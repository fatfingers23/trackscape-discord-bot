use crate::database::ClanMatesDb;
use anyhow::Error;
use async_trait::async_trait;
use futures::TryStreamExt;
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
    pub rank: Option<String>,
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
            rank: None,
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

    async fn update_clan_mate(&self, model: ClanMateModel) -> Result<ClanMateModel, anyhow::Error>;

    async fn get_clan_member_count(&self, guild_id: u64) -> Result<u64, Error>;

    async fn get_clan_mates_by_guild_id(&self, guild_id: u64) -> Result<Vec<ClanMateModel>, Error>;

    async fn remove_clan_mate(&self, guild_id: u64, player_name: String) -> Result<(), Error>;

    async fn change_name(
        &self,
        guild_id: u64,
        old_name: String,
        new_name: String,
    ) -> Result<(), Error>;
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
        let collection = self
            .db
            .collection::<ClanMateModel>(ClanMateModel::COLLECTION_NAME);
        let clan_mate =
            ClanMateModel::new(guild_id, player_name.replace(" ", "\u{a0}"), wom_player_id);
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
            "player_name": bson::to_bson(&player_name.replace(" ", "\u{a0}")).unwrap(),
        };
        //TODO possible lower case look up
        // let agg = vec![doc! {
        //     "$match": doc!{
        //         "$expr": doc!{
        //             "$eq": doc!{
        //                 "$toLower": doc! {"player_name": bson::to_bson(&player_name.to_lowercase().replace(" ", "\u{a0}")).unwrap()}
        //             }
        //         }
        //     }
        // }];
        // let mut multi_results = collection.aggregate(agg, None).await?;
        // while let Some(result) = multi_results.try_next().await? {
        //     let doc = bson::from_document(result)?;
        //     // println!("* {}", doc);
        // }
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
            "previous_names": bson::to_bson(&player_name.replace(" ", "\u{a0}")).unwrap(),
        };
        let result = collection.find_one(filter, None).await?;
        Ok(result)
    }

    async fn update_clan_mate(&self, mut model: ClanMateModel) -> Result<ClanMateModel, Error> {
        let collection = self
            .db
            .collection::<ClanMateModel>(ClanMateModel::COLLECTION_NAME);
        model.player_name = model.player_name.replace(" ", "\u{a0}");
        let filter = doc! {
            "_id": bson::to_bson(&model.id).unwrap(),
        };
        let _ = collection.replace_one(filter, model.clone(), None).await?;
        Ok(model)
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

    async fn get_clan_mates_by_guild_id(&self, guild_id: u64) -> Result<Vec<ClanMateModel>, Error> {
        let collection = self
            .db
            .collection::<ClanMateModel>(ClanMateModel::COLLECTION_NAME);
        let filter = doc! {
            "guild_id": bson::to_bson(&guild_id).unwrap(),
        };
        let result = collection.find(filter, None).await?;
        let clan_mates = result.try_collect().await?;
        Ok(clan_mates)
    }

    async fn remove_clan_mate(&self, guild_id: u64, player_name: String) -> Result<(), Error> {
        //TODO add a bit to clean up other collections too
        let collection = self
            .db
            .collection::<ClanMateModel>(ClanMateModel::COLLECTION_NAME);
        let filter = doc! {
                "guild_id":bson::to_bson(&guild_id).unwrap(),
                "player_name": bson::to_bson(&player_name.replace(" ", "\u{a0}")).unwrap()
        };
        let result = collection.delete_one(filter, None).await?;
        if result.deleted_count == 0 {
            return Err(anyhow::anyhow!("Failed to remove clan mate"));
        }
        Ok(())
    }

    async fn change_name(
        &self,
        guild_id: u64,
        old_name: String,
        new_name: String,
    ) -> Result<(), Error> {
        let clan_mate = self.find_by_current_name(old_name.clone()).await?;
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
        self.update_clan_mate(clan_mate).await?;
        Ok(())
    }
}

pub fn name_compare(name1: &str, name2: &str) -> bool {
    name_normalize(name1) == name_normalize(name2)
}

pub fn name_normalize(name: &str) -> String {
    name.replace(" ", "\u{a0}").to_lowercase()
}
