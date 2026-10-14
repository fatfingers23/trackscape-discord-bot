pub mod choices;
pub mod clear_channel;
pub mod expel_clanmate_command;
pub mod get_custom_drop_broadcast_filter;
pub mod get_verification_code;
pub mod hide_from_website_command;
pub mod info;
pub(crate) mod manually_run_wom_sync_command;
pub mod name_change_command;
pub mod reset_broadcasts_thresholds;
pub mod reset_verification_code;
pub mod set_broadcast_channel;
pub mod set_clan_chat_channel;
pub mod set_clog_max_percentage;
pub mod set_custom_drop_broadcast_filter;
pub mod set_diary_min_command;
pub mod set_leagues_broadcast_channel;
pub mod set_quest_min_command;
pub mod set_threshold_command;
pub mod set_wom_id_command;
pub mod stop_leagues_notifications;
pub mod toggle_broadcasts_command;

use crate::{Context, Error};

/// The id of the guild the command was used in. Every `guild_only` command has one.
pub fn guild_id(ctx: Context<'_>) -> Result<u64, Error> {
    ctx.guild_id()
        .map(|guild_id| guild_id.get())
        .ok_or_else(|| anyhow::anyhow!("This command can only be used in a server."))
}
