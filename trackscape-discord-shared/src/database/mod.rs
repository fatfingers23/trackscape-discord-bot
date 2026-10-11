use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub mod broadcasts;
pub mod clan_mate_collection_log_totals;
pub mod clan_mates;
pub mod drop_logs_db;
pub mod enum_serde;
pub mod guilds_db;
#[cfg(feature = "legacy-mongo")]
pub mod legacy_mongo;
pub mod pb_activities_db;
pub mod pb_records_db;

/// Embedded sqlx migrations (`trackscape-discord-shared/migrations/`).
/// Exported so tests and binaries can run them with `MIGRATOR.run(&pool)`.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

/// Runs the embedded migrations against the given pool. sqlx takes a
/// Postgres advisory lock while migrating, so it is safe for several
/// services to call this at startup at the same time.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}

/// The database facade used by every binary. Holds one cheap-to-clone
/// `PgPool` per sub-database (a `PgPool` is an `Arc` internally).
#[derive(Clone)]
pub struct BotDb {
    pub guilds: GuildsDb,
    pub drop_logs: DropLogsDb,
    pub clan_mates: ClanMatesDb,
    pub clan_mate_collection_log_totals: ClanMateCollectionLogTotalsDb,
    pub broadcasts: BroadcastsDb,
    pub pb_activities: PersonalBestActivitiesDb,
    pub pb_records: PersonalBestRecordsDb,
    pool: PgPool,
}

impl BotDb {
    /// Connects to Postgres and builds the facade. Pools are kept small on
    /// purpose: four services run against one database (api, bot, worker,
    /// cron worker), and Postgres only allows so many connections.
    pub async fn new_db_instance(db_url: String) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(db_url.as_str())
            .await?;
        Ok(Self::new(pool))
    }

    pub fn new(pool: PgPool) -> Self {
        Self {
            guilds: GuildsDb::new(pool.clone()),
            drop_logs: DropLogsDb::new(pool.clone()),
            clan_mates: ClanMatesDb::new(pool.clone()),
            clan_mate_collection_log_totals: ClanMateCollectionLogTotalsDb::new(pool.clone()),
            broadcasts: BroadcastsDb::new(pool.clone()),
            pb_activities: PersonalBestActivitiesDb::new(pool.clone()),
            pb_records: PersonalBestRecordsDb::new(pool.clone()),
            pool,
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[derive(Clone)]
pub struct GuildsDb {
    pool: PgPool,
}

#[derive(Clone)]
pub struct DropLogsDb {
    pool: PgPool,
}

#[derive(Clone)]
pub struct ClanMatesDb {
    pool: PgPool,
}

#[derive(Clone)]
pub struct ClanMateCollectionLogTotalsDb {
    pool: PgPool,
}

#[derive(Clone)]
pub struct BroadcastsDb {
    pool: PgPool,
}

#[derive(Clone)]
pub struct PersonalBestActivitiesDb {
    pool: PgPool,
}

#[derive(Clone)]
pub struct PersonalBestRecordsDb {
    pool: PgPool,
}
