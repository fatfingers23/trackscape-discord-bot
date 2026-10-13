use crate::database::BotDb;
use crate::database::clan_mates::{ClanMateModel, ClanMates, name_compare, name_normalize};
use crate::database::guilds_db::RegisteredGuildModel;
use crate::jobs::job_helpers::{invalidate_clan_mate_cache, try_get_redis_connection};
use crate::wom::{ApiLimiter, get_wom_client};
use chrono::{DateTime, Utc};
use log::{error, info, warn};
use uuid::Uuid;
use wom_rs::Pagination;
use wom_rs::models::group::GroupMemberShip;
use wom_rs::models::name::{NameChange, NameChangeStatus};

/// One reconciliation step decided by `plan_wom_sync`.
#[derive(Debug, PartialEq)]
pub enum SyncAction {
    /// An existing clan mate matched a WOM membership; `clan_mate` is the
    /// model in its desired state. `changed == false` means the row already
    /// looks like this and no database write is needed.
    UpdateMember { clan_mate: ClanMateModel, changed: bool },
    /// A WOM member with no matching row: create a new clan mate.
    CreateMember { player_name: String, wom_player_id: u64 },
    /// A WOM member whose player was renamed: rename the matching row and
    /// link it to the WOM player id. `old_name` is the row's current name.
    RenameAndLink {
        old_name: String,
        new_name: String,
        wom_player_id: u64,
    },
    /// A member renamed in WOM whose new name is already held by an
    /// unlinked (chat-created) duplicate row. The duplicate is folded into
    /// `clan_mate` — its name history and sightings are absorbed here, its
    /// personal bests and collection log totals move in the DB layer — and
    /// the duplicate row is deleted, all before `clan_mate` takes the new
    /// name (the `(guild_id, player_name)` unique constraint means the
    /// rename can only succeed once the duplicate is gone).
    MergeDuplicate {
        clan_mate: ClanMateModel,
        duplicate_id: Uuid,
    },
    /// A clan mate no longer in the WOM group: flag it as left (soft remove —
    /// never delete, PB records and clog totals must survive).
    SoftRemove { clan_mate: ClanMateModel },
}

/// Pure reconciliation: decides what to do with the DB rows given the WOM
/// group's memberships and (approved) name changes. No I/O, so it is unit
/// testable. `now` is injected for testability.
pub fn plan_wom_sync(
    db_members: &[ClanMateModel],
    wom_memberships: &[GroupMemberShip],
    name_changes: &[NameChange],
    now: DateTime<Utc>,
) -> Vec<SyncAction> {
    let mut actions = Vec::new();
    let mut claimed: Vec<i64> = Vec::new();
    let mut matched: Vec<Uuid> = Vec::new();

    // Pass 1 — match by WOM player id.
    for row in db_members {
        if matched.contains(&row.id) {
            continue;
        }
        let Some(wom_id) = row.wom_player_id else {
            continue;
        };
        let membership = wom_memberships
            .iter()
            .find(|m| m.player.id == wom_id as i64 && !claimed.contains(&m.player.id));
        let Some(membership) = membership else {
            continue;
        };
        claimed.push(membership.player.id);
        matched.push(row.id);

        let mut updated = row.clone();
        updated.in_wom = true;
        updated.left_at = None;
        if !name_compare(&row.player_name, &membership.player.display_name) {
            match find_rename_holder(db_members, row.id, &membership.player.display_name) {
                // Nobody else holds the new name — plain rename.
                None => {
                    updated.previous_names.push(row.player_name.clone());
                    updated.player_name = membership.player.display_name.clone();
                }
                // A chat-created duplicate (never linked to a WOM player)
                // already holds the new name. Fold it into this row so the
                // rename cannot violate the unique constraint, keeping its
                // name history and sightings.
                Some(holder) if holder.wom_player_id.is_none() => {
                    updated.player_name = membership.player.display_name.clone();
                    updated.previous_names.push(row.player_name.clone());
                    updated
                        .previous_names
                        .extend(holder.previous_names.iter().cloned());
                    dedup_previous_names(&mut updated.previous_names);
                    updated.created_at = row.created_at.min(holder.created_at);
                    updated.last_seen_in_chat_at =
                        row.last_seen_in_chat_at.max(holder.last_seen_in_chat_at);
                    updated.rank = updated.rank.or(holder.rank.clone());
                    matched.push(holder.id);
                    actions.push(SyncAction::MergeDuplicate {
                        clan_mate: updated,
                        duplicate_id: holder.id,
                    });
                    continue;
                }
                // The new name is held by a row linked to a different WOM
                // player — a recycled name. Renaming onto it would violate
                // the unique constraint (and steal another member's name),
                // so this row keeps its current name but is still
                // re-activated. Re-planned the same way on every sync, so
                // the member's name is corrected as soon as the holder is
                // renamed or removed.
                Some(_) => {}
            }
        }
        let changed = updated != *row;
        actions.push(SyncAction::UpdateMember {
            clan_mate: updated,
            changed,
        });
    }

    // Pass 2 — name match for still-unmatched rows (rows with no wom id, or
    // whose id had no unclaimed membership).
    for row in db_members {
        if matched.contains(&row.id) {
            continue;
        }
        let membership = wom_memberships.iter().find(|m| {
            !claimed.contains(&m.player.id) && name_compare(&m.player.username, &row.player_name)
        });
        let Some(membership) = membership else {
            continue;
        };
        claimed.push(membership.player.id);
        matched.push(row.id);

        let mut updated = row.clone();
        updated.wom_player_id = Some(membership.player.id as u64);
        updated.in_wom = true;
        updated.left_at = None;
        let changed = updated != *row;
        actions.push(SyncAction::UpdateMember {
            clan_mate: updated,
            changed,
        });
    }

    // Pass 3 — unclaimed memberships: rename-link or create.
    for membership in wom_memberships {
        if claimed.contains(&membership.player.id) {
            continue;
        }
        claimed.push(membership.player.id);

        // Latest approved + resolved name change that applies to this member.
        let name_change = name_changes
            .iter()
            .filter(|nc| {
                nc.status == NameChangeStatus::Approved
                    && nc.resolved_at.is_some()
                    && (nc.player_id == membership.player.id
                        || name_compare(&nc.new_name, &membership.player.username))
            })
            .max_by(|a, b| a.resolved_at.cmp(&b.resolved_at));

        if let Some(nc) = name_change {
            let row = db_members
                .iter()
                .find(|r| !matched.contains(&r.id) && name_compare(&r.player_name, &nc.old_name));
            if let Some(row) = row {
                // The new name can only be held by a row matched by WOM
                // player id in pass 1 (pass 2 claims memberships by name,
                // and WOM usernames are unique) — a stale link holding a
                // recycled name. Renaming on top of it would violate the
                // unique constraint, so leave this member for a later sync.
                let target_taken = db_members.iter().any(|r| {
                    r.id != row.id && name_compare(&r.player_name, &membership.player.display_name)
                });
                if target_taken {
                    continue;
                }
                matched.push(row.id);
                actions.push(SyncAction::RenameAndLink {
                    old_name: row.player_name.clone(),
                    new_name: membership.player.display_name.clone(),
                    wom_player_id: membership.player.id as u64,
                });
                continue;
            }
        }

        // Same guard for creates: a row holding the name was matched in
        // pass 1 (see above), and inserting a second row with that name
        // would violate the unique constraint.
        let name_taken = db_members
            .iter()
            .any(|r| name_compare(&r.player_name, &membership.player.display_name));
        if name_taken {
            continue;
        }

        actions.push(SyncAction::CreateMember {
            player_name: membership.player.display_name.clone(),
            wom_player_id: membership.player.id as u64,
        });
    }

    // Pass 4 — soft removal for rows no longer in the WOM group.
    for row in db_members {
        if matched.contains(&row.id) {
            continue;
        }
        if row.in_wom {
            let mut updated = row.clone();
            updated.in_wom = false;
            updated.left_at = Some(now);
            actions.push(SyncAction::SoftRemove { clan_mate: updated });
        }
    }

    actions
}

/// Why a fetched WOM group should not be reconciled against this clan's rows.
/// A group that lists nobody, or that shares no player with a clan we already
/// know a fair number of members for, is far more likely a wrong or stale
/// `wom_id` than a real roster change; acting on it would create strangers
/// as clan mates. The caller logs the reason and skips the guild.
pub fn wom_group_looks_unrelated(
    db_members: &[ClanMateModel],
    wom_memberships: &[GroupMemberShip],
    actions: &[SyncAction],
) -> Option<&'static str> {
    if wom_memberships.is_empty() {
        return Some("the WOM group has no members");
    }
    const MIN_KNOWN_ROWS: usize = 5;
    let overlaps = actions.iter().any(|a| {
        matches!(
            a,
            SyncAction::UpdateMember { .. }
                | SyncAction::RenameAndLink { .. }
                | SyncAction::MergeDuplicate { .. }
        )
    });
    if db_members.len() >= MIN_KNOWN_ROWS && !overlaps {
        return Some("the WOM group shares no player with the clan's existing members");
    }
    None
}

/// Finds another row already holding `new_name` (case-insensitive), which
/// would make renaming `row_id` onto it violate the `(guild_id,
/// player_name)` unique constraint. Matched rows count as holders too: they
/// are linked to a WOM player, so their name must not be stolen.
fn find_rename_holder<'a>(
    db_members: &'a [ClanMateModel],
    row_id: Uuid,
    new_name: &str,
) -> Option<&'a ClanMateModel> {
    db_members
        .iter()
        .find(|r| r.id != row_id && name_compare(&r.player_name, new_name))
}

/// Drops duplicate entries from a merged name history, keeping first
/// occurrence order (the oldest recorded name stays first).
fn dedup_previous_names(names: &mut Vec<String>) {
    let mut seen = std::collections::HashSet::new();
    names.retain(|name| seen.insert(name_normalize(name)));
}

pub async fn sync_wom_by_guild(guild: &RegisteredGuildModel, db: &BotDb) {
    info!("Syncing guild: {:?}", guild.clan_name);
    let Some(wom_id) = guild.wom_id else {
        info!("No wom id found for guild: {:?}", guild.clan_name);
        return;
    };
    let wom_client = get_wom_client();
    let mut limiter = ApiLimiter::new();

    let wom_group = match limiter
        .api_limit_request(
            || async { wom_client.group_client.get_group_details(wom_id).await },
            None,
        )
        .await
    {
        Ok(wom_group) => wom_group,
        Err(e) => {
            error!(
                "Failed to get WOM group details for clan {:?} (wom id {}): {:?}. Skipping this guild.",
                guild.clan_name, wom_id, e
            );
            return;
        }
    };

    let name_changes = match limiter
        .api_limit_request(
            || async {
                wom_client
                    .group_client
                    .get_group_name_changes(
                        wom_id,
                        Some(Pagination {
                            offset: None,
                            limit: Some(50),
                        }),
                    )
                    .await
            },
            None,
        )
        .await
    {
        Ok(name_changes) => name_changes,
        Err(e) => {
            warn!(
                "Failed to get WOM group name changes for clan {:?} (wom id {}): {:?}. Continuing without name changes.",
                guild.clan_name, wom_id, e
            );
            Vec::new()
        }
    };

    let db_members = match db.clan_mates.get_clan_mates_by_guild_id(guild.guild_id).await {
        Ok(db_members) => db_members,
        Err(e) => {
            error!(
                "Failed to get clan mates for guild {:?}: {:?}. Skipping this guild.",
                guild.guild_id, e
            );
            return;
        }
    };

    let actions = plan_wom_sync(&db_members, &wom_group.memberships, &name_changes, Utc::now());

    if let Some(reason) = wom_group_looks_unrelated(&db_members, &wom_group.memberships, &actions) {
        warn!(
            "Skipping WOM sync for clan {:?} (wom id {}): {}",
            guild.clan_name, wom_id, reason
        );
        return;
    }

    let mut created = 0usize;
    let mut renamed = 0usize;
    let mut updated = 0usize;
    let mut merged = 0usize;
    let mut soft_removed = 0usize;
    // Opened on the first soft removal only; `None` inside means Redis was
    // unavailable (already logged), so cache invalidation is skipped.
    let mut redis: Option<Option<redis::Connection>> = None;

    for action in actions {
        match action {
            SyncAction::UpdateMember { changed: false, .. } => {}
            SyncAction::UpdateMember { clan_mate, .. } => {
                match db.clan_mates.update_clan_mate(clan_mate).await {
                    Ok(_) => updated += 1,
                    Err(e) => error!("Failed to update clan mate: {:?}", e),
                }
            }
            SyncAction::CreateMember {
                player_name,
                wom_player_id,
            } => {
                match db
                    .clan_mates
                    .create_new_clan_mate(guild.guild_id, player_name, Some(wom_player_id))
                    .await
                {
                    Ok(_) => created += 1,
                    Err(e) => error!("Failed to create new clan mate: {:?}", e),
                }
            }
            SyncAction::RenameAndLink {
                old_name,
                new_name,
                wom_player_id,
            } => match db
                .clan_mates
                .change_name(guild.guild_id, old_name, new_name)
                .await
            {
                Ok(mut clan_mate) => {
                    clan_mate.wom_player_id = Some(wom_player_id);
                    clan_mate.in_wom = true;
                    clan_mate.left_at = None;
                    match db.clan_mates.update_clan_mate(clan_mate).await {
                        Ok(_) => renamed += 1,
                        Err(e) => error!("Failed to link renamed clan mate to WOM: {:?}", e),
                    }
                }
                Err(e) => error!("Failed to change clan mate name: {:?}", e),
            },
            SyncAction::MergeDuplicate {
                clan_mate,
                duplicate_id,
            } => match db.clan_mates.merge_clan_mates(clan_mate, duplicate_id).await {
                Ok(_) => merged += 1,
                Err(e) => error!("Failed to merge duplicate clan mate: {:?}", e),
            },
            SyncAction::SoftRemove { clan_mate } => {
                match db.clan_mates.update_clan_mate(clan_mate).await {
                    Ok(removed) => {
                        soft_removed += 1;
                        // Drop the chat job's cached copy so a rejoin restores them.
                        if let Some(connection) = redis.get_or_insert_with(try_get_redis_connection)
                        {
                            invalidate_clan_mate_cache(
                                connection,
                                guild.guild_id,
                                &removed.player_name,
                            );
                        }
                    }
                    Err(e) => error!("Failed to soft remove clan mate: {:?}", e),
                }
            }
        }
    }

    info!(
        "WOM sync for {:?} (wom id {}): created {}, renamed {}, updated {}, merged {}, soft-removed {}",
        guild.clan_name, wom_id, created, renamed, updated, merged, soft_removed
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Duration};
    use wom_rs::models::player::{Player, PlayerBuild, PlayerStatus, PlayerType};

    fn test_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, 12, 0, 0).unwrap()
    }

    fn test_player(id: i64, username: &str, display_name: &str) -> Player {
        Player {
            id,
            username: username.to_string(),
            display_name: display_name.to_string(),
            player_type: PlayerType::Regular,
            build: PlayerBuild::Main,
            country: None,
            status: PlayerStatus::Active,
            patron: false,
            exp: 0,
            ehp: 0.0,
            ehb: 0.0,
            ttm: 0.0,
            tt200m: 0.0,
            registered_at: "2020-01-01T00:00:00.000Z".to_string(),
            updated_at: None,
            last_changed_at: None,
            last_imported_at: None,
        }
    }

    fn test_membership(player: Player) -> GroupMemberShip {
        let created_at = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        GroupMemberShip {
            player_id: player.id,
            group_id: 1,
            role: None,
            created_at,
            updated_at: created_at,
            player,
        }
    }

    fn test_name_change(
        player_id: i64,
        old_name: &str,
        new_name: &str,
        resolved_at: Option<DateTime<Utc>>,
    ) -> NameChange {
        let created_at = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        NameChange {
            id: player_id,
            player_id,
            old_name: old_name.to_string(),
            new_name: new_name.to_string(),
            status: NameChangeStatus::Approved,
            review_context: None,
            resolved_at,
            created_at,
            updated_at: created_at,
            player: None,
        }
    }

    fn test_clan_mate(
        id: u128,
        player_name: &str,
        wom_player_id: Option<u64>,
        in_wom: bool,
        left_at: Option<DateTime<Utc>>,
    ) -> ClanMateModel {
        ClanMateModel {
            id: Uuid::from_u128(id),
            guild_id: 1,
            player_name: player_name.to_string(),
            wom_player_id,
            previous_names: Vec::new(),
            rank: None,
            created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            in_wom,
            left_at,
            last_seen_in_chat_at: None,
        }
    }

    fn update_member(action: &SyncAction) -> (&ClanMateModel, bool) {
        match action {
            SyncAction::UpdateMember { clan_mate, changed } => (clan_mate, *changed),
            other => panic!("expected UpdateMember, got {:?}", other),
        }
    }

    #[test]
    fn new_wom_member_not_in_db_creates() {
        let memberships = vec![test_membership(test_player(5, "New Guy", "New Guy"))];

        let actions = plan_wom_sync(&[], &memberships, &[], test_now());

        assert_eq!(
            actions,
            vec![SyncAction::CreateMember {
                player_name: "New Guy".to_string(),
                wom_player_id: 5,
            }]
        );
    }

    #[test]
    fn matched_member_same_name_needs_no_write() {
        let row = test_clan_mate(1, "Same Guy", Some(5), true, None);
        let memberships = vec![test_membership(test_player(5, "Same Guy", "Same Guy"))];

        let actions = plan_wom_sync(&[row], &memberships, &[], test_now());

        assert_eq!(actions.len(), 1);
        let (_, changed) = update_member(&actions[0]);
        assert!(!changed, "no write expected for an unchanged member");
    }

    #[test]
    fn rejoining_member_is_reactivated() {
        let old_left_at = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let row = test_clan_mate(1, "Rejoiner", Some(5), false, Some(old_left_at));
        let memberships = vec![test_membership(test_player(5, "Rejoiner", "Rejoiner"))];

        let actions = plan_wom_sync(&[row], &memberships, &[], test_now());

        assert_eq!(actions.len(), 1);
        let (clan_mate, changed) = update_member(&actions[0]);
        assert!(changed);
        assert!(clan_mate.in_wom);
        assert_eq!(clan_mate.left_at, None);
    }

    #[test]
    fn row_without_wom_id_links_by_username() {
        let row = test_clan_mate(1, "Link Me", None, false, None);
        let memberships = vec![test_membership(test_player(7, "Link Me", "Link Me"))];

        let actions = plan_wom_sync(&[row], &memberships, &[], test_now());

        assert_eq!(actions.len(), 1);
        let (clan_mate, changed) = update_member(&actions[0]);
        assert!(changed);
        assert_eq!(clan_mate.wom_player_id, Some(7));
        assert!(clan_mate.in_wom);
        assert_eq!(clan_mate.left_at, None);
        // The row's own spelling is kept.
        assert_eq!(clan_mate.player_name, "Link Me");
    }

    #[test]
    fn member_renamed_in_wom_updates_name() {
        let row = test_clan_mate(1, "Old Name", Some(5), true, None);
        let memberships = vec![test_membership(test_player(5, "New Name", "New Name"))];

        let actions = plan_wom_sync(&[row], &memberships, &[], test_now());

        assert_eq!(actions.len(), 1);
        let (clan_mate, changed) = update_member(&actions[0]);
        assert!(changed);
        assert_eq!(clan_mate.player_name, "New Name");
        assert_eq!(clan_mate.previous_names, vec!["Old Name".to_string()]);
    }

    #[test]
    fn approved_name_change_renames_and_links() {
        let row = test_clan_mate(1, "Old Name", None, true, None);
        let memberships = vec![test_membership(test_player(5, "New Name", "New Name"))];
        let name_changes = vec![test_name_change(5, "Old Name", "New Name", Some(test_now()))];

        let actions = plan_wom_sync(&[row], &memberships, &name_changes, test_now());

        assert_eq!(
            actions,
            vec![SyncAction::RenameAndLink {
                old_name: "Old Name".to_string(),
                new_name: "New Name".to_string(),
                wom_player_id: 5,
            }]
        );
    }

    #[test]
    fn latest_name_change_wins() {
        let now = test_now();
        let ten_days_ago = now - Duration::days(10);
        let rows = vec![
            test_clan_mate(1, "OldA", None, true, None),
            test_clan_mate(2, "OldB", None, true, None),
        ];
        let memberships = vec![test_membership(test_player(5, "New Name", "New Name"))];
        let name_changes = vec![
            test_name_change(5, "OldA", "New Name", Some(ten_days_ago)),
            test_name_change(5, "OldB", "New Name", Some(now)),
        ];

        let actions = plan_wom_sync(&rows, &memberships, &name_changes, now);

        // The latest resolved change ("OldB") is the one applied.
        assert_eq!(
            actions[0],
            SyncAction::RenameAndLink {
                old_name: "OldB".to_string(),
                new_name: "New Name".to_string(),
                wom_player_id: 5,
            }
        );
        // "OldA" was not matched and is soft removed.
        assert!(matches!(actions[1], SyncAction::SoftRemove { .. }));
    }

    #[test]
    fn member_missing_from_wom_is_soft_removed() {
        let now = test_now();
        let row = test_clan_mate(1, "Leaver", Some(5), true, None);

        let actions = plan_wom_sync(&[row], &[], &[], now);

        assert_eq!(
            actions,
            vec![SyncAction::SoftRemove {
                clan_mate: test_clan_mate(1, "Leaver", Some(5), false, Some(now)),
            }]
        );
    }

    #[test]
    fn inactive_member_missing_from_wom_produces_no_action() {
        let left_at = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let row = test_clan_mate(1, "Gone", Some(5), false, Some(left_at));

        let actions = plan_wom_sync(&[row], &[], &[], test_now());

        assert!(actions.is_empty());
    }

    #[test]
    fn pass_4_leaves_unlinked_rows_alone() {
        // A chat-created row that WOM never linked is not WOM's to remove:
        // only rows the sync itself marked in_wom are soft-removed.
        let unlinked = test_clan_mate(1, "Chatty", None, false, None);
        let now = Utc.with_ymd_and_hms(2026, 10, 14, 0, 0, 0).unwrap();
        let actions = plan_wom_sync(&[unlinked], &[], &[], now);
        assert!(actions.is_empty(), "got {:?}", actions);
    }

    #[test]
    fn renamed_member_whose_new_name_is_taken_by_chat_duplicate_merges() {
        // Arcane PvM shape: the player renamed in WOM (Braunfel -> "Auss
        // tx") while clan chat already created an unlinked row for the new
        // name. Before the merge, the rename violated the
        // (guild_id, player_name) unique constraint on every sync.
        let mut linked = test_clan_mate(1, "Braunfel", Some(62361), true, None);
        linked.created_at = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let mut duplicate = test_clan_mate(2, "Auss tx", None, false, None);
        duplicate.created_at = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap();
        let memberships = vec![test_membership(test_player(62361, "auss tx", "Auss tx"))];

        let actions = plan_wom_sync(
            &[linked, duplicate.clone()],
            &memberships,
            &[],
            test_now(),
        );

        assert_eq!(actions.len(), 1);
        match &actions[0] {
            SyncAction::MergeDuplicate {
                clan_mate,
                duplicate_id,
            } => {
                assert_eq!(*duplicate_id, duplicate.id);
                assert_eq!(clan_mate.player_name, "Auss tx");
                assert_eq!(clan_mate.previous_names, vec!["Braunfel".to_string()]);
                assert!(clan_mate.in_wom);
                assert_eq!(clan_mate.left_at, None);
                assert_eq!(clan_mate.wom_player_id, Some(62361));
                // The merged row keeps the earlier creation date.
                assert_eq!(
                    clan_mate.created_at,
                    Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
                );
            }
            other => panic!("expected MergeDuplicate, got {:?}", other),
        }
    }

    #[test]
    fn merge_absorbs_duplicate_history_and_sightings() {
        let mut linked = test_clan_mate(1, "Old Name", Some(5), true, None);
        linked.last_seen_in_chat_at = Some(Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap());
        let mut duplicate = test_clan_mate(2, "New Name", None, false, None);
        duplicate.previous_names = vec!["Newer Name".to_string(), "old name".to_string()];
        duplicate.last_seen_in_chat_at = Some(Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap());
        duplicate.rank = Some("Deputy Owner".to_string());
        let memberships = vec![test_membership(test_player(5, "new name", "New Name"))];

        let actions = plan_wom_sync(&[linked, duplicate], &memberships, &[], test_now());

        let SyncAction::MergeDuplicate { clan_mate, .. } = &actions[0] else {
            panic!("expected MergeDuplicate, got {:?}", actions[0])
        };
        // The old name is remembered; the duplicate's history is absorbed
        // without case-insensitive duplicates; the latest chat sighting and
        // a missing rank are carried over.
        assert_eq!(
            clan_mate.previous_names,
            vec!["Old Name".to_string(), "Newer Name".to_string()]
        );
        assert_eq!(
            clan_mate.last_seen_in_chat_at,
            Some(Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap())
        );
        assert_eq!(clan_mate.rank, Some("Deputy Owner".to_string()));
    }

    #[test]
    fn renamed_member_whose_new_name_is_taken_by_linked_row_keeps_old_name() {
        // Recycled name: a row linked to a *different* WOM player holds the
        // new name. The member keeps its current name (still re-activated)
        // instead of stealing it, and the sync stays stable and idempotent.
        let linked = test_clan_mate(1, "Old Name", Some(5), false, Some(test_now()));
        let holder = test_clan_mate(2, "New Name", Some(9), false, Some(test_now()));
        let memberships = vec![test_membership(test_player(5, "new name", "New Name"))];

        let actions = plan_wom_sync(&[linked, holder], &memberships, &[], test_now());

        assert_eq!(actions.len(), 1);
        let (clan_mate, changed) = update_member(&actions[0]);
        assert_eq!(clan_mate.player_name, "Old Name");
        assert_eq!(clan_mate.previous_names, Vec::<String>::new());
        assert!(clan_mate.in_wom);
        assert_eq!(clan_mate.left_at, None);
        assert!(changed);
    }

    #[test]
    fn name_change_rename_onto_taken_name_is_skipped() {
        // Player 5 renamed Old -> New, but holder B (stale-linked to player
        // 9) still holds "New": its own rename to player 9's current name
        // "Whatever" is blocked by holder C, which holds that name. The
        // rename onto "New" would violate the unique constraint, so the
        // member is left for a later sync — and self-heals as soon as the
        // chain of stale names resolves (C frees "Whatever", then B frees
        // "New").
        let row = test_clan_mate(1, "Old", None, true, None);
        let holder_b = test_clan_mate(2, "New", Some(9), true, None);
        let holder_c = test_clan_mate(3, "Whatever", Some(10), true, None);
        let memberships = vec![
            test_membership(test_player(5, "new", "New")),
            test_membership(test_player(9, "whatever", "Whatever")),
            test_membership(test_player(10, "else", "Else")),
        ];
        let name_changes = vec![test_name_change(5, "Old", "New", Some(test_now()))];

        let actions = plan_wom_sync(
            &[row, holder_b.clone(), holder_c],
            &memberships,
            &name_changes,
            test_now(),
        );

        // Holder B is matched by WOM player id but keeps its name (its own
        // rename is blocked); the rename onto "New" is skipped, so "Old"
        // just soft-removes instead of colliding.
        assert!(actions.contains(&SyncAction::UpdateMember {
            clan_mate: holder_b,
            changed: false,
        }));
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, SyncAction::RenameAndLink { .. }))
        );
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, SyncAction::SoftRemove { .. }))
        );
    }

    #[test]
    fn create_member_whose_name_is_taken_by_linked_row_is_skipped() {
        // Player 5's name is held by holder B, whose own rename is blocked
        // by holder C (same stale-name chain as above). Creating a second
        // row with the name would violate the unique constraint, so the
        // member waits for a later sync.
        let holder_b = test_clan_mate(2, "Held", Some(9), true, None);
        let holder_c = test_clan_mate(3, "Other", Some(10), true, None);
        let memberships = vec![
            test_membership(test_player(5, "held", "Held")),
            test_membership(test_player(9, "other", "Other")),
            test_membership(test_player(10, "else", "Else")),
        ];

        let actions = plan_wom_sync(&[holder_b, holder_c], &memberships, &[], test_now());

        assert_eq!(actions.len(), 2);
        assert!(
            actions
                .iter()
                .all(|a| matches!(a, SyncAction::UpdateMember { .. }))
        );
    }

    fn known_rows(count: u128) -> Vec<ClanMateModel> {
        (1..=count)
            .map(|i| test_clan_mate(i, &format!("Known {i}"), None, false, None))
            .collect()
    }

    fn strangers() -> Vec<GroupMemberShip> {
        (100..103)
            .map(|id| {
                let name = format!("Stranger {id}");
                test_membership(test_player(id, &name.to_lowercase(), &name))
            })
            .collect()
    }

    #[test]
    fn empty_group_looks_unrelated() {
        let rows = known_rows(10);
        let actions = plan_wom_sync(&rows, &[], &[], test_now());

        assert_eq!(
            wom_group_looks_unrelated(&rows, &[], &actions),
            Some("the WOM group has no members")
        );
    }

    #[test]
    fn group_of_strangers_for_known_clan_looks_unrelated() {
        let rows = known_rows(10);
        let memberships = strangers();
        let actions = plan_wom_sync(&rows, &memberships, &[], test_now());

        assert_eq!(
            wom_group_looks_unrelated(&rows, &memberships, &actions),
            Some("the WOM group shares no player with the clan's existing members")
        );
    }

    #[test]
    fn group_of_strangers_for_new_clan_is_accepted() {
        let rows = known_rows(2);
        let memberships = strangers();
        let actions = plan_wom_sync(&rows, &memberships, &[], test_now());

        assert_eq!(wom_group_looks_unrelated(&rows, &memberships, &actions), None);
    }

    #[test]
    fn group_sharing_one_player_is_accepted() {
        let rows = known_rows(10);
        let mut memberships = strangers();
        memberships.push(test_membership(test_player(200, "known 3", "Known 3")));
        let actions = plan_wom_sync(&rows, &memberships, &[], test_now());
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, SyncAction::UpdateMember { .. }))
        );

        assert_eq!(wom_group_looks_unrelated(&rows, &memberships, &actions), None);
    }
}
