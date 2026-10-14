use include_dir::{Dir, include_dir};
use log::{error, info};
use serenity::builder::CreateAttachment;
use serenity::http::Http;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// The clan rank icons, one png per emoji, named after the emoji.
/// Filled by scripts/fetch_clan_rank_icons.py.
static ICONS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/clan_rank_icons");

/// The emoji for lines the clan itself sends, like broadcasts.
pub const CLAN_EMOJI_NAME: &str = "clan_icon";

/// The name of the emoji for an in-game clan rank title. Discord only allows
/// letters, digits and underscores, so "Deputy Owner" is rank_deputy_owner.
pub fn rank_emoji_name(rank: &str) -> String {
    let name: String = rank
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("rank_{}", name)
}

/// Every icon in the repo as (emoji name, png).
fn icons() -> impl Iterator<Item = (&'static str, &'static [u8])> {
    ICONS.files().filter_map(|file| {
        let path = file.path();
        if path.extension()?.to_str()? != "png" {
            return None;
        }
        Some((path.file_stem()?.to_str()?, file.contents()))
    })
}

/// The clan rank emojis the bot's Discord application owns, by name. The ids
/// are different for every application (dev and production bots), so they are
/// read from Discord at startup instead of being written down anywhere.
#[derive(Clone, Default)]
pub struct RankEmojis {
    ids: Arc<RwLock<HashMap<String, u64>>>,
}

impl RankEmojis {
    pub fn set(&self, name: String, id: u64) {
        self.ids.write().unwrap().insert(name, id);
    }

    fn emoji(&self, name: &str) -> Option<String> {
        let id = *self.ids.read().unwrap().get(name)?;
        Some(format!("<:{}:{}>", name, id))
    }

    /// The emoji to write in a message for a rank, if Discord has it.
    pub fn for_rank(&self, rank: &str) -> Option<String> {
        self.emoji(&rank_emoji_name(rank))
    }

    pub fn for_clan(&self) -> Option<String> {
        self.emoji(CLAN_EMOJI_NAME)
    }

    fn missing_icons(&self) -> impl Iterator<Item = (&'static str, &'static [u8])> {
        let ids = self.ids.read().unwrap();
        icons()
            .filter(|(name, _)| !ids.contains_key(*name))
            .collect::<Vec<_>>()
            .into_iter()
    }

    async fn load(&self, http: &Http) {
        match http.get_application_emojis().await {
            Ok(emojis) => {
                for emoji in emojis {
                    self.set(emoji.name, emoji.id.get());
                }
            }
            Err(e) => error!("Failed to list the bot's emojis: {:?}", e),
        }
    }

    /// Reads the emojis the application already has and uploads the icons it
    /// is missing. The first run uploads all of them, which takes a while
    /// under Discord's rate limits, so run this in the background. Until an
    /// emoji is known, lines for that rank are just sent without one.
    pub async fn sync(&self, http: &Http) {
        match http.get_current_application_info().await {
            Ok(application) => http.set_application_id(application.id),
            Err(e) => {
                error!("Failed to get the bot's application for emojis: {:?}", e);
                return;
            }
        }
        self.load(http).await;

        let missing: Vec<_> = self.missing_icons().collect();
        if missing.is_empty() {
            return;
        }
        info!("Uploading {} clan rank emojis to Discord", missing.len());
        for (name, image) in missing {
            let image = CreateAttachment::bytes(image, format!("{}.png", name)).to_base64();
            let new_emoji = serde_json::json!({ "name": name, "image": image });
            match http.create_application_emoji(&new_emoji).await {
                Ok(emoji) => self.set(emoji.name, emoji.id.get()),
                Err(e) => error!("Failed to upload the emoji {}: {:?}", name, e),
            }
        }
        // Another instance of the API may have been uploading the same ones
        self.load(http).await;
        info!("Finished uploading clan rank emojis");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_emoji_name_is_lowercase_with_underscores() {
        assert_eq!(rank_emoji_name("Owner"), "rank_owner");
        assert_eq!(rank_emoji_name("Deputy Owner"), "rank_deputy_owner");
        assert_eq!(rank_emoji_name("Speed-Runner"), "rank_speed_runner");
        assert_eq!(rank_emoji_name("TzTok"), "rank_tztok");
        // The game sends names with non-breaking spaces
        assert_eq!(rank_emoji_name("Gnome\u{a0}Child"), "rank_gnome_child");
    }

    #[test]
    fn every_icon_has_a_name_discord_accepts_for_an_emoji() {
        let icons: Vec<(&str, &[u8])> = icons().collect();
        assert_eq!(icons.len(), 271);
        for (name, image) in icons {
            assert!(
                name.len() >= 2
                    && name.len() <= 32
                    && name
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "{} is not a valid emoji name",
                name
            );
            assert!(image.starts_with(b"\x89PNG"), "{} is not a png", name);
        }
    }

    #[test]
    fn ranks_the_game_sends_have_an_icon() {
        let names: Vec<&str> = icons().map(|(name, _)| name).collect();
        for rank in [
            "Owner",
            "Deputy Owner",
            "Guest",
            "Speed-Runner",
            "Red Topaz",
        ] {
            assert!(names.contains(&rank_emoji_name(rank).as_str()), "{}", rank);
        }
        assert!(names.contains(&CLAN_EMOJI_NAME));
    }

    #[test]
    fn uploaded_rank_is_written_as_a_discord_emoji() {
        let emojis = RankEmojis::default();
        emojis.set("rank_deputy_owner".to_string(), 123);
        assert_eq!(
            emojis.for_rank("Deputy Owner"),
            Some("<:rank_deputy_owner:123>".to_string())
        );
    }

    #[test]
    fn rank_that_is_not_uploaded_has_no_emoji() {
        let emojis = RankEmojis::default();
        emojis.set("rank_owner".to_string(), 123);
        assert_eq!(emojis.for_rank("Some New Rank"), None);
        assert_eq!(emojis.for_clan(), None);
    }

    #[test]
    fn clan_broadcast_lines_use_the_clan_icon() {
        let emojis = RankEmojis::default();
        emojis.set(CLAN_EMOJI_NAME.to_string(), 9);
        assert_eq!(emojis.for_clan(), Some("<:clan_icon:9>".to_string()));
    }

    #[test]
    fn only_icons_discord_does_not_have_yet_are_uploaded() {
        let emojis = RankEmojis::default();
        emojis.set("rank_owner".to_string(), 123);
        let missing: Vec<&str> = emojis.missing_icons().map(|(name, _)| name).collect();
        assert_eq!(missing.len(), 270);
        assert!(!missing.contains(&"rank_owner"));
    }
}
