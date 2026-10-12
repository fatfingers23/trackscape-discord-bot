//! Integration tests for the Postgres data layer. Each test runs against its
//! own throwaway database with the real migrations applied
//! (`#[sqlx::test]` picks up `trackscape-discord-shared/migrations`).

use chrono::Utc;
use sqlx::PgPool;
use trackscape_discord_shared::database::BotDb;
use trackscape_discord_shared::database::clan_mate_collection_log_totals::ClanMateCollectionLogTotals;
use trackscape_discord_shared::database::clan_mates::ClanMates;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;

async fn setup(pool: PgPool) -> BotDb {
    BotDb::new(pool)
}

#[sqlx::test]
async fn guild_roundtrip_and_update(pool: PgPool) {
    let db = setup(pool).await;

    db.guilds.save_new_guild(123).await;
    let guild = db
        .guilds
        .get_by_guild_id(123)
        .await
        .unwrap()
        .expect("guild was saved");

    assert_eq!(guild.guild_id, 123);
    assert_eq!(guild.clan_name, None);
    assert_eq!(guild.disallowed_broadcast_types, Vec::new());
    // The generated code is a 9 digit dashed string and hashes to the stored hash.
    assert_eq!(guild.verification_code.len(), 11);
    assert!(!guild.hashed_verification_code.is_empty());
    assert!(guild.created_at.is_some());

    // Update the whole model and make sure it persists.
    let mut updated = guild.clone();
    updated.clan_name = Some("Test Clan".to_string());
    updated.broadcast_channel = Some(456);
    updated
        .disallowed_broadcast_types
        .push(BroadcastType::PetDrop);
    db.guilds.update_guild(updated).await;

    let refetched = db.guilds.get_by_guild_id(123).await.unwrap().unwrap();
    assert_eq!(refetched.clan_name, Some("Test Clan".to_string()));
    assert_eq!(refetched.broadcast_channel, Some(456));
    assert_eq!(
        refetched.disallowed_broadcast_types,
        vec![BroadcastType::PetDrop]
    );

    // list_clans only includes guilds with a non-empty clan name.
    db.guilds.save_new_guild(124).await;
    let clans = db.guilds.list_clans().await.unwrap();
    let names: Vec<String> = clans.iter().filter_map(|g| g.clan_name.clone()).collect();
    assert!(names.contains(&"Test Clan".to_string()));
    assert_eq!(clans.len(), 1);

    // get_by_id and delete work.
    let by_id = db.guilds.get_by_id(guild.id).await.unwrap().unwrap();
    assert_eq!(by_id.guild_id, 123);
    db.guilds.delete_guild(123).await;
    assert!(db.guilds.get_by_guild_id(123).await.unwrap().is_none());
}

#[sqlx::test]
async fn verification_codes_stay_unique_and_findable(pool: PgPool) {
    let db = setup(pool).await;

    db.guilds.save_new_guild(1).await;
    db.guilds.save_new_guild(2).await;

    let first = db.guilds.get_by_guild_id(1).await.unwrap().unwrap();
    let second = db.guilds.get_by_guild_id(2).await.unwrap().unwrap();
    assert_ne!(first.verification_code, second.verification_code);

    // A verification code resolves back to its guild.
    let found = db
        .guilds
        .get_guild_by_code(first.verification_code.clone())
        .await
        .unwrap()
        .expect("code should find the guild");
    assert_eq!(found.guild_id, 1);

    // Resetting produces a new, still-findable code.
    let new_code = db.guilds.reset_verification_code(1).await.unwrap();
    assert_ne!(new_code, first.verification_code);
    let refound = db
        .guilds
        .get_guild_by_code(new_code)
        .await
        .unwrap()
        .expect("new code should find the guild");
    assert_eq!(refound.guild_id, 1);
}

#[sqlx::test]
async fn clan_mate_find_or_create_and_duplicate_rejected(pool: PgPool) {
    let db = setup(pool).await;
    db.guilds.save_new_guild(123).await;

    let first = db
        .clan_mates
        .find_or_create_clan_mate(123, "Test Player".to_string())
        .await
        .unwrap();
    assert_eq!(first.guild_id, 123);
    // Spaces are normalized to NBSP like the Mongo data was.
    assert_eq!(first.player_name, "Test\u{a0}Player");

    // Second find_or_create returns the same row, not a duplicate.
    let second = db
        .clan_mates
        .find_or_create_clan_mate(123, "Test Player".to_string())
        .await
        .unwrap();
    assert_eq!(first.id, second.id);

    // A direct create of the same (guild, player_name) violates the unique
    // constraint, which the old Mongo layer silently allowed.
    let duplicate = db
        .clan_mates
        .create_new_clan_mate(123, "Test Player".to_string(), None)
        .await;
    assert!(duplicate.is_err());

    // find_by_current_name matches the normalized name.
    let found = db
        .clan_mates
        .find_by_current_name(123, "Test Player".to_string())
        .await
        .unwrap()
        .expect("player should be found");
    assert_eq!(found.id, first.id);

    // The player was discovered via chat, is not in the WOM group and has no
    // recorded chat sighting, so it is not counted as an active member yet.
    assert_eq!(db.clan_mates.get_clan_member_count(123).await.unwrap(), 0);

    // change_name moves the old name into previous_names and returns the
    // updated model.
    let changed = db
        .clan_mates
        .change_name(123, "Test Player".to_string(), "New Name".to_string())
        .await
        .unwrap();
    assert_eq!(changed.player_name, "New\u{a0}Name");
    assert!(
        changed
            .previous_names
            .contains(&"Test\u{a0}Player".to_string())
    );
    let renamed = db
        .clan_mates
        .find_by_current_name(123, "New Name".to_string())
        .await
        .unwrap()
        .expect("renamed player should be found");
    assert!(
        renamed
            .previous_names
            .contains(&"Test\u{a0}Player".to_string())
    );
}

#[sqlx::test]
async fn remove_clan_mate_cascades_to_pb_records_and_clog_totals(pool: PgPool) {
    let db = setup(pool).await;
    db.guilds.save_new_guild(123).await;

    let clan_mate = db
        .clan_mates
        .find_or_create_clan_mate(123, "Cascade Player".to_string())
        .await
        .unwrap();
    let activity = db
        .pb_activities
        .create_or_get_activity("Zulrah".to_string())
        .await
        .unwrap();

    db.pb_records
        .create_or_update_pb_record(clan_mate.id, activity.id, 123, 100.0)
        .await
        .unwrap();
    db.clan_mate_collection_log_totals
        .update_or_create(123, clan_mate.id, 500)
        .await
        .unwrap();

    db.clan_mates
        .remove_clan_mate(123, "Cascade Player".to_string())
        .await
        .unwrap();

    // The PB records and collection log totals were removed by the foreign
    // key cascades (the old Mongo layer orphaned these rows).
    let pb_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM personal_best_records")
        .fetch_one(db.pool())
        .await
        .unwrap();
    let clog_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM clan_mate_collection_log_totals")
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(pb_count, 0);
    assert_eq!(clog_count, 0);
}

#[sqlx::test]
async fn pb_record_only_updates_when_better_and_leaderboard_orders(pool: PgPool) {
    let db = setup(pool).await;
    db.guilds.save_new_guild(123).await;

    let slow_player = db
        .clan_mates
        .find_or_create_clan_mate(123, "Slow Player".to_string())
        .await
        .unwrap();
    let fast_player = db
        .clan_mates
        .find_or_create_clan_mate(123, "Fast Player".to_string())
        .await
        .unwrap();
    let activity = db
        .pb_activities
        .create_or_get_activity("Zulrah".to_string())
        .await
        .unwrap();

    // First records insert.
    db.pb_records
        .create_or_update_pb_record(slow_player.id, activity.id, 123, 100.0)
        .await
        .unwrap();
    db.pb_records
        .create_or_update_pb_record(fast_player.id, activity.id, 123, 80.0)
        .await
        .unwrap();

    // A worse time does not overwrite; a better one does.
    db.pb_records
        .create_or_update_pb_record(slow_player.id, activity.id, 123, 200.0)
        .await
        .unwrap();
    db.pb_records
        .create_or_update_pb_record(slow_player.id, activity.id, 123, 90.0)
        .await
        .unwrap();

    let leaderboard = db
        .pb_records
        .get_pb_records_leaderboard(activity.id, 123)
        .await
        .unwrap();
    assert_eq!(leaderboard.len(), 2);
    // Ordered by time ascending, and each record carries its clan mate.
    assert_eq!(
        leaderboard[0].clan_mate.as_ref().unwrap().id,
        fast_player.id
    );
    assert_eq!(leaderboard[0].time_in_seconds, 80.0);
    assert_eq!(
        leaderboard[1].clan_mate.as_ref().unwrap().id,
        slow_player.id
    );
    assert_eq!(leaderboard[1].time_in_seconds, 90.0);
}

#[sqlx::test]
async fn clog_totals_upsert_and_descending_order(pool: PgPool) {
    let db = setup(pool).await;
    db.guilds.save_new_guild(123).await;

    let player_one = db
        .clan_mates
        .find_or_create_clan_mate(123, "Player One".to_string())
        .await
        .unwrap();
    let player_two = db
        .clan_mates
        .find_or_create_clan_mate(123, "Player Two".to_string())
        .await
        .unwrap();

    // Upserting the same (guild, player) overwrites the total.
    db.clan_mate_collection_log_totals
        .update_or_create(123, player_one.id, 100)
        .await
        .unwrap();
    db.clan_mate_collection_log_totals
        .update_or_create(123, player_one.id, 250)
        .await
        .unwrap();
    db.clan_mate_collection_log_totals
        .update_or_create(123, player_two.id, 300)
        .await
        .unwrap();

    let totals = db
        .clan_mate_collection_log_totals
        .get_guild_totals(123)
        .await
        .unwrap();
    assert_eq!(totals.len(), 2);
    // Descending by total, joined clan mates included.
    assert_eq!(totals[0].total, 300);
    assert_eq!(totals[0].clan_mate.as_ref().unwrap().id, player_two.id);
    assert_eq!(totals[1].total, 250);
    assert_eq!(totals[1].clan_mate.as_ref().unwrap().id, player_one.id);
}

#[sqlx::test]
async fn activities_are_case_insensitively_unique(pool: PgPool) {
    let db = setup(pool).await;

    let first = db
        .pb_activities
        .create_or_get_activity("Zulrah".to_string())
        .await
        .unwrap();
    // Different case and surrounding whitespace resolve to the same row.
    let second = db
        .pb_activities
        .create_or_get_activity("  zulRAH ".to_string())
        .await
        .unwrap();
    assert_eq!(first.id, second.id);

    let third = db
        .pb_activities
        .create_or_get_activity("Vorkath".to_string())
        .await
        .unwrap();
    assert_ne!(first.id, third.id);

    let activities = db.pb_activities.get_activities().await.unwrap();
    assert_eq!(activities.len(), 2);
    // Activities keep their `_id` JSON key (the dashboard reads it).
    let json = serde_json::to_value(&activities[0]).unwrap();
    assert!(json.get("_id").is_some());
}

#[sqlx::test]
async fn find_by_current_name_is_guild_scoped(pool: PgPool) {
    // find_by_current_name is scoped to a single guild so players with the
    // same name in different clans are not misattributed.
    let db = setup(pool).await;
    db.guilds.save_new_guild(123).await;
    db.guilds.save_new_guild(456).await;

    let player = db
        .clan_mates
        .find_or_create_clan_mate(123, "Shared Name".to_string())
        .await
        .unwrap();

    let found = db
        .clan_mates
        .find_by_current_name(123, "Shared Name".to_string())
        .await
        .unwrap()
        .expect("name should be found in its own guild");
    assert_eq!(found.id, player.id);
    assert_eq!(found.guild_id, 123);

    // The same name in another guild is not found...
    let not_found = db
        .clan_mates
        .find_by_current_name(456, "Shared Name".to_string())
        .await
        .unwrap();
    assert!(not_found.is_none());

    // ...and find_or_create makes a separate row for the other guild.
    let other = db
        .clan_mates
        .find_or_create_clan_mate(456, "Shared Name".to_string())
        .await
        .unwrap();
    assert_ne!(other.id, player.id);
    assert_eq!(other.guild_id, 456);
}

#[sqlx::test]
async fn clan_member_count_counts_active_members(pool: PgPool) {
    let db = setup(pool).await;
    db.guilds.save_new_guild(123).await;

    let in_wom_player = db
        .clan_mates
        .find_or_create_clan_mate(123, "In Wom".to_string())
        .await
        .unwrap();
    let gone_player = db
        .clan_mates
        .find_or_create_clan_mate(123, "Gone".to_string())
        .await
        .unwrap();
    let recent_chat_player = db
        .clan_mates
        .find_or_create_clan_mate(123, "Recent Chat".to_string())
        .await
        .unwrap();
    let stale_chat_player = db
        .clan_mates
        .find_or_create_clan_mate(123, "Stale Chat".to_string())
        .await
        .unwrap();

    // In the WOM group: counted regardless of chat activity.
    let mut model = in_wom_player.clone();
    model.in_wom = true;
    db.clan_mates.update_clan_mate(model).await.unwrap();

    // Left the WOM group and never seen in chat: not counted.
    let mut model = gone_player.clone();
    model.in_wom = false;
    model.last_seen_in_chat_at = None;
    db.clan_mates.update_clan_mate(model).await.unwrap();

    // Not in the WOM group but seen in chat recently: counted.
    let mut model = recent_chat_player.clone();
    model.in_wom = false;
    model.last_seen_in_chat_at = Some(Utc::now() - chrono::Duration::days(10));
    db.clan_mates.update_clan_mate(model).await.unwrap();

    // Not in the WOM group and last seen over the 90 day window: not counted.
    let mut model = stale_chat_player.clone();
    model.in_wom = false;
    model.last_seen_in_chat_at = Some(Utc::now() - chrono::Duration::days(91));
    db.clan_mates.update_clan_mate(model).await.unwrap();

    assert_eq!(db.clan_mates.get_clan_member_count(123).await.unwrap(), 2);
}

#[sqlx::test]
async fn touch_last_seen_updates_timestamp(pool: PgPool) {
    let db = setup(pool).await;
    db.guilds.save_new_guild(123).await;

    let player = db
        .clan_mates
        .find_or_create_clan_mate(123, "Chat Player".to_string())
        .await
        .unwrap();
    assert!(player.last_seen_in_chat_at.is_none());

    db.clan_mates
        .touch_last_seen_in_chat(player.id)
        .await
        .unwrap();

    let refetched = db
        .clan_mates
        .find_by_current_name(123, "Chat Player".to_string())
        .await
        .unwrap()
        .expect("player should be found");
    let touched = refetched
        .last_seen_in_chat_at
        .expect("last_seen_in_chat_at should be set");
    assert!(touched > Utc::now() - chrono::Duration::minutes(1));
}
