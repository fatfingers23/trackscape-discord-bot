use mongodb::bson::doc;
use mongodb::options::ClientOptions;
use mongodb::Database;
pub mod drop_logs_db;
pub mod guilds_db;

use crate::database::drop_logs_db::DropLogs;
use async_trait::async_trait;
use mockall::automock;

#[automock]
#[async_trait]
pub trait MongoDb {
    async fn new_db_instance(db_url: String) -> Self;
}

#[derive(Clone)]
pub struct BotMongoDb {
    pub guilds: GuildsDb,
    pub drop_logs: DropLogsDb,
}

#[derive(Clone)]
pub struct GuildsDb {
    db: Database,
}

#[derive(Clone)]
pub struct DropLogsDb {
    db: Database,
}

#[async_trait]
impl MongoDb for BotMongoDb {
    async fn new_db_instance(db_url: String) -> Self {
        let client_options = ClientOptions::parse(db_url.as_str())
            .await
            .expect("Could not connect to the mongo db");
        let client = mongodb::Client::with_options(client_options)
            .expect("Could not parse the mongod db url");

        let db = client.database("TrackScapeDB");
        Self {
            guilds: GuildsDb::new(db.clone()),
            drop_logs: DropLogsDb::new_instance(db.clone()),
        }
    }
}
