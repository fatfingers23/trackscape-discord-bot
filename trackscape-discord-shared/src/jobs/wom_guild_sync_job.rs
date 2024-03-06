use crate::database::clan_mates::ClanMates;
use crate::jobs::job_helpers::get_mongodb;
use crate::wom::get_wom_client;
use celery::task::TaskResult;
use wom_rs::Pagination;

#[celery::task]
pub async fn wom_guild_sync() -> TaskResult<()> {
    let mongodb = get_mongodb().await;
    let wom_client = get_wom_client();
    let guilds = mongodb
        .guilds
        .list_clans()
        .await
        .expect("Failed to get all guilds");

    for guild in guilds {
        if guild.wom_id.is_none() {
            continue;
        }
        let wom_id = guild.wom_id.unwrap();
        let wom_group = wom_client.group_client.get_group_details(wom_id).await;
        let wom_group_name_changes = wom_client
            .group_client
            .get_group_name_changes(
                wom_id,
                Some(Pagination {
                    offset: None,
                    limit: Some(100),
                }),
            )
            .await;

        if wom_group.is_err() {
            continue;
        }
        let guild_clan_mates = mongodb
            .clan_mates
            .get_clan_mates_by_guild_id(guild.guild_id)
            .await
            .expect("Failed to get clan mates");

        let wom_group = wom_group.unwrap();
        for member in wom_group.memberships {
            let player = guild_clan_mates.iter().find(|x| {
                x.player_name.replace("\u{a0}", "").to_lowercase()
                    == member.player.username.to_lowercase()
            });
            if player.is_none() {
                //TODO check name change first
                mongodb
                    .clan_mates
                    .create_new_clan_mate(
                        guild.guild_id,
                        member.player.username,
                        Some(member.player.id as u64),
                    )
                    .await
                    .expect("Failed to create new clan mate");
            }
        }
    }

    Ok(())
}
