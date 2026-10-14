mod commands;
mod events;
mod on_boarding_message;
use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use std::env;
use std::sync::atomic::AtomicUsize;
use tracing::{error, info};
use trackscape_discord_shared::api_web_client::ApiWebClient;
use trackscape_discord_shared::database::BotDb;

pub struct Data {
    pub db: BotDb,
    pub trackscape_base_api: String,
    pub trackscape_api_web_client: ApiWebClient,
    pub server_count: AtomicUsize,
}
pub type Error = anyhow::Error;
pub type Context<'a> = poise::Context<'a, Data, Error>;

fn event_handler<'a>(
    framework: poise::FrameworkContext<'a, Data, Error>,
    event: &'a serenity::FullEvent,
) -> poise::BoxFuture<'a, Result<(), Error>> {
    Box::pin(async move {
        let ctx = framework.serenity_context;
        let data = framework.user_data;
        match event {
            serenity::FullEvent::GuildCreate { guild, is_new } => {
                events::guild_create(ctx, data, guild, *is_new).await
            }
            serenity::FullEvent::GuildDelete { incomplete, full } => {
                events::guild_delete(data, incomplete, full.as_ref()).await
            }
            serenity::FullEvent::Message { new_message } => {
                events::message(ctx, data, new_message).await
            }
            _ => {}
        }
        Ok(())
    })
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // FrameworkError's Display leaves out the underlying cause, so log it here
    match &error {
        poise::FrameworkError::Command {
            error: cause, ctx, ..
        } => error!(
            "Error in command `{}`: {:?}",
            ctx.command().qualified_name,
            cause
        ),
        poise::FrameworkError::Setup { error: cause, .. }
        | poise::FrameworkError::EventHandler { error: cause, .. } => {
            error!("Bot framework error: {}: {:?}", error, cause)
        }
        other => error!("Bot framework error: {}", other),
    }
    if let Err(e) = poise::builtins::on_error(error).await {
        error!("Error while handling error: {}", e)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set!");
    let api_base = env::var("TRACKSCAPE_API_BASE").expect("TRACKSCAPE_API_BASE not set!");
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set!");
    let trackscape_api_token = env::var("MANAGEMENT_API_KEY").expect("MANAGEMENT_API_KEY not set!");
    let dev_guild_id = match env::var("DEV_GUILD_ID") {
        Ok(id) => Some(id.parse::<u64>().expect("DEV_GUILD_ID is not a number")),
        Err(_) => None,
    };

    let db = BotDb::new_db_instance(db_url)
        .await
        .expect("Could not connect to the Postgres database");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILDS;
    let api_client = ApiWebClient::new(api_base.clone(), trackscape_api_token);

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::clan_chat_embeds_command::clan_chat_embeds_command(),
                commands::clear_channel::clear_channel(),
                commands::expel_clanmate_command::expel_clanmate_command(),
                commands::get_custom_drop_broadcast_filter::get_custom_drop_broadcast_filter(),
                commands::get_verification_code::get_verification_code(),
                commands::hide_from_website_command::hide_from_website_command(),
                commands::info::info(),
                commands::manually_run_wom_sync_command::manually_run_wom_sync_command(),
                commands::name_change_command::name_change_command(),
                commands::reset_broadcasts_thresholds::reset_broadcasts_thresholds(),
                commands::reset_verification_code::reset_verification_code(),
                commands::set_broadcast_channel::set_broadcast_channel(),
                commands::set_clan_chat_channel::set_clan_chat_channel(),
                commands::set_clog_max_percentage::set_clog_max_percentage(),
                commands::set_custom_drop_broadcast_filter::set_custom_drop_broadcast_filter(),
                commands::set_diary_min_command::set_diary_min_command(),
                commands::set_leagues_broadcast_channel::set_leagues_broadcast_channel(),
                commands::set_quest_min_command::set_quest_min_command(),
                commands::set_threshold_command::set_threshold_command(),
                commands::set_wom_id_command::set_wom_id_command(),
                commands::stop_leagues_notifications::stop_leagues_notifications(),
                commands::toggle_broadcasts_command::toggle_broadcasts_command(),
            ],
            event_handler,
            on_error: |error| Box::pin(on_error(error)),
            // Slash only bot, a message that starts with a mention is not a command
            prefix_options: poise::PrefixFrameworkOptions {
                mention_as_prefix: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                println!("{} is connected!", ready.user.name);
                let commands = &framework.options().commands;
                // A failed registration is only logged, the bot still needs its Data to run
                match dev_guild_id {
                    Some(dev_guild_id) => {
                        let guild_id = serenity::GuildId::new(dev_guild_id);
                        let result =
                            poise::builtins::register_in_guild(ctx, commands, guild_id).await;
                        if let Err(e) = result {
                            error!("Error creating guild commands: {}, for: {}", e, guild_id)
                        }
                    }
                    None => match poise::builtins::register_globally(ctx, commands).await {
                        Ok(_) => {
                            info!("Created global commands");
                        }
                        Err(e) => {
                            error!("Error creating global commands: {}", e)
                        }
                    },
                }
                Ok(Data {
                    db,
                    trackscape_base_api: api_base,
                    trackscape_api_web_client: api_client,
                    server_count: AtomicUsize::new(0),
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(&token, intents)
        .framework(framework)
        .await
        .expect("Err creating client");

    client.start().await?;

    Ok(())
}
