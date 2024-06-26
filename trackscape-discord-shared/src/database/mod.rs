use crate::database::clan_mate_collection_log_totals::ClanMateCollectionLogTotals;
use crate::database::clan_mates::ClanMates;
use crate::database::drop_logs_db::DropLogs;
use async_trait::async_trait;
use mockall::automock;
use mongodb::bson::doc;
use mongodb::options::ClientOptions;
use mongodb::Database;

pub mod broadcasts;
pub mod clan_mate_collection_log_totals;
pub mod clan_mates;
pub mod drop_logs_db;
pub mod guilds_db;
pub mod pb_activities_db;
pub mod pb_records_db;

#[automock]
#[async_trait]
pub trait MongoDb {
    async fn new_db_instance(db_url: String) -> Self;
}

#[derive(Clone)]
pub struct BotMongoDb {
    pub guilds: GuildsDb,
    pub drop_logs: DropLogsDb,
    pub clan_mates: ClanMatesDb,
    pub clan_mate_collection_log_totals: ClanMateCollectionLogTotalsDb,
    pub broadcasts: BroadcastsDb,
    pub pb_activities: PersonalBestActivitiesDb,
    pub pb_records: PersonalBestRecordsDb,
}

#[derive(Clone)]
pub struct GuildsDb {
    db: Database,
}

#[derive(Clone)]
pub struct DropLogsDb {
    db: Database,
}

#[derive(Clone)]
pub struct ClanMatesDb {
    db: Database,
}

#[derive(Clone)]
pub struct ClanMateCollectionLogTotalsDb {
    db: Database,
}

#[derive(Clone)]
pub struct BroadcastsDb {
    db: Database,
}

#[derive(Clone)]
pub struct PersonalBestActivitiesDb {
    db: Database,
}

#[derive(Clone)]
pub struct PersonalBestRecordsDb {
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
            clan_mates: ClanMatesDb::new_instance(db.clone()),
            clan_mate_collection_log_totals: ClanMateCollectionLogTotalsDb::new_instance(
                db.clone(),
            ),
            broadcasts: BroadcastsDb::new_instance(db.clone()),
            pb_activities: PersonalBestActivitiesDb::new_instance(db.clone()),
            pb_records: PersonalBestRecordsDb::new_instance(db),
        }
    }
}
