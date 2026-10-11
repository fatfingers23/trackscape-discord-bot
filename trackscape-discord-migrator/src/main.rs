//! MongoDB -> Postgres migrator (safe to re-run).
//!
//! Reads every collection from the legacy Mongo database (MONGO_DB_URL,
//! database name MONGO_DB_NAME, defaults to "TrackScapeDB") and copies the
//! data into the Postgres database (DATABASE_URL), applying the unique
//! constraints the Mongo database never had:
//!
//! * guilds:     deduped on `guild_id` (first document wins)
//! * clan_mates: deduped on `(guild_id, player_name)` (first document wins);
//!               rows whose guild does not exist are skipped
//! * personal_best_activities: deduped case-insensitively on the name
//! * personal_best_records:     deduped on `(clan_mate, activity)`, keeping
//!               the best (lowest) time; rows referencing missing clan
//!               mates or activities are skipped
//! * clan_mate_collection_log_totals: deduped on `(guild, player)`, keeping
//!               the latest created_at
//! * drop_logs / broadcasts: copied as-is, skipping rows whose guild does
//!               not exist
//!
//! New UUIDs are derived deterministically from the old ObjectIds, so every
//! run maps the same old document to the same Postgres row. Child rows are
//! re-pointed at the winner of their dedupe group, so references survive the
//! dedupes.
//!
//! # Re-running
//!
//! The migrator is safe to run as many times as you want BEFORE the
//! Postgres-backed services go live:
//!
//! * Without `--truncate-first`: an incremental refresh. Existing rows are
//!   updated in place from Mongo (guild settings, ranks, totals, best
//!   times), new rows are inserted, and nothing is duplicated. Rows that
//!   were deleted from Mongo since the last run are NOT removed from
//!   Postgres.
//! * With `--truncate-first`: an exact mirror. All tables are emptied and
//!   repopulated from Mongo, so deletions in Mongo are reflected too.
//!
//! AFTER the cutover (once the Railway stack is live and taking writes)
//! Postgres is the source of truth: never run the migrator again, because
//! it would overwrite live data with stale Mongo data (`--truncate-first`
//! would even wipe live rows entirely).

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use log::{error, info, warn};
use mongodb::Database as MongoDatabase;
use mongodb::bson::oid::ObjectId;
use mongodb::options::ClientOptions;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use trackscape_discord_shared::database::enum_serde;
use trackscape_discord_shared::database::legacy_mongo::{
    LegacyBroadcast, LegacyClanMate, LegacyClogTotal, LegacyDropLog, LegacyGuild, LegacyPbActivity,
    LegacyPbRecord, collections,
};
use trackscape_discord_shared::database::run_migrations;
use uuid::Uuid;

const BATCH_SIZE: usize = 500;

/// Derives the Postgres row id deterministically from the old Mongo
/// ObjectId. Running the migrator again maps the same old document to the
/// same Postgres row, so re-runs update in place instead of duplicating.
fn uuid_from_object_id(object_id: &ObjectId) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, &object_id.bytes())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let truncate_first = std::env::args().any(|arg| arg == "--truncate-first");

    let mongo_url = std::env::var("MONGO_DB_URL").context("MONGO_DB_URL not set!")?;
    let mongo_db_name =
        std::env::var("MONGO_DB_NAME").unwrap_or_else(|_| "TrackScapeDB".to_string());
    let pg_url = std::env::var("DATABASE_URL").context("DATABASE_URL not set!")?;

    let mongo = connect_mongo(&mongo_url, &mongo_db_name).await?;
    let pg = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(pg_url.as_str())
        .await
        .context("Could not connect to Postgres")?;
    run_migrations(&pg)
        .await
        .context("Could not run Postgres migrations")?;

    if truncate_first {
        info!("Truncating all Postgres tables (--truncate-first)");
        // Runtime query on purpose: the migrator is a one-off tool and should
        // not need a database or .sqlx offline metadata at build time.
        sqlx::query(
            "TRUNCATE clan_mate_collection_log_totals, personal_best_records,
             personal_best_activities, broadcasts, drop_logs, clan_mates, guilds
             RESTART IDENTITY CASCADE",
        )
        .execute(&pg)
        .await?;
    }

    let mut report = MigrationReport::default();

    let guild_ids = migrate_guilds(&mongo, &pg, &mut report).await?;
    let clan_mate_id_map = migrate_clan_mates(&mongo, &pg, &guild_ids, &mut report).await?;
    let activity_id_map = migrate_pb_activities(&mongo, &pg, &mut report).await?;
    migrate_pb_records(
        &mongo,
        &pg,
        &clan_mate_id_map,
        &activity_id_map,
        &mut report,
    )
    .await?;
    migrate_clog_totals(&mongo, &pg, &clan_mate_id_map, &mut report).await?;
    migrate_drop_logs(&mongo, &pg, &guild_ids, &mut report).await?;
    migrate_broadcasts(&mongo, &pg, &guild_ids, &mut report).await?;

    verify_counts(&mongo, &pg, &report).await;

    info!("Migration finished");
    Ok(())
}

async fn connect_mongo(url: &str, db_name: &str) -> Result<MongoDatabase> {
    let client_options = ClientOptions::parse(url)
        .await
        .context("Could not parse the Mongo db url")?;
    let client = mongodb::Client::with_options(client_options)
        .context("Could not connect to the Mongo db")?;
    Ok(client.database(db_name))
}

#[derive(Default)]
struct MigrationReport {
    guilds: TableReport,
    clan_mates: TableReport,
    pb_activities: TableReport,
    pb_records: TableReport,
    clog_totals: TableReport,
    drop_logs: TableReport,
    broadcasts: TableReport,
}

#[derive(Default)]
struct TableReport {
    mongo_count: u64,
    deduped: u64,
    skipped: u64,
    kept: u64,
}

impl TableReport {
    fn note_inserted(&mut self) {
        self.mongo_count += 1;
        self.kept += 1;
    }
    fn note_deduped(&mut self) {
        self.mongo_count += 1;
        self.deduped += 1;
    }
    fn note_skipped(&mut self) {
        self.mongo_count += 1;
        self.skipped += 1;
    }
    fn expected_pg_count(&self) -> u64 {
        self.kept
    }
}

fn bson_time_to_chrono(value: mongodb::bson::DateTime) -> DateTime<Utc> {
    value.to_chrono()
}

/// Migrates guilds. Returns the set of migrated Discord guild ids.
/// Duplicate `guild_id` documents are deduped: the first document wins.
async fn migrate_guilds(
    mongo: &MongoDatabase,
    pg: &PgPool,
    report: &mut MigrationReport,
) -> Result<HashSet<i64>> {
    info!("Migrating guilds...");
    let collection = mongo.collection::<LegacyGuild>(collections::GUILDS);
    let mut cursor = collection.find(None, None).await?;

    struct GuildRow {
        id: Uuid,
        legacy: LegacyGuild,
    }

    let mut seen_guild_ids: HashSet<i64> = HashSet::new();
    let mut rows: Vec<GuildRow> = Vec::new();

    while let Some(guild) = cursor.try_next().await? {
        let guild_id = guild.guild_id as i64;
        let new_id = uuid_from_object_id(&guild.id);

        if seen_guild_ids.insert(guild_id) {
            rows.push(GuildRow {
                id: new_id,
                legacy: guild,
            });
            report.guilds.note_inserted();
        } else {
            // Duplicate guild: the first document's row wins, the rest are
            // dropped (child rows reference guilds by guild_id, not by uuid,
            // so no id remapping is needed here).
            warn!(
                "Duplicate guild_id {} deduped, first document wins",
                guild_id
            );
            report.guilds.note_deduped();
        }
    }

    for chunk in rows.chunks(BATCH_SIZE) {
        let mut builder = sqlx::QueryBuilder::new(
            r#"INSERT INTO guilds (id, guild_id, clan_name, broadcast_channel, clan_chat_channel,
                                   leagues_broadcast_channel, drop_price_threshold,
                                   disallowed_broadcast_types, verification_code,
                                   hashed_verification_code, min_quest_difficulty, min_diary_tier,
                                   pk_value_threshold, wom_id, created_at,
                                   custom_drop_broadcast_filter, collection_log_max_percentage)
               "#,
        );
        builder.push_values(chunk, |mut b, row| {
            let legacy = &row.legacy;
            let disallowed_broadcast_types =
                enum_serde::enum_vec_to_strings(&legacy.disallowed_broadcast_types);
            let custom_drop_broadcast_filter = legacy
                .custom_drop_broadcast_filter
                .as_ref()
                .map(|filter| sqlx::types::Json(enum_serde::custom_filter_to_json(filter)));
            b.push_bind(row.id)
                .push_bind(legacy.guild_id as i64)
                .push_bind(legacy.clan_name.clone())
                .push_bind(legacy.broadcast_channel.map(|c| c as i64))
                .push_bind(legacy.clan_chat_channel.map(|c| c as i64))
                .push_bind(legacy.leagues_broadcast_channel.map(|c| c as i64))
                .push_bind(legacy.drop_price_threshold)
                .push_bind(disallowed_broadcast_types)
                .push_bind(legacy.verification_code.clone())
                .push_bind(legacy.hashed_verification_code.clone())
                .push_bind(
                    legacy
                        .min_quest_difficulty
                        .as_ref()
                        .map(enum_serde::enum_to_string),
                )
                .push_bind(
                    legacy
                        .min_diary_tier
                        .as_ref()
                        .map(enum_serde::enum_to_string),
                )
                .push_bind(legacy.pk_value_threshold)
                .push_bind(legacy.wom_id)
                .push_bind(legacy.created_at.map(bson_time_to_chrono))
                .push_bind(custom_drop_broadcast_filter)
                .push_bind(legacy.collection_log_max_percentage);
        });
        // Upsert so re-runs refresh guild settings in place from Mongo.
        builder.push(
            r#" ON CONFLICT (guild_id) DO UPDATE SET
                clan_name = EXCLUDED.clan_name, broadcast_channel = EXCLUDED.broadcast_channel,
                clan_chat_channel = EXCLUDED.clan_chat_channel,
                leagues_broadcast_channel = EXCLUDED.leagues_broadcast_channel,
                drop_price_threshold = EXCLUDED.drop_price_threshold,
                disallowed_broadcast_types = EXCLUDED.disallowed_broadcast_types,
                verification_code = EXCLUDED.verification_code,
                hashed_verification_code = EXCLUDED.hashed_verification_code,
                min_quest_difficulty = EXCLUDED.min_quest_difficulty,
                min_diary_tier = EXCLUDED.min_diary_tier,
                pk_value_threshold = EXCLUDED.pk_value_threshold, wom_id = EXCLUDED.wom_id,
                created_at = EXCLUDED.created_at,
                custom_drop_broadcast_filter = EXCLUDED.custom_drop_broadcast_filter,
                collection_log_max_percentage = EXCLUDED.collection_log_max_percentage"#,
        );
        builder.build().execute(pg).await?;
    }
    info!(
        "Guilds done: {} read, {} kept, {} deduped",
        report.guilds.mongo_count, report.guilds.kept, report.guilds.deduped
    );
    Ok(seen_guild_ids)
}

/// Migrates clan mates, deduping on (guild_id, player_name) and skipping
/// rows whose guild was not migrated.
async fn migrate_clan_mates(
    mongo: &MongoDatabase,
    pg: &PgPool,
    guild_ids: &HashSet<i64>,
    report: &mut MigrationReport,
) -> Result<HashMap<ObjectId, Uuid>> {
    info!("Migrating clan mates...");
    let collection = mongo.collection::<LegacyClanMate>(collections::CLAN_MATES);
    let mut cursor = collection.find(None, None).await?;

    struct ClanMateRow {
        id: Uuid,
        legacy: LegacyClanMate,
    }

    let mut seen: HashSet<(i64, String)> = HashSet::new();
    let mut id_map: HashMap<ObjectId, Uuid> = HashMap::new();
    let mut rows: Vec<ClanMateRow> = Vec::new();

    while let Some(clan_mate) = cursor.try_next().await? {
        let guild_id = clan_mate.guild_id as i64;

        // The Postgres schema requires every clan mate to belong to a real
        // guild; old orphan rows cannot be migrated.
        if !guild_ids.contains(&guild_id) {
            warn!(
                "Skipping clan mate {} in unknown guild {}",
                clan_mate.player_name, guild_id
            );
            report.clan_mates.note_skipped();
            continue;
        }

        let new_id = uuid_from_object_id(&clan_mate.id);
        if seen.insert((guild_id, clan_mate.player_name.clone())) {
            id_map.insert(clan_mate.id, new_id);
            rows.push(ClanMateRow {
                id: new_id,
                legacy: clan_mate,
            });
            report.clan_mates.note_inserted();
        } else {
            let winner = rows
                .iter()
                .find(|row| {
                    row.legacy.guild_id as i64 == guild_id
                        && row.legacy.player_name == clan_mate.player_name
                })
                .map(|row| row.id);
            if let Some(winner_id) = winner {
                id_map.insert(clan_mate.id, winner_id);
            }
            report.clan_mates.note_deduped();
        }
    }

    for chunk in rows.chunks(BATCH_SIZE) {
        let mut builder = sqlx::QueryBuilder::new(
            r#"INSERT INTO clan_mates (id, guild_id, player_name, wom_player_id, previous_names, "rank", created_at)
               "#,
        );
        builder.push_values(chunk, |mut b, row| {
            let legacy = &row.legacy;
            b.push_bind(row.id)
                .push_bind(legacy.guild_id as i64)
                .push_bind(legacy.player_name.clone())
                .push_bind(legacy.wom_player_id.map(|id| id as i64))
                .push_bind(legacy.previous_names.clone())
                .push_bind(legacy.rank.clone())
                .push_bind(bson_time_to_chrono(legacy.created_at));
        });
        // Upsert keyed on the row's own id (stable, derived from the Mongo
        // ObjectId) so re-runs refresh ranks and name changes in place from
        // Mongo. Keying on the name would break when a player is renamed.
        builder.push(
            r#" ON CONFLICT (id) DO UPDATE SET
                guild_id = EXCLUDED.guild_id, player_name = EXCLUDED.player_name,
                wom_player_id = EXCLUDED.wom_player_id,
                previous_names = EXCLUDED.previous_names, "rank" = EXCLUDED."rank",
                created_at = EXCLUDED.created_at"#,
        );
        builder.build().execute(pg).await?;
    }
    info!(
        "Clan mates done: {} read, {} kept, {} deduped, {} skipped",
        report.clan_mates.mongo_count,
        report.clan_mates.kept,
        report.clan_mates.deduped,
        report.clan_mates.skipped
    );
    Ok(id_map)
}

/// Migrates personal best activities, deduping case-insensitively on the
/// name (the old Mongo code matched `^name$` case-insensitively).
async fn migrate_pb_activities(
    mongo: &MongoDatabase,
    pg: &PgPool,
    report: &mut MigrationReport,
) -> Result<HashMap<ObjectId, Uuid>> {
    info!("Migrating personal best activities...");
    let collection = mongo.collection::<LegacyPbActivity>(collections::PB_ACTIVITIES);
    let mut cursor = collection.find(None, None).await?;

    struct ActivityRow {
        id: Uuid,
        activity_name: String,
        created_at: DateTime<Utc>,
    }

    let mut seen: HashSet<String> = HashSet::new();
    let mut id_map: HashMap<ObjectId, Uuid> = HashMap::new();
    let mut rows: Vec<ActivityRow> = Vec::new();

    while let Some(activity) = cursor.try_next().await? {
        let trimmed_name = activity.activity_name.trim();
        let key = trimmed_name.to_lowercase();
        let new_id = uuid_from_object_id(&activity.id);
        if seen.insert(key) {
            id_map.insert(activity.id, new_id);
            rows.push(ActivityRow {
                id: new_id,
                activity_name: trimmed_name.to_string(),
                created_at: bson_time_to_chrono(activity.created_at),
            });
            report.pb_activities.note_inserted();
        } else {
            let winner = rows
                .iter()
                .find(|row| row.activity_name.to_lowercase() == trimmed_name.to_lowercase())
                .map(|row| row.id);
            if let Some(winner_id) = winner {
                id_map.insert(activity.id, winner_id);
            }
            report.pb_activities.note_deduped();
        }
    }

    for chunk in rows.chunks(BATCH_SIZE) {
        let mut builder = sqlx::QueryBuilder::new(
            "INSERT INTO personal_best_activities (id, activity_name, created_at) ",
        );
        builder.push_values(chunk, |mut b, row| {
            b.push_bind(row.id)
                .push_bind(row.activity_name.clone())
                .push_bind(row.created_at);
        });
        builder.push(" ON CONFLICT DO NOTHING");
        builder.build().execute(pg).await?;
    }
    info!(
        "Personal best activities done: {} read, {} kept, {} deduped",
        report.pb_activities.mongo_count, report.pb_activities.kept, report.pb_activities.deduped
    );
    Ok(id_map)
}

/// Migrates personal best records, keeping the best (lowest) time per
/// (clan mate, activity) and skipping rows with unknown references.
async fn migrate_pb_records(
    mongo: &MongoDatabase,
    pg: &PgPool,
    clan_mate_ids: &HashMap<ObjectId, Uuid>,
    activity_ids: &HashMap<ObjectId, Uuid>,
    report: &mut MigrationReport,
) -> Result<()> {
    info!("Migrating personal best records...");
    let collection = mongo.collection::<LegacyPbRecord>(collections::PB_RECORDS);
    let mut cursor = collection.find(None, None).await?;

    struct PbRecordRow {
        id: Uuid,
        clan_mate_id: Uuid,
        activity_id: Uuid,
        guild_id: i64,
        time_in_seconds: f64,
    }

    let mut best: HashMap<(Uuid, Uuid), PbRecordRow> = HashMap::new();

    while let Some(record) = cursor.try_next().await? {
        let Some(&clan_mate_id) = clan_mate_ids.get(&record.clan_mate_id) else {
            report.pb_records.note_skipped();
            continue;
        };
        let Some(&activity_id) = activity_ids.get(&record.activity_id) else {
            report.pb_records.note_skipped();
            continue;
        };
        let key = (clan_mate_id, activity_id);
        let candidate = PbRecordRow {
            id: uuid_from_object_id(&record.id),
            clan_mate_id,
            activity_id,
            guild_id: record.guild_id as i64,
            time_in_seconds: record.time_in_seconds,
        };
        match best.get_mut(&key) {
            // Keep the better (lower) time; ties keep the first seen.
            Some(existing) => {
                if candidate.time_in_seconds < existing.time_in_seconds {
                    *existing = candidate;
                }
                report.pb_records.note_deduped();
            }
            None => {
                best.insert(key, candidate);
                report.pb_records.note_inserted();
            }
        }
    }

    let rows: Vec<PbRecordRow> = best.into_values().collect();
    for chunk in rows.chunks(BATCH_SIZE) {
        let mut builder = sqlx::QueryBuilder::new(
            "INSERT INTO personal_best_records (id, clan_mate_id, activity_id, guild_id, time_in_seconds) ",
        );
        builder.push_values(chunk, |mut b, row| {
            b.push_bind(row.id)
                .push_bind(row.clan_mate_id)
                .push_bind(row.activity_id)
                .push_bind(row.guild_id)
                .push_bind(row.time_in_seconds);
        });
        // Upsert so re-runs refresh times when Mongo recorded a new best.
        builder.push(
            r#" ON CONFLICT (clan_mate_id, activity_id) DO UPDATE SET
                time_in_seconds = EXCLUDED.time_in_seconds, updated_at = now()"#,
        );
        builder.build().execute(pg).await?;
    }
    info!(
        "Personal best records done: {} read, {} kept, {} deduped, {} skipped",
        report.pb_records.mongo_count,
        report.pb_records.kept,
        report.pb_records.deduped,
        report.pb_records.skipped
    );
    Ok(())
}

/// Migrates collection log totals, keeping the latest created_at per
/// (guild, player) and skipping rows with unknown references.
async fn migrate_clog_totals(
    mongo: &MongoDatabase,
    pg: &PgPool,
    clan_mate_ids: &HashMap<ObjectId, Uuid>,
    report: &mut MigrationReport,
) -> Result<()> {
    info!("Migrating collection log totals...");
    let collection = mongo.collection::<LegacyClogTotal>(collections::CLOG_TOTALS);
    let mut cursor = collection.find(None, None).await?;

    struct ClogTotalRow {
        guild_id: i64,
        player_id: Uuid,
        total: i64,
        created_at: Option<DateTime<Utc>>,
    }

    let mut latest: HashMap<(i64, Uuid), ClogTotalRow> = HashMap::new();

    while let Some(total) = cursor.try_next().await? {
        let Some(&player_id) = clan_mate_ids.get(&total.player_id) else {
            report.clog_totals.note_skipped();
            continue;
        };
        let key = (total.guild_id as i64, player_id);
        let candidate = ClogTotalRow {
            guild_id: total.guild_id as i64,
            player_id,
            total: total.total,
            created_at: total.created_at.map(bson_time_to_chrono),
        };
        match latest.get_mut(&key) {
            Some(existing) => {
                // The sync overwrote totals, so the newest document wins.
                if candidate.created_at > existing.created_at {
                    *existing = candidate;
                }
                report.clog_totals.note_deduped();
            }
            None => {
                latest.insert(key, candidate);
                report.clog_totals.note_inserted();
            }
        }
    }

    let rows: Vec<ClogTotalRow> = latest.into_values().collect();
    for chunk in rows.chunks(BATCH_SIZE) {
        let mut builder = sqlx::QueryBuilder::new(
            "INSERT INTO clan_mate_collection_log_totals (guild_id, player_id, total, created_at) ",
        );
        builder.push_values(chunk, |mut b, row| {
            b.push_bind(row.guild_id)
                .push_bind(row.player_id)
                .push_bind(row.total)
                .push_bind(row.created_at);
        });
        // Upsert so re-runs refresh totals when players gained slots.
        builder.push(
            " ON CONFLICT (guild_id, player_id) DO UPDATE SET
                total = EXCLUDED.total, created_at = EXCLUDED.created_at",
        );
        builder.build().execute(pg).await?;
    }
    info!(
        "Collection log totals done: {} read, {} kept, {} deduped, {} skipped",
        report.clog_totals.mongo_count,
        report.clog_totals.kept,
        report.clog_totals.deduped,
        report.clog_totals.skipped
    );
    Ok(())
}

/// A guild-scoped jsonb row (drop logs and broadcasts share this shape).
struct GuildJsonRow {
    id: Uuid,
    guild_id: i64,
    payload: sqlx::types::Json<serde_json::Value>,
    created_at: DateTime<Utc>,
}

/// Flushes a batch of guild-scoped jsonb rows into the given table and
/// jsonb column. `table` and `column` are only ever the compile-time
/// constants passed by this file. The ids are derived from the old Mongo
/// ObjectIds, so re-runs skip rows that were already copied.
async fn flush_guild_json_batch(
    pg: &PgPool,
    table: &'static str,
    column: &'static str,
    batch: &[GuildJsonRow],
) -> Result<()> {
    let sql = match (table, column) {
        ("drop_logs", "drop_item") => {
            "INSERT INTO drop_logs (id, guild_id, drop_item, created_at) "
        }
        ("broadcasts", "broadcast") => {
            "INSERT INTO broadcasts (id, guild_id, broadcast, created_at) "
        }
        _ => unreachable!("unknown table/column pair {}.{}", table, column),
    };
    let mut builder = sqlx::QueryBuilder::new(sql);
    builder.push_values(batch, |mut b, row| {
        b.push_bind(row.id)
            .push_bind(row.guild_id)
            .push_bind(&row.payload)
            .push_bind(row.created_at);
    });
    builder.push(" ON CONFLICT (id) DO NOTHING");
    builder.build().execute(pg).await?;
    Ok(())
}

/// Migrates drop logs (embedded documents become jsonb), skipping rows whose
/// guild was not migrated.
async fn migrate_drop_logs(
    mongo: &MongoDatabase,
    pg: &PgPool,
    guild_ids: &HashSet<i64>,
    report: &mut MigrationReport,
) -> Result<()> {
    info!("Migrating drop logs...");
    let collection = mongo.collection::<LegacyDropLog>(collections::DROP_LOGS);
    let mut cursor = collection.find(None, None).await?;

    let mut batch: Vec<GuildJsonRow> = Vec::new();

    while let Some(drop_log) = cursor.try_next().await? {
        let guild_id = drop_log.guild_id as i64;
        if !guild_ids.contains(&guild_id) {
            report.drop_logs.note_skipped();
            continue;
        }
        report.drop_logs.note_inserted();
        batch.push(GuildJsonRow {
            id: uuid_from_object_id(&drop_log.id),
            guild_id,
            payload: sqlx::types::Json(serde_json::to_value(&drop_log.drop_item)?),
            created_at: bson_time_to_chrono(drop_log.created_at),
        });
        if batch.len() >= BATCH_SIZE {
            flush_guild_json_batch(pg, "drop_logs", "drop_item", &batch).await?;
            batch.clear();
        }
    }

    if !batch.is_empty() {
        flush_guild_json_batch(pg, "drop_logs", "drop_item", &batch).await?;
    }
    info!(
        "Drop logs done: {} read, {} kept, {} skipped",
        report.drop_logs.mongo_count, report.drop_logs.kept, report.drop_logs.skipped
    );
    Ok(())
}

/// Migrates broadcasts, skipping rows whose guild was not migrated.
async fn migrate_broadcasts(
    mongo: &MongoDatabase,
    pg: &PgPool,
    guild_ids: &HashSet<i64>,
    report: &mut MigrationReport,
) -> Result<()> {
    info!("Migrating broadcasts...");
    let collection = mongo.collection::<LegacyBroadcast>(collections::BROADCASTS);
    let mut cursor = collection.find(None, None).await?;

    let mut batch: Vec<GuildJsonRow> = Vec::new();

    while let Some(broadcast) = cursor.try_next().await? {
        let guild_id = broadcast.guild_id as i64;
        if !guild_ids.contains(&guild_id) {
            report.broadcasts.note_skipped();
            continue;
        }
        report.broadcasts.note_inserted();
        batch.push(GuildJsonRow {
            id: uuid_from_object_id(&broadcast.id),
            guild_id,
            payload: sqlx::types::Json(serde_json::to_value(&broadcast.broadcast)?),
            created_at: broadcast.created_at.to_chrono(),
        });
        if batch.len() >= BATCH_SIZE {
            flush_guild_json_batch(pg, "broadcasts", "broadcast", &batch).await?;
            batch.clear();
        }
    }

    if !batch.is_empty() {
        flush_guild_json_batch(pg, "broadcasts", "broadcast", &batch).await?;
    }
    info!(
        "Broadcasts done: {} read, {} kept, {} skipped",
        report.broadcasts.mongo_count, report.broadcasts.kept, report.broadcasts.skipped
    );
    Ok(())
}

/// Prints the final count comparison between Mongo and Postgres.
async fn verify_counts(mongo: &MongoDatabase, pg: &PgPool, report: &MigrationReport) {
    info!("Verification:");
    let checks: [(&str, &str, &TableReport); 7] = [
        ("guilds", collections::GUILDS, &report.guilds),
        ("clan_mates", collections::CLAN_MATES, &report.clan_mates),
        (
            "personal_best_activities",
            collections::PB_ACTIVITIES,
            &report.pb_activities,
        ),
        (
            "personal_best_records",
            collections::PB_RECORDS,
            &report.pb_records,
        ),
        (
            "clan_mate_collection_log_totals",
            collections::CLOG_TOTALS,
            &report.clog_totals,
        ),
        ("drop_logs", collections::DROP_LOGS, &report.drop_logs),
        ("broadcasts", collections::BROADCASTS, &report.broadcasts),
    ];
    for (table, collection_name, table_report) in checks {
        let collection = mongo.collection::<mongodb::bson::Document>(collection_name);
        let mongo_count = collection
            .count_documents(None, None)
            .await
            .unwrap_or_else(|e| {
                error!("Could not count Mongo collection {}: {:?}", table, e);
                0
            });
        let pg_count = postgres_counts(pg).await.get(table).copied().unwrap_or(-1);
        if pg_count as u64 == table_report.expected_pg_count() {
            info!(
                "  {:<32} mongo: {:>8}  pg: {:>8}  deduped: {:>5} skipped: {:>5}  OK",
                table, mongo_count, pg_count, table_report.deduped, table_report.skipped
            );
        } else {
            warn!(
                "  {:<32} mongo: {:>8}  pg: {:>8}  expected: {:>8} (deduped: {}, skipped: {})  MISMATCH",
                table,
                mongo_count,
                pg_count,
                table_report.expected_pg_count(),
                table_report.deduped,
                table_report.skipped
            );
        }
    }
}

/// Reads the Postgres row counts for every migrated table in one query.
/// (A static query string: sqlx refuses dynamically built strings.)
async fn postgres_counts(pg: &PgPool) -> HashMap<String, i64> {
    const COUNT_SQL: &str = "
        SELECT 'guilds' AS table_name, COUNT(*) AS count FROM guilds
        UNION ALL SELECT 'clan_mates', COUNT(*) FROM clan_mates
        UNION ALL SELECT 'personal_best_activities', COUNT(*) FROM personal_best_activities
        UNION ALL SELECT 'personal_best_records', COUNT(*) FROM personal_best_records
        UNION ALL SELECT 'clan_mate_collection_log_totals', COUNT(*) FROM clan_mate_collection_log_totals
        UNION ALL SELECT 'drop_logs', COUNT(*) FROM drop_logs
        UNION ALL SELECT 'broadcasts', COUNT(*) FROM broadcasts";
    sqlx::query_as::<_, (String, i64)>(COUNT_SQL)
        .fetch_all(pg)
        .await
        .unwrap_or_default()
        .into_iter()
        .collect()
}
