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

    // The player was discovered via chat and is not marked as left or a
    // Guest, so it counts as a member.
    assert_eq!(db.clan_mates.get_clan_member_count(123).await.unwrap(), 1);

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
async fn soft_remove_keeps_history_and_drops_from_count(pool: PgPool) {
    let db = setup(pool).await;
    db.guilds.save_new_guild(123).await;

    let clan_mate = db
        .clan_mates
        .find_or_create_clan_mate(123, "Leaving Player".to_string())
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
    assert_eq!(db.clan_mates.get_clan_member_count(123).await.unwrap(), 1);

    // Casing and plain spaces in the request must not matter.
    let removed = db
        .clan_mates
        .soft_remove_clan_mate(123, "leaving player".to_string())
        .await
        .unwrap();
    assert_eq!(removed.id, clan_mate.id);
    assert!(removed.left_at.is_some());
    assert!(!removed.in_wom);

    // Row and history survive; the member just stops counting.
    assert_eq!(db.clan_mates.get_clan_member_count(123).await.unwrap(), 0);
    assert!(
        db.clan_mates
            .get_active_clan_mates_by_guild_id(123)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        db.clan_mates
            .get_clan_mates_by_guild_id(123)
            .await
            .unwrap()
            .len(),
        1
    );
    let pb_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM personal_best_records")
        .fetch_one(db.pool())
        .await
        .unwrap();
    let clog_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM clan_mate_collection_log_totals")
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(pb_count, 1);
    assert_eq!(clog_count, 1);

    // Removing an already-removed member is not an error.
    db.clan_mates
        .soft_remove_clan_mate(123, "Leaving Player".to_string())
        .await
        .unwrap();

    // Unknown names are an error so the bot can report it.
    assert!(
        db.clan_mates
            .soft_remove_clan_mate(123, "Nobody".to_string())
            .await
            .is_err()
    );
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
async fn clan_member_count_counts_members_not_marked_left(pool: PgPool) {
    let db = setup(pool).await;
    db.guilds.save_new_guild(123).await;

    // (name, rank, left_at set?, counted?)
    let fixture: [(&str, Option<&str>, bool, bool); 5] = [
        ("Quiet Member", Some("Sapphire"), false, true),
        ("Never Ranked", None, false, true),
        ("Left", Some("Ruby"), true, false),
        ("Guest Chat", Some("Guest"), false, false),
        ("Guest Left", Some("Guest"), true, false),
    ];
    for (name, rank, left, _) in fixture {
        let mut model = db
            .clan_mates
            .find_or_create_clan_mate(123, name.to_string())
            .await
            .unwrap();
        model.rank = rank.map(str::to_string);
        model.left_at = left.then(Utc::now);
        db.clan_mates.update_clan_mate(model).await.unwrap();
    }

    let expected = fixture.iter().filter(|(_, _, _, counted)| *counted).count() as u64;
    assert_eq!(
        db.clan_mates.get_clan_member_count(123).await.unwrap(),
        expected
    );
}

#[sqlx::test]
async fn active_clan_mates_match_member_count(pool: PgPool) {
    let db = setup(pool).await;
    db.guilds.save_new_guild(123).await;

    // (name, rank, left_at set?, in_wom, counted?) — in_wom must not matter.
    let fixture: [(&str, Option<&str>, bool, bool, bool); 6] = [
        ("In Wom", Some("Owner"), false, true, true),
        ("Not In Wom", Some("Sapphire"), false, false, true),
        ("Left In Wom", Some("Ruby"), true, true, false),
        ("Left", None, true, false, false),
        ("Guest Chat", Some("Guest"), false, false, false),
        ("Wom Guest", Some("Guest"), false, true, false),
    ];
    for (name, rank, left, in_wom, _) in fixture {
        let mut model = db
            .clan_mates
            .find_or_create_clan_mate(123, name.to_string())
            .await
            .unwrap();
        model.rank = rank.map(str::to_string);
        model.left_at = left.then(Utc::now);
        model.in_wom = in_wom;
        db.clan_mates.update_clan_mate(model).await.unwrap();
    }

    let active = db
        .clan_mates
        .get_active_clan_mates_by_guild_id(123)
        .await
        .unwrap();
    let mut active_names: Vec<String> = active.iter().map(|m| m.player_name.clone()).collect();
    active_names.sort();
    let mut expected: Vec<String> = fixture
        .iter()
        .filter(|(_, _, _, _, counted)| *counted)
        .map(|(name, _, _, _, _)| name.replace(' ', "\u{a0}"))
        .collect();
    expected.sort();
    assert_eq!(active_names, expected);

    assert_eq!(
        active.len() as u64,
        db.clan_mates.get_clan_member_count(123).await.unwrap()
    );

    // The unfiltered list the sync jobs depend on still returns every row.
    assert_eq!(
        db.clan_mates
            .get_clan_mates_by_guild_id(123)
            .await
            .unwrap()
            .len(),
        fixture.len()
    );
}

#[sqlx::test]
async fn list_clans_paginated_pages_searches_and_counts_members(pool: PgPool) {
    let db = setup(pool).await;

    // Three clans plus one guild without a clan name (which must not appear).
    for (guild_id, clan_name) in [(123, "Alpha"), (456, "Beta"), (789, "Gamma")] {
        db.guilds.save_new_guild(guild_id).await;
        let mut guild = db.guilds.get_by_guild_id(guild_id).await.unwrap().unwrap();
        guild.clan_name = Some(clan_name.to_string());
        db.guilds.update_guild(guild).await;
    }
    db.guilds.save_new_guild(999).await;

    // Member counts come back from the JOIN, using the same membership
    // predicate as get_clan_member_count: left_at IS NULL and not a Guest.
    for (name, rank, left) in [
        ("Member", Some("Sapphire"), false),
        ("Left", Some("Ruby"), true),
        ("Guest Chat", Some("Guest"), false),
    ] {
        let mut model = db
            .clan_mates
            .find_or_create_clan_mate(123, name.to_string())
            .await
            .unwrap();
        model.rank = rank.map(str::to_string);
        model.left_at = left.then(Utc::now);
        db.clan_mates.update_clan_mate(model).await.unwrap();
    }

    // Pages are ordered by clan name and respect the limit/offset.
    let first_page = db.guilds.list_clans_paginated(None, 2, 0).await.unwrap();
    let names: Vec<&str> = first_page.iter().map(|c| c.clan_name.as_str()).collect();
    assert_eq!(names, vec!["Alpha", "Beta"]);
    let second_page = db.guilds.list_clans_paginated(None, 2, 2).await.unwrap();
    let names: Vec<&str> = second_page.iter().map(|c| c.clan_name.as_str()).collect();
    assert_eq!(names, vec!["Gamma"]);

    // The total matches across pages.
    assert_eq!(db.guilds.count_clans(None).await.unwrap(), 3);

    // The JOIN only counts members (left_at IS NULL and not a Guest).
    assert_eq!(first_page[0].clan_name, "Alpha");
    assert_eq!(first_page[0].registered_members, 1);
    assert_eq!(first_page[1].registered_members, 0);

    // A case-insensitive search narrows both the page and the total.
    let hits = db
        .guilds
        .list_clans_paginated(Some("AMM"), 10, 0)
        .await
        .unwrap();
    let names: Vec<&str> = hits.iter().map(|c| c.clan_name.as_str()).collect();
    assert_eq!(names, vec!["Gamma"]);
    assert_eq!(db.guilds.count_clans(Some("AMM")).await.unwrap(), 1);

    // A search that matches nothing comes back empty with a zero total.
    assert_eq!(
        db.guilds
            .list_clans_paginated(Some("nope"), 10, 0)
            .await
            .unwrap()
            .len(),
        0
    );
    assert_eq!(db.guilds.count_clans(Some("nope")).await.unwrap(), 0);
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

#[sqlx::test]
async fn find_by_current_name_is_case_insensitive(pool: PgPool) {
    let db = setup(pool).await;
    db.guilds.save_new_guild(123).await;

    let player = db
        .clan_mates
        .find_or_create_clan_mate(123, "Vorrentz".to_string())
        .await
        .unwrap();

    for query in ["vorrentz", "VORRENTZ", "VoRrEnTz"] {
        let found = db
            .clan_mates
            .find_by_current_name(123, query.to_string())
            .await
            .unwrap()
            .unwrap_or_else(|| panic!("{query} should find the row"));
        assert_eq!(found.id, player.id);
        // The stored spelling is untouched.
        assert_eq!(found.player_name, "Vorrentz");
    }

    // find_or_create reuses the row instead of making a case-variant twin.
    let again = db
        .clan_mates
        .find_or_create_clan_mate(123, "VORRENTZ".to_string())
        .await
        .unwrap();
    assert_eq!(again.id, player.id);
    assert_eq!(
        db.clan_mates
            .get_clan_mates_by_guild_id(123)
            .await
            .unwrap()
            .len(),
        1
    );
}

#[sqlx::test]
async fn creating_a_case_only_duplicate_is_rejected(pool: PgPool) {
    let db = setup(pool).await;
    db.guilds.save_new_guild(123).await;
    db.clan_mates
        .create_new_clan_mate(123, "Vorrentz".to_string(), None)
        .await
        .unwrap();
    let err = db
        .clan_mates
        .create_new_clan_mate(123, "vorrentz".to_string(), None)
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("clan_mates_guild_id_lower_player_name_idx"),
        "{err}"
    );
}

#[sqlx::test]
async fn find_or_create_restores_a_left_member(pool: PgPool) {
    // A broadcast about a player (PB, clog, !pb) is proof they are in the
    // clan, so finding them clears left_at.
    let db = setup(pool).await;
    db.guilds.save_new_guild(123).await;

    let mut model = db
        .clan_mates
        .find_or_create_clan_mate(123, "Came Back".to_string())
        .await
        .unwrap();
    model.left_at = Some(Utc::now());
    db.clan_mates.update_clan_mate(model.clone()).await.unwrap();
    assert_eq!(db.clan_mates.get_clan_member_count(123).await.unwrap(), 0);

    let restored = db
        .clan_mates
        .find_or_create_clan_mate(123, "came back".to_string())
        .await
        .unwrap();
    assert_eq!(restored.id, model.id);
    assert_eq!(restored.left_at, None);
    assert_eq!(db.clan_mates.get_clan_member_count(123).await.unwrap(), 1);

    // A Guest row is restored too but still does not count: rank wins.
    let mut guest = db
        .clan_mates
        .find_or_create_clan_mate(123, "Visitor".to_string())
        .await
        .unwrap();
    guest.rank = Some("Guest".to_string());
    guest.left_at = Some(Utc::now());
    db.clan_mates.update_clan_mate(guest).await.unwrap();
    let guest = db
        .clan_mates
        .find_or_create_clan_mate(123, "Visitor".to_string())
        .await
        .unwrap();
    assert_eq!(guest.left_at, None);
    assert_eq!(db.clan_mates.get_clan_member_count(123).await.unwrap(), 1);
}

#[sqlx::test]
async fn change_name_refuses_a_name_held_by_another_row(pool: PgPool) {
    let db = setup(pool).await;
    db.guilds.save_new_guild(123).await;
    let first = db
        .clan_mates
        .find_or_create_clan_mate(123, "First".to_string())
        .await
        .unwrap();
    db.clan_mates
        .find_or_create_clan_mate(123, "Second".to_string())
        .await
        .unwrap();

    let err = db
        .clan_mates
        .change_name(123, "First".to_string(), "second".to_string())
        .await
        .unwrap_err();
    assert!(err.to_string().contains("already"), "{err}");

    // Changing only the casing of your own name is allowed.
    let recased = db
        .clan_mates
        .change_name(123, "First".to_string(), "FIRST".to_string())
        .await
        .unwrap();
    assert_eq!(recased.id, first.id);
    assert_eq!(recased.player_name, "FIRST");
}
