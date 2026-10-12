use crate::database::BotDb;
use crate::database::clan_mates::{ClanMateModel, ClanMates, name_compare};
use crate::database::guilds_db::RegisteredGuildModel;
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
        updated.wom_left_at = None;
        if !name_compare(&row.player_name, &membership.player.display_name) {
            updated.previous_names.push(row.player_name.clone());
            updated.player_name = membership.player.display_name.clone();
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
        updated.wom_left_at = None;
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
                matched.push(row.id);
                actions.push(SyncAction::RenameAndLink {
                    old_name: row.player_name.clone(),
                    new_name: membership.player.display_name.clone(),
                    wom_player_id: membership.player.id as u64,
                });
                continue;
            }
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
            updated.wom_left_at = Some(now);
            actions.push(SyncAction::SoftRemove { clan_mate: updated });
        }
    }

    actions
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

    let mut created = 0usize;
    let mut renamed = 0usize;
    let mut updated = 0usize;
    let mut soft_removed = 0usize;

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
                    clan_mate.wom_left_at = None;
                    match db.clan_mates.update_clan_mate(clan_mate).await {
                        Ok(_) => renamed += 1,
                        Err(e) => error!("Failed to link renamed clan mate to WOM: {:?}", e),
                    }
                }
                Err(e) => error!("Failed to change clan mate name: {:?}", e),
            },
            SyncAction::SoftRemove { clan_mate } => {
                match db.clan_mates.update_clan_mate(clan_mate).await {
                    Ok(_) => soft_removed += 1,
                    Err(e) => error!("Failed to soft remove clan mate: {:?}", e),
                }
            }
        }
    }

    info!(
        "WOM sync for {:?} (wom id {}): created {}, renamed {}, updated {}, soft-removed {}",
        guild.clan_name, wom_id, created, renamed, updated, soft_removed
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
        wom_left_at: Option<DateTime<Utc>>,
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
            wom_left_at,
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
        assert_eq!(clan_mate.wom_left_at, None);
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
        assert_eq!(clan_mate.wom_left_at, None);
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
}
