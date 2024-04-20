use crate::database::clan_mates::{name_compare, ClanMates};
use crate::jobs::job_helpers::get_mongodb;
use crate::wom::{get_latest_name_change, get_wom_client, ApiLimiter};
use celery::prelude::*;
use log::info;

#[celery::task]
pub async fn name_change() -> TaskResult<()> {
    info!("Running name change job");
    let wom_client = get_wom_client();
    let mongodb = get_mongodb().await;

    let mut limiter = ApiLimiter::new();

    let guilds = mongodb
        .guilds
        .list_clans()
        .await
        .expect("Failed to get all guilds");

    for guild in guilds {
        let players = mongodb
            .clan_mates
            .get_clan_mates_by_guild_id(guild.guild_id)
            .await
            .expect("Failed to get clan mates");
        for player in players {
            let player_name = player.player_name.clone();
            let latest_player_name_change_result = limiter
                .api_limit_request(
                    || async { get_latest_name_change(&wom_client, player_name.clone()).await },
                    Some(std::time::Duration::from_millis(400)),
                )
                .await;

            match latest_player_name_change_result {
                Ok(possible_latest_player_name_change) => {
                    if possible_latest_player_name_change.is_some() {
                        let latest_name = possible_latest_player_name_change.unwrap().new_name;
                        if name_compare(&latest_name, &player.player_name) {
                            info!("No new name changes found for player: {}", player_name);
                            continue;
                        }
                        mongodb
                            .clan_mates
                            .change_name(guild.guild_id, player_name.clone(), latest_name.clone())
                            .await
                            .expect("Failed to change name");

                        info!(
                            "Updated player: {:?} with new name: {:?}",
                            player_name, latest_name
                        );
                    }
                }

                Err(_) => {
                    info!("Failed to get name changes for player: {}", player_name);
                    continue;
                }
            }
        }
    }

    Ok(())
}
