use crate::database::BotMongoDb;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandOptionType, CreateCommand,
    CreateCommandOption, CreateMessage,
};

use serenity::builder;
use serenity::client::Context;
use serenity::model::channel::ChannelType;
use serenity::model::prelude::Permissions;
use tracing::{error, info};
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;

pub fn register() -> builder::CreateCommand {
    CreateCommand::new("set_broadcast_channel")
        .description("Sets a Channel to receive broadcasts. This will get embed messages for Pets, drops, pks, quests, etc")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "broadcast_type",
            "The type of broadcast to set the channel for. Default is all types.",
        )
             .add_string_choice(
                BroadcastType::Default.to_string(), 
                BroadcastType::Default.to_slug(),
             )
             .add_string_choice(
                BroadcastType::ItemDrop.to_string(), 
                BroadcastType::ItemDrop.to_slug(),
             )
             .add_string_choice(
                BroadcastType::RaidDrop.to_string(), 
                BroadcastType::RaidDrop.to_slug(),
             )
             .add_string_choice(
                BroadcastType::PetDrop.to_string(), 
                BroadcastType::PetDrop.to_slug(),
             )
             .add_string_choice(
                BroadcastType::ClueItem.to_string(),
                BroadcastType::ClueItem.to_slug(),
             )
             .add_string_choice(
                BroadcastType::CollectionLog.to_string(), 
                BroadcastType::CollectionLog.to_slug(),
             )
             .add_string_choice(
                BroadcastType::Quest.to_string(), 
                BroadcastType::Quest.to_slug(),
             )
            .add_string_choice(
                BroadcastType::Diary.to_string(), 
                BroadcastType::Diary.to_slug(),
            )
            .add_string_choice(
                BroadcastType::PersonalBest.to_string(), 
                BroadcastType::PersonalBest.to_slug(),
            )
            .add_string_choice(
                BroadcastType::Pk.to_string(), 
                BroadcastType::Pk.to_slug(),
            )
            .add_string_choice(
                BroadcastType::CofferDonation.to_string(), 
                BroadcastType::CofferDonation.to_slug(),
            )
            .add_string_choice(
                BroadcastType::Invite.to_string(),
                BroadcastType::Invite.to_slug(),
            )
            .add_string_choice(
                BroadcastType::ExpelledFromClan.to_string(),
                 BroadcastType::ExpelledFromClan.to_slug(),
            )
            .add_string_choice(
                BroadcastType::LeftTheClan.to_string(),
                BroadcastType::LeftTheClan.to_slug(),
            )
            .add_string_choice(
                BroadcastType::LevelMilestone.to_string(),
                BroadcastType::LevelMilestone.to_slug(),
            )
            .add_string_choice(
                BroadcastType::XPMilestone.to_string(),
                BroadcastType::XPMilestone.to_slug(),
            )
            .add_string_choice(
                BroadcastType::CofferDonation.to_string(),
                BroadcastType::CofferDonation.to_slug(),
            )
            .add_string_choice(
                BroadcastType::CofferWithdrawal.to_string(),
                BroadcastType::CofferWithdrawal.to_slug(),
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Channel,
                "channel", 
                "The discord channel to get broadcast messages in.", 
            )
            .required(true),
        )
}

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    db: &BotMongoDb,
    guild_id: u64,
) -> Option<String> {
    let option = options.get(1).expect("Expected Channel Id option");

    if let CommandDataOptionValue::Channel(channel) = option.value {
        let possible_actual_channel = channel.to_channel(&ctx).await;
        if possible_actual_channel.is_err() {
            error!("Error getting channel: {:?}", possible_actual_channel.err());
            return Some("Error getting channel".to_string());
        }
        let guild_channel = possible_actual_channel
            .expect("Expected channel")
            .guild()
            .expect("Expected guild channel");
        info!("Guild Channel: {:?}", guild_channel);
        if guild_channel.kind != ChannelType::Text {
            error!("Please select a text channel.");
            return Some("Please select a text channel.".to_string());
        }

        let saved_guild_query = db.guilds.get_by_guild_id(guild_id).await;

        return match saved_guild_query {
            Ok(possible_guild) => match possible_guild {
                Some(mut saved_guild) => {
                    let saved_guild_code = saved_guild.verification_code.clone();
                    let broadcast_type = options.get(0).expect("Expected a broadcast type option");
                    if let CommandDataOptionValue::String(broadcast_type) = broadcast_type.clone().value {
                        let  broadcast_type = BroadcastType::from_string(broadcast_type.replace("_", " "));
                        if let Some(ref mut filter_map)= saved_guild.specific_broadcast_channels{
                            filter_map.insert(broadcast_type.clone(), channel.get());
                        }
                        else {
                            let mut new_filter_map = std::collections::HashMap::new();
                            new_filter_map.insert(broadcast_type.clone(), channel.get());
                            saved_guild.specific_broadcast_channels = Some(new_filter_map);
                        }
                        db.guilds.update_guild(saved_guild).await;
                        let send_message = channel
                            .send_message(&ctx.http,
                                        CreateMessage::new().content(format!("This channel has been set as the {} broadcast chat channel.",broadcast_type.clone().to_string()))).await;

                        match send_message {
                            Ok(_) => {}
                            Err(error) => {
                                info!("Error sending message: {}", error);
                                return Some("Error sending a message to the selected channel. Please check that the bot has permission to access this channel. Broadcast messages will be sent once this is resolved.".to_string());
                            }
                        }
                    }
                    
                    //TODO: Send message to channel with verfication code and a picture of where to add it
                    Some(format!("The channel has been set successfully. Please use the code: `{}` in the TrackScape Connection RuneLite Plugin to begin receiving messages in the selected channel.", saved_guild_code))
                }
                None => {
                    Some("Error finding your server as registered. Try kicking and re adding the bot please.".to_string())
                }
            },
            Err(_) => {
                Some("There was a technical error. Please try again later.".to_string())
            }
        };
    }
    info!("Error getting channel");
    Some("Error getting channel".to_string())
}
