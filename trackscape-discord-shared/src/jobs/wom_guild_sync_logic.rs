use crate::database::clan_mates::{name_compare, ClanMateModel, ClanMates};
use crate::database::guilds_db::RegisteredGuildModel;
use crate::database::BotMongoDb;
use crate::wom::{get_wom_client, ApiLimiter};
use log::{error, info};
use wom_rs::models::name::NameChangeStatus;
use wom_rs::Pagination;

pub async fn sync_wom_by_guild(guild: &RegisteredGuildModel, mongodb: &BotMongoDb) {
    info!("Syncing guild: {:?}", guild.clan_name);
    let wom_id = guild.wom_id.unwrap();
    let wom_client = get_wom_client();
    let mut limiter = ApiLimiter::new();

    let wom_group = limiter
        .api_limit_request(
            || async { wom_client.group_client.get_group_details(wom_id).await },
            None,
        )
        .await;
    let wom_group_name_changes = limiter
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
        .expect("Failed to get group name changes");

    if wom_group.is_err() {
        return;
    }

    let guild_clan_mates = mongodb
        .clan_mates
        .get_clan_mates_by_guild_id(guild.guild_id)
        .await
        .expect("Failed to get clan mates");

    let wom_group = wom_group.unwrap();
    for member in wom_group.memberships.clone() {
        //Checks to see if the wom player is in the db list
        let player_in_db_check = guild_clan_mates
            .iter()
            .find(|x| name_compare(&x.player_name, &member.player.username));
        let mut player_whose_name_is_changing: Option<&ClanMateModel> = None;

        if player_in_db_check.is_some() {
            let mut found_player = player_in_db_check.unwrap().clone();
            found_player.wom_player_id = Some(member.player.id as u64);
            let update_clan_mate = mongodb.clan_mates.update_clan_mate(found_player).await;
            if update_clan_mate.is_err() {
                error!("Failed to update clan mate: {:?}", update_clan_mate.err());
            }
        }

        //If wom player is not found in db list
        if player_in_db_check.is_none() {
            //Checks to see if maybe it was a name change
            let name_change = wom_group_name_changes
                .iter()
                .filter(|name_change| {
                    name_change.status == NameChangeStatus::Approved
                        && name_change.resolved_at.is_some()
                        && name_compare(&name_change.new_name, &member.player.username)
                })
                .max_by(|a, b| {
                    a.resolved_at
                        .unwrap()
                        .cmp(&b.resolved_at.unwrap())
                        .reverse()
                });

            //If it is a name change
            if let Some(name_change) = name_change {
                //Get the db player whose name is changing
                player_whose_name_is_changing = guild_clan_mates
                    .iter()
                    .find(|x| name_compare(&x.player_name, &name_change.old_name.clone()));

                //If the db player is not found
                if player_whose_name_is_changing.is_none() {
                    info!(
                        "Player not found for name change: {:?}",
                        name_change.old_name
                    );
                } else {
                    //If the db player already has the new name
                    if player_whose_name_is_changing
                        .unwrap()
                        .previous_names
                        .contains(&name_change.new_name.replace(" ", "\u{a0}"))
                    {
                        info!("Player already has this name: {:?}", name_change.new_name);
                        continue;
                    }
                    info!(
                        "Changing name: {:?} to: {:?}",
                        name_change.old_name, member.player.username
                    );
                    let name_change = mongodb
                        .clan_mates
                        .change_name(
                            guild.guild_id,
                            name_change.old_name.clone(),
                            member.player.display_name.clone(),
                        )
                        .await;

                    if name_change.is_err() {
                        error!("Failed to change name: {:?}", name_change.err());
                    }
                }
            }
        }

        //If the player is already in the db list or the name change was successful
        if player_whose_name_is_changing.is_some() || player_in_db_check.is_some() {
            continue;
        }
        info!("Creating new clan mate: {:?}", member.player.username);

        let create_new_clan_mate = mongodb
            .clan_mates
            .create_new_clan_mate(
                guild.guild_id,
                member.player.display_name,
                Some(member.player.id as u64),
            )
            .await;
        if create_new_clan_mate.is_err() {
            error!(
                "Failed to create new clan mate: {:?}",
                create_new_clan_mate.err()
            );
        }
    }

    //Grabs a new list with fresh entries from name chanegs and new clanmates
    let fresh_guild_clan_mates = mongodb
        .clan_mates
        .get_clan_mates_by_guild_id(guild.guild_id)
        .await
        .expect("Failed to get clan mates");

    //checks for who has left the clan
    //CHeck that they still have collection log id for them
    for db_member in fresh_guild_clan_mates {
        let member = wom_group
            .memberships
            .iter()
            .find(|x| name_compare(&x.player.username, &db_member.player_name));
        if member.is_none() {
            info!("Removing clan mate: {:?}", db_member.player_name);
            let remove_clan_mate = mongodb
                .clan_mates
                .remove_clan_mate(guild.guild_id, db_member.player_name.clone())
                .await;
            if remove_clan_mate.is_err() {
                error!("Failed to remove clan mate: {:?}", remove_clan_mate.err());
            }
        }
    }
}
