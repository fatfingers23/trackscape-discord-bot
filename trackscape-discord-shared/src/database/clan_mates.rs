use crate::database::DropLogsDb;
use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::DropItemBroadcast;
use async_trait::async_trait;
use futures::TryStreamExt;
use mockall::predicate::*;
use mockall::*;
use mongodb::bson::{doc, DateTime};
use mongodb::{bson, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClanMateModel {
    pub guild_id: u64,
    pub created_at: DateTime,
}

impl ClanMateModel {
    pub const COLLECTION_NAME: &'static str = "clan_mates";

    pub fn new(guild_id: u64) -> Self {
        Self {
            guild_id,
            created_at: DateTime::now(),
        }
    }
}

#[automock]
#[async_trait]
pub trait ClanMates {
    fn new_instance(mongodb: Database) -> Self;
}

#[async_trait]
impl ClanMates for DropLogsDb {
    fn new_instance(mongodb: Database) -> Self {
        Self { db: mongodb }
    }
}
