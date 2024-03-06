use crate::database::clan_mates::{name_compare, ClanMates};
use crate::jobs::job_helpers::get_mongodb;
use crate::wom::{get_wom_client, ApiLimiter};
use celery::prelude::*;
use log::info;
use wom_rs;
use wom_rs::models::name::NameChangeStatus;

const RATE_LIMIT: i32 = 100;

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
        for mut player in players {
            let player_name = player.player_name.clone();
            let player_name_change_result = limiter
                .api_limit_request(
                    || async {
                        let player_name_change_result = wom_client
                            .player_client
                            .get_name_changes(player_name.clone())
                            .await;

                        player_name_change_result
                    },
                    Some(std::time::Duration::from_millis(400)),
                )
                .await;

            match player_name_change_result {
                Ok(player_name_changes) => {
                    let latest_name_change = player_name_changes
                        .iter()
                        .filter(|name_change| name_change.status == NameChangeStatus::Approved)
                        .max_by(|a, b| a.resolved_at.cmp(&b.resolved_at));
                    if latest_name_change.is_none() {
                        info!("No name changes found for player: {}", player_name);
                        continue;
                    }
                    let latest_name = latest_name_change.unwrap().new_name.clone();
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

                Err(err) => {
                    info!("Failed to get name changes for player: {}", player_name);
                    continue;
                }
            }
        }
    }

    Ok(())
}
