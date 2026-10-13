use crate::Data;
use crate::on_boarding_message::send_on_boarding;
use poise::serenity_prelude as serenity;
use serenity::{Guild, Message, UnavailableGuild};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use tracing::{error, info};

pub async fn guild_create(
    ctx: &serenity::Context,
    data: &Data,
    guild: &Guild,
    is_new: Option<bool>,
) {
    info!("Guild: {}", guild.name);
    data.db.guilds.create_if_new_guild(guild.id.get()).await;
    data.server_count.fetch_add(1, Ordering::SeqCst);
    let new_server_count = data.server_count.load(Ordering::SeqCst);
    info!("Server Count: {}", new_server_count);
    data.trackscape_api_web_client
        .send_server_count(new_server_count as i64)
        .await;

    if is_new.unwrap_or(false) {
        //This fires if it's a new guild it's been added to
        if let Some(guild_system_channel) = guild.system_channel_id {
            send_on_boarding(guild_system_channel, ctx).await;
        }
        info!(
            "Joined a new Discord Server Id: {} and name {}",
            guild.id.get(),
            guild.name
        );
    }
}

pub async fn guild_delete(data: &Data, incomplete: &UnavailableGuild, full: Option<&Guild>) {
    match full {
        None => info!("Removed from a guild that we don't have access to"),
        Some(full) => info!("Removed from the guild: {}", full.name),
    }

    if !incomplete.unavailable {
        data.db.guilds.delete_guild(incomplete.id.get()).await;
        info!("The guild mention has been deleted when removed.");
    }

    data.server_count.fetch_sub(1, Ordering::SeqCst);
    let new_server_count = data.server_count.load(Ordering::SeqCst);
    info!("Server Count: {}", new_server_count);
    data.trackscape_api_web_client
        .send_server_count(new_server_count as i64)
        .await;
}

pub async fn message(ctx: &serenity::Context, data: &Data, msg: &Message) {
    let possible_guild_id = msg.guild_id;
    if msg.author.bot {
        return;
    }
    match possible_guild_id {
        None => {}
        Some(guild_id) => {
            let guild_id = guild_id.get();
            let guild = data.db.guilds.get_by_guild_id(guild_id).await;
            match guild {
                Ok(guild) => {
                    if guild.is_none() {
                        return;
                    }
                    let unwrapped_guild = guild.unwrap();
                    if unwrapped_guild.clan_chat_channel.is_none() {
                        return;
                    }
                    if unwrapped_guild.clan_chat_channel.unwrap() == msg.channel_id.get() {
                        let mut map = HashMap::new();
                        map.insert("message", shorten_for_game(&msg.content));
                        let nick_name = msg.author_nick(&ctx.http).await;
                        match nick_name {
                            Some(nick_name) => map.insert("sender", nick_name),
                            None => map.insert("sender", msg.author.name.clone()),
                        };

                        let client = reqwest::Client::new();

                        let resp = client
                            .post(
                                format!(
                                    "{}{}",
                                    data.trackscape_base_api, "/api/chat/new-discord-message"
                                )
                                .as_str(),
                            )
                            .header("verification-code", unwrapped_guild.verification_code)
                            .json(&map)
                            .send()
                            .await;
                        if let Err(e) = resp {
                            error!("Error sending message to api: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Error getting guild: {}", e)
                }
            }
        }
    }
}

/// The max length, in bytes, of a message in game
const MAX_GAME_MESSAGE_LEN: usize = 78;

/// Cuts a Discord message down to what fits in a single in game message.
fn shorten_for_game(content: &str) -> String {
    // Back off to a char boundary, String::truncate panics in the middle of a character
    let end = content.floor_char_boundary(MAX_GAME_MESSAGE_LEN);
    content[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shorten_for_game_keeps_a_short_ascii_message() {
        assert_eq!(shorten_for_game("gz on the pet"), "gz on the pet");
    }

    #[test]
    fn shorten_for_game_cuts_a_long_ascii_message_to_78_bytes() {
        let shortened = shorten_for_game(&"a".repeat(100));
        assert_eq!(shortened, "a".repeat(78));
    }

    #[test]
    fn shorten_for_game_backs_off_when_a_character_straddles_byte_78() {
        // 77 ASCII bytes, then a four byte emoji covering bytes 77..81
        let content = format!("{}\u{1F600}tail", "a".repeat(77));
        let shortened = shorten_for_game(&content);
        assert_eq!(shortened, "a".repeat(77));
    }

    #[test]
    fn shorten_for_game_keeps_a_character_that_ends_on_byte_78() {
        // 76 ASCII bytes, then a two byte character covering bytes 76..78
        let content = format!("{}\u{e9}tail", "a".repeat(76));
        let shortened = shorten_for_game(&content);
        assert_eq!(shortened, format!("{}\u{e9}", "a".repeat(76)));
    }
}
