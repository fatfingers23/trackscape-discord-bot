use crate::database::BotMongoDb;
use std::any::Any;
use std::time::Duration;

use serenity::builder::CreateMessage;
use serenity::client::Context;
use serenity::model::channel::ChannelType;
use serenity::model::id::ChannelId;

use serenity::model::application::component::InputTextStyle;
use serenity::model::prelude::application_command::{
    ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::message_component::MessageComponentInteraction;
use serenity::model::prelude::modal::ModalSubmitInteraction;
use serenity::model::prelude::Permissions;
use serenity::{builder, Error};
use tracing::info;

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("broadcast_config")
        .description("Sets settings for broadcast messages.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
}

pub async fn run(
    command: ApplicationCommandInteraction,
    ctx: &Context,
    db: &BotMongoDb,
) -> Result<(), Error> {
    let saved_guild_query = db.get_by_guild_id(command.guild_id.unwrap().0).await;

    command
        .create_interaction_response(&ctx.http, |response| {
            response.interaction_response_data(|message| {
                message.content("Can select the settings you'd like to change below.");
                message.components(|c| {
                    c.create_action_row(|a| {
                        a.create_input_text(|i| {
                            i.label("Minimal drop gp value to broadcast")
                                .custom_id("broadcast_drop_threshold")
                                // .style(InputTextStyle::Short)
                                .required(false)
                        });
                        a.create_button(|b| b.label("Save").custom_id("broadcast_config_save"))
                    })
                });
                message.ephemeral(true)
            })
        })
        .await
        .expect("Failed to send response.");

    Ok(())
}

pub async fn handle_submit(
    component: MessageComponentInteraction,
    ctx: &Context,
    db: &BotMongoDb,
) -> Result<(), Error> {
    // let saved_guild_query = db.get_by_guild_id(command.guild_id.unwrap().0).await;

    println!("Received broadcast config save");
    println!("Component: {:#?}", component);
    // component.message.id.delete(&ctx.http).await;
    component
        .delete_original_interaction_response(&ctx.http)
        .await
        .expect("Failed to delete original interaction response.");
    component
        .create_interaction_response(&ctx.http, |response| {
            response.interaction_response_data(|message| {
                message.content("Updated broadcast settings");
                message.ephemeral(true)
            })
        })
        .await
        .expect("Failed to send response.");

    Ok(())
}
