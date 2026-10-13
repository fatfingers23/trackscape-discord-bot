//! Tests that run the real migration files against fixture data inserted
//! between migrations. `#[sqlx::test(migrations = false)]` gives a bare
//! database; the helpers below apply the embedded MIGRATOR entries by
//! version so a test can insert pre-migration rows and then apply the
//! migration under test.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::migrate::Migrate;
use trackscape_discord_shared::database::MIGRATOR;

/// Last migration before the membership-state change.
const PRE_MEMBERSHIP_VERSION: i64 = 20261012000000;

async fn apply_migrations_through(pool: &PgPool, max_version: i64) {
    let mut conn = pool.acquire().await.unwrap();
    conn.ensure_migrations_table(&MIGRATOR.table_name)
        .await
        .unwrap();
    for migration in MIGRATOR.iter().filter(|m| m.version <= max_version) {
        conn.apply(&MIGRATOR.table_name, migration).await.unwrap();
    }
}

async fn apply_migrations_after(pool: &PgPool, min_version: i64) {
    let mut conn = pool.acquire().await.unwrap();
    for migration in MIGRATOR.iter().filter(|m| m.version > min_version) {
        conn.apply(&MIGRATOR.table_name, migration).await.unwrap();
    }
}

async fn insert_guild(pool: &PgPool, guild_id: i64, wom_id: Option<i64>) {
    sqlx::query(
        "INSERT INTO guilds (id, guild_id, verification_code, hashed_verification_code, wom_id)
         VALUES (gen_random_uuid(), $1, $2, $3, $4)",
    )
    .bind(guild_id)
    .bind(format!("code-{guild_id}"))
    .bind(format!("hash-{guild_id}"))
    .bind(wom_id)
    .execute(pool)
    .await
    .unwrap();
}

async fn left_at_of(pool: &PgPool, name: &str) -> Option<DateTime<Utc>> {
    sqlx::query_scalar("SELECT left_at FROM clan_mates WHERE player_name = $1")
        .bind(name)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = false)]
async fn left_at_backfill_rules(pool: PgPool) {
    apply_migrations_through(&pool, PRE_MEMBERSHIP_VERSION).await;
    insert_guild(&pool, 1, Some(500)).await; // WOM-governed: has a confirmed member
    insert_guild(&pool, 2, Some(123456)).await; // wom_id the sync never confirmed
    insert_guild(&pool, 3, None).await; // no WOM group

    let earlier: DateTime<Utc> = "2026-09-01T00:00:00Z".parse().unwrap();

    // Guild 1, rule 1: unconfirmed rows are marked left unless they chatted
    // as a non-guest in the last 90 days.
    sqlx::query(
        "INSERT INTO clan_mates (id, guild_id, player_name, \"rank\", in_wom, wom_left_at, last_seen_in_chat_at) VALUES
         (gen_random_uuid(), 1, 'wom confirmed', NULL, true, NULL, NULL),
         (gen_random_uuid(), 1, 'wom unconfirmed', NULL, false, NULL, NULL),
         (gen_random_uuid(), 1, 'wom already left', NULL, false, $1, NULL),
         (gen_random_uuid(), 1, 'wom recent chatter', 'Sapphire', false, NULL, now() - interval '1 day'),
         (gen_random_uuid(), 1, 'wom stale chatter', 'Sapphire', false, NULL, now() - interval '120 days'),
         (gen_random_uuid(), 1, 'wom recent guest', 'Guest', false, NULL, now() - interval '1 day')",
    )
    .bind(earlier)
    .execute(&pool)
    .await
    .unwrap();

    // Guilds 2 and 3, rule 2: a row with no evidence in 180 days is marked
    // left. Evidence: created_at, last chat line, latest PB, latest clog.
    sqlx::query(
        "INSERT INTO clan_mates (id, guild_id, player_name, created_at, in_wom, wom_left_at, last_seen_in_chat_at) VALUES
         (gen_random_uuid(), 2, 'never synced fresh', now(), false, NULL, NULL),
         (gen_random_uuid(), 2, 'never synced stale', now() - interval '400 days', false, NULL, NULL),
         ('00000000-0000-0000-0000-000000000021', 2, 'never synced stale with pb', now() - interval '400 days', false, NULL, NULL),
         ('00000000-0000-0000-0000-000000000022', 2, 'never synced stale with clog', now() - interval '400 days', false, NULL, NULL),
         (gen_random_uuid(), 2, 'never synced stale recent chat', now() - interval '400 days', false, NULL, now() - interval '10 days'),
         (gen_random_uuid(), 3, 'no wom fresh', now() - interval '10 days', false, NULL, NULL),
         (gen_random_uuid(), 3, 'no wom stale', now() - interval '200 days', false, NULL, NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO personal_best_activities (id, activity_name) VALUES
         ('10000000-0000-0000-0000-000000000001', 'Zulrah')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO personal_best_records (id, clan_mate_id, activity_id, guild_id, time_in_seconds, created_at, updated_at) VALUES
         (gen_random_uuid(), '00000000-0000-0000-0000-000000000021', '10000000-0000-0000-0000-000000000001', 2, 100,
          now() - interval '400 days', now() - interval '10 days')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO clan_mate_collection_log_totals (guild_id, player_id, total, created_at) VALUES
         (2, '00000000-0000-0000-0000-000000000022', 500, now() - interval '10 days')",
    )
    .execute(&pool)
    .await
    .unwrap();

    apply_migrations_after(&pool, PRE_MEMBERSHIP_VERSION).await;

    // Rule 1 (guild 1).
    assert_eq!(left_at_of(&pool, "wom confirmed").await, None);
    assert!(left_at_of(&pool, "wom unconfirmed").await.is_some());
    assert_eq!(left_at_of(&pool, "wom already left").await, Some(earlier));
    assert_eq!(left_at_of(&pool, "wom recent chatter").await, None, "recent non-guest chat spares the row");
    assert!(left_at_of(&pool, "wom stale chatter").await.is_some());
    assert!(left_at_of(&pool, "wom recent guest").await.is_some(), "guests are not spared");

    // Rule 2 (guild 2: wom_id never confirmed).
    assert_eq!(left_at_of(&pool, "never synced fresh").await, None);
    assert!(left_at_of(&pool, "never synced stale").await.is_some());
    assert_eq!(left_at_of(&pool, "never synced stale with pb").await, None);
    assert_eq!(left_at_of(&pool, "never synced stale with clog").await, None);
    assert_eq!(left_at_of(&pool, "never synced stale recent chat").await, None);

    // Rule 2 (guild 3: no wom_id).
    assert_eq!(left_at_of(&pool, "no wom fresh").await, None);
    assert!(left_at_of(&pool, "no wom stale").await.is_some());
}

async fn count_where(pool: &PgPool, sql: &'static str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = false)]
async fn case_only_duplicates_are_merged_into_one_keeper(pool: PgPool) {
    apply_migrations_through(&pool, PRE_MEMBERSHIP_VERSION).await;
    insert_guild(&pool, 1, Some(500)).await;

    // Three rows for the same player differing only by case. The WOM-linked
    // row is the keeper even though it is neither the oldest nor the most
    // recently seen in chat.
    sqlx::query(
        "INSERT INTO clan_mates (id, guild_id, player_name, wom_player_id, previous_names, \"rank\", created_at, in_wom, wom_left_at, last_seen_in_chat_at) VALUES
         ('00000000-0000-0000-0000-000000000001', 1, 'Vorrentz', 77, '{\"Old Name\"}', NULL, '2026-03-01T00:00:00Z', true, NULL, NULL),
         ('00000000-0000-0000-0000-000000000002', 1, 'VORRENTZ', NULL, '{}', 'Ruby', '2026-01-01T00:00:00Z', false, '2026-09-01T00:00:00Z', '2026-10-01T00:00:00Z'),
         ('00000000-0000-0000-0000-000000000003', 1, 'vorrentz', NULL, '{\"Older Name\"}', NULL, '2026-02-01T00:00:00Z', false, NULL, NULL),
         ('00000000-0000-0000-0000-000000000004', 1, 'Someone Else', NULL, '{}', NULL, '2026-02-01T00:00:00Z', false, NULL, NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO personal_best_activities (id, activity_name) VALUES
         ('10000000-0000-0000-0000-000000000001', 'Zulrah'),
         ('10000000-0000-0000-0000-000000000002', 'Vorkath'),
         ('10000000-0000-0000-0000-000000000003', 'Inferno')",
    )
    .execute(&pool)
    .await
    .unwrap();
    // Zulrah: keeper 100s vs loser 80s -> keeper takes 80s.
    // Vorkath: only loser 2 has it -> moves.
    // Inferno: losers 2 and 3 both have it, keeper does not -> the better one
    //          (loser 3, 50s) must win without a unique violation.
    sqlx::query(
        "INSERT INTO personal_best_records (id, clan_mate_id, activity_id, guild_id, time_in_seconds) VALUES
         (gen_random_uuid(), '00000000-0000-0000-0000-000000000001', '10000000-0000-0000-0000-000000000001', 1, 100),
         (gen_random_uuid(), '00000000-0000-0000-0000-000000000002', '10000000-0000-0000-0000-000000000001', 1, 80),
         (gen_random_uuid(), '00000000-0000-0000-0000-000000000002', '10000000-0000-0000-0000-000000000002', 1, 200),
         (gen_random_uuid(), '00000000-0000-0000-0000-000000000002', '10000000-0000-0000-0000-000000000003', 1, 60),
         (gen_random_uuid(), '00000000-0000-0000-0000-000000000003', '10000000-0000-0000-0000-000000000003', 1, 50)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO clan_mate_collection_log_totals (guild_id, player_id, total) VALUES
         (1, '00000000-0000-0000-0000-000000000001', 500),
         (1, '00000000-0000-0000-0000-000000000002', 700)",
    )
    .execute(&pool)
    .await
    .unwrap();

    apply_migrations_after(&pool, PRE_MEMBERSHIP_VERSION).await;

    // Exactly one Vorrentz row survives: the WOM-linked keeper.
    assert_eq!(
        count_where(&pool, "SELECT COUNT(*) FROM clan_mates WHERE lower(player_name) = 'vorrentz'").await,
        1
    );
    let (name, wom_id, in_wom, left_at, last_seen, rank, previous): (
        String, Option<i64>, bool, Option<DateTime<Utc>>, Option<DateTime<Utc>>, Option<String>, Vec<String>,
    ) = sqlx::query_as(
        "SELECT player_name, wom_player_id, in_wom, left_at, last_seen_in_chat_at, \"rank\", previous_names
         FROM clan_mates WHERE id = '00000000-0000-0000-0000-000000000001'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(name, "Vorrentz");
    assert_eq!(wom_id, Some(77));
    assert!(in_wom);
    assert_eq!(left_at, None, "the keeper was a member in its own right");
    assert_eq!(last_seen, Some("2026-10-01T00:00:00Z".parse().unwrap()));
    assert_eq!(rank.as_deref(), Some("Ruby"));
    for expected in ["Old Name", "Older Name", "VORRENTZ", "vorrentz"] {
        assert!(previous.contains(&expected.to_string()), "missing {expected} in {previous:?}");
    }
    assert!(!previous.contains(&"Vorrentz".to_string()));

    // All PB records now point at the keeper, with the best time per activity.
    let pbs: Vec<(String, f64)> = sqlx::query_as(
        "SELECT a.activity_name, r.time_in_seconds FROM personal_best_records r
         JOIN personal_best_activities a ON a.id = r.activity_id
         WHERE r.clan_mate_id = '00000000-0000-0000-0000-000000000001' ORDER BY a.activity_name",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        pbs,
        vec![
            ("Inferno".to_string(), 50.0),
            ("Vorkath".to_string(), 200.0),
            ("Zulrah".to_string(), 80.0),
        ]
    );
    assert_eq!(count_where(&pool, "SELECT COUNT(*) FROM personal_best_records").await, 3);

    // Collection log keeps the higher total on the keeper.
    let total: i64 = sqlx::query_scalar(
        "SELECT total FROM clan_mate_collection_log_totals WHERE player_id = '00000000-0000-0000-0000-000000000001'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(total, 700);
    assert_eq!(count_where(&pool, "SELECT COUNT(*) FROM clan_mate_collection_log_totals").await, 1);

    // Unrelated rows are untouched.
    assert_eq!(count_where(&pool, "SELECT COUNT(*) FROM clan_mates").await, 2);

    // A new case-only duplicate is now rejected.
    let err = sqlx::query(
        "INSERT INTO clan_mates (id, guild_id, player_name) VALUES (gen_random_uuid(), 1, 'VoRrEnTz')",
    )
    .execute(&pool)
    .await
    .unwrap_err();
    assert!(err.to_string().contains("clan_mates_guild_id_lower_player_name_idx"), "{err}");
}

#[sqlx::test(migrations = false)]
async fn merged_keeper_is_a_member_if_any_duplicate_was(pool: PgPool) {
    apply_migrations_through(&pool, PRE_MEMBERSHIP_VERSION).await;
    // No WOM group, so migration A's backfill rule 2 applies; both rows are
    // created within its 180-day window, so wom_left_at stays as inserted.
    insert_guild(&pool, 2, None).await;

    // The keeper is WOM-linked but marked as having left; the loser is a
    // current member. Membership must survive the merge. The loser carries a
    // 'Guest' rank from a chat line that predates joining; since the merged
    // row is WOM-confirmed it must not end up labelled Guest.
    sqlx::query(
        "INSERT INTO clan_mates (id, guild_id, player_name, wom_player_id, \"rank\", created_at, in_wom, wom_left_at) VALUES
         ('00000000-0000-0000-0000-000000000011', 2, 'Zezima', 88, NULL, now() - interval '10 days', false, '2026-09-01T00:00:00Z'),
         ('00000000-0000-0000-0000-000000000012', 2, 'ZEZIMA', NULL, 'Guest', now() - interval '20 days', true, NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();

    apply_migrations_after(&pool, PRE_MEMBERSHIP_VERSION).await;

    assert_eq!(count_where(&pool, "SELECT COUNT(*) FROM clan_mates").await, 1);
    let (in_wom, left_at, rank): (bool, Option<DateTime<Utc>>, Option<String>) = sqlx::query_as(
        "SELECT in_wom, left_at, \"rank\" FROM clan_mates WHERE id = '00000000-0000-0000-0000-000000000011'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(in_wom, "the merged loser was in WOM, so the keeper is");
    assert_eq!(left_at, None, "the merged loser had not left, so the keeper has not");
    assert_eq!(rank, None, "a WOM-confirmed merged row is never labelled Guest");
}

#[sqlx::test(migrations = false)]
async fn merge_without_wom_keeps_a_guest_rank(pool: PgPool) {
    apply_migrations_through(&pool, PRE_MEMBERSHIP_VERSION).await;
    insert_guild(&pool, 2, None).await;

    // Neither row is WOM-confirmed, so the rank fold is the plain COALESCE:
    // the keeper (latest chat line) has no rank and takes the loser's Guest.
    sqlx::query(
        "INSERT INTO clan_mates (id, guild_id, player_name, \"rank\", created_at, in_wom, wom_left_at, last_seen_in_chat_at) VALUES
         ('00000000-0000-0000-0000-000000000031', 2, 'Lynx', NULL, now() - interval '10 days', false, NULL, now() - interval '1 day'),
         ('00000000-0000-0000-0000-000000000032', 2, 'LYNX', 'Guest', now() - interval '20 days', false, NULL, NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();

    apply_migrations_after(&pool, PRE_MEMBERSHIP_VERSION).await;

    assert_eq!(count_where(&pool, "SELECT COUNT(*) FROM clan_mates").await, 1);
    let rank: Option<String> = sqlx::query_scalar(
        "SELECT \"rank\" FROM clan_mates WHERE id = '00000000-0000-0000-0000-000000000031'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(rank.as_deref(), Some("Guest"));
}
