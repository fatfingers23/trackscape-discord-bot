use crate::commands::choices::FilterBroadcast;
use crate::commands::guild_id;
use crate::{Context, Error};
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;

/// Gets the filter list being used to filter broadcasts for the selected broadcast type.
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn get_custom_drop_broadcast_filter(
    ctx: Context<'_>,
    #[description = "Broadcast type to return the filter list for."] broadcast: FilterBroadcast,
) -> Result<(), Error> {
    let guild_id = guild_id(ctx)?;

    let reply = match ctx.data().db.guilds.get_by_guild_id(guild_id).await {
        Ok(saved_guild) => {
            let saved_guild = saved_guild.unwrap_or(RegisteredGuildModel::new(guild_id));
            let broadcast_type = BroadcastType::from(broadcast);

            match saved_guild.custom_drop_broadcast_filter {
                Some(ref filter_map) => match filter_map.get(&broadcast_type) {
                    Some(filter_list) => {
                        filter_list_reply(&broadcast_type.to_string(), filter_list)
                    }
                    None => format!(
                        "No filter list found for {}. Please set a new filter list.",
                        broadcast_type.to_string()
                    ),
                },
                None => "Error finding filter lists, try again later.".to_string(),
            }
        }
        Err(_) => "There was a technical error. Please try again later.".to_string(),
    };
    ctx.say(reply).await?;
    Ok(())
}

/// How many bytes of the joined filter list are shown, well under Discord's 2000 character limit
const MAX_SHOWN_FILTER_LEN: usize = 1500;

/// Builds the reply that lists a filter, cutting the list short when it would not fit in a message.
fn filter_list_reply(broadcast_name: &str, filter_list: &[String]) -> String {
    let joined = filter_list.join(", ");
    if joined.len() <= MAX_SHOWN_FILTER_LEN {
        return format!(
            "The current {} filter list is: `{}`",
            broadcast_name, joined
        );
    }
    let end = joined.floor_char_boundary(MAX_SHOWN_FILTER_LEN);
    format!(
        "The current {} filter list has {} entries in total, too many to show them all. It starts with: `{}...`",
        broadcast_name,
        filter_list.len(),
        &joined[..end]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_list_reply_shows_a_short_list_in_full() {
        let filter_list = vec!["Twisted bow".to_string(), "Scythe of vitur".to_string()];
        assert_eq!(
            filter_list_reply("Item Drop", &filter_list),
            "The current Item Drop filter list is: `Twisted bow, Scythe of vitur`"
        );
    }

    #[test]
    fn filter_list_reply_cuts_a_long_list_and_gives_the_total() {
        let filter_list: Vec<String> = (0..1000).map(|i| format!("item {}", i)).collect();
        let reply = filter_list_reply("Item Drop", &filter_list);
        assert!(reply.chars().count() < 2000);
        assert!(reply.contains("1000 entries in total"));
        assert!(reply.contains("item 0, item 1, "));
    }

    #[test]
    fn filter_list_reply_cuts_a_long_multi_byte_list_on_a_char_boundary() {
        // 1499 ASCII bytes, then a two byte character covering bytes 1499..1501
        let filter_list = vec![format!("{}\u{e9}{}", "a".repeat(1499), "b".repeat(1000))];
        let reply = filter_list_reply("Item Drop", &filter_list);
        assert!(reply.chars().count() < 2000);
        assert!(reply.ends_with(&format!("`{}...`", "a".repeat(1499))));
    }
}
