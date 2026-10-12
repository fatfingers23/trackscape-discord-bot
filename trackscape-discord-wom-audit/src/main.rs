//! Wise Old Man connection audit / member re-sync tool (safe to re-run).
//!
//! Walks every registered clan (a guild with a `clan_name`) that has a WOM
//! group id, asks Wise Old Man whether that group still exists, and reports
//! the member counts side by side:
//!
//! * default mode: read-only. For each clan it checks the WOM group and
//!   prints the WOM member count next to the active and total number of
//!   `clan_mates` rows in Postgres, and flags clans whose WOM group id is
//!   dead. A dead group id used to silently break the twice-daily sync for
//!   every clan that came after it in the run, so this is the quick way to
//!   find the culprit.
//! * `--resync`: additionally re-runs the member sync for every clan whose
//!   WOM group is reachable. The sync is non-destructive (it creates missing
//!   members, renames via name changes, and soft-removes members that left
//!   the WOM group), with one deliberate exception: when a member renamed in
//!   WOM but their new name is already held by an unlinked (chat-created)
//!   duplicate row, the duplicate is merged into the member's row — personal
//!   bests and collection log totals move over (keeping the better values)
//!   and the duplicate row is deleted — so it is safe to run repeatedly.
//! * `--guild-id <discord_guild_id>`: restrict the run to a single clan,
//!   e.g. to re-sync just one clan without touching the rest.
//!
//! Rows without a WOM group id are reported and skipped; dead/errored WOM
//! groups are reported and never re-synced.
//!
//! The report is printed to stdout with plain `println!` so it reads cleanly
//! regardless of RUST_LOG; operational logging goes through `log`.

use anyhow::{Context, Result};
use log::{error, info};
use std::time::Duration;
use trackscape_discord_shared::database::BotDb;
use trackscape_discord_shared::database::clan_mates::ClanMates;
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;
use trackscape_discord_shared::database::run_migrations;
use trackscape_discord_shared::jobs::wom_guild_sync_logic::sync_wom_by_guild;
use trackscape_discord_shared::wom::{ApiLimiter, get_wom_client};

/// How long to wait before each WOM call. Keeping this polite matters:
/// there can be many guilds, and the audit is not in a hurry.
const WOM_WAIT_BEFORE: Duration = Duration::from_millis(200);

const USAGE: &str = "\
Usage: trackscape-discord-wom-audit [--resync] [--guild-id <discord_guild_id>]

  (no flags)            Audit every registered clan's WOM connection (read-only).
  --resync              Re-run the member sync for every clan with a reachable
                        WOM group (non-destructive; nothing is deleted).
  --guild-id <number>   Restrict to a single Discord guild id.

Reads DATABASE_URL (Postgres) and WOM_API_KEY (Wise Old Man) from the env.";

/// Parsed command line flags.
struct Options {
    resync: bool,
    guild_id: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let options = match parse_args(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}\n\n{USAGE}");
            std::process::exit(1);
        }
    };

    let db_url = std::env::var("DATABASE_URL").context("DATABASE_URL not set!")?;
    let db = BotDb::new_db_instance(db_url)
        .await
        .context("Could not connect to Postgres")?;
    // Safe to call even if the API is already running: sqlx takes an advisory
    // lock, so this just makes `--resync` work against a database whose
    // services have not picked up the latest migrations yet.
    run_migrations(db.pool())
        .await
        .context("Could not run Postgres migrations")?;

    let clans = db
        .guilds
        .list_clans()
        .await
        .context("Could not list registered clans")?;

    let clans: Vec<RegisteredGuildModel> = match options.guild_id {
        Some(guild_id) => clans
            .into_iter()
            .filter(|clan| clan.guild_id == guild_id)
            .collect(),
        None => clans,
    };

    if clans.is_empty() {
        match options.guild_id {
            Some(guild_id) => println!("No registered clan found for guild id {guild_id}."),
            None => println!("No registered clans found."),
        }
        return Ok(());
    }

    let total_clans = clans.len();
    let mut checked = 0usize;
    let mut ok = 0usize;
    let mut dead = 0usize;
    let mut no_wom_group = 0usize;

    let wom_client = get_wom_client();
    // One limiter for the whole run so the rate limit is shared across guilds.
    let mut limiter = ApiLimiter::new();

    for clan in &clans {
        let name = display_name(clan);
        let guild_id = clan.guild_id;

        let Some(wom_id) = clan.wom_id else {
            println!("{name} ({guild_id}): no WOM group connected");
            no_wom_group += 1;
            continue;
        };

        checked += 1;
        let group = limiter
            .api_limit_request(
                || async { wom_client.group_client.get_group_details(wom_id).await },
                Some(WOM_WAIT_BEFORE),
            )
            .await;

        match group {
            Ok(group) => {
                ok += 1;
                let db_active = db
                    .clan_mates
                    .get_clan_member_count(guild_id)
                    .await
                    .unwrap_or(0);
                let db_total = db
                    .clan_mates
                    .get_clan_mates_by_guild_id(guild_id)
                    .await
                    .map(|members| members.len())
                    .unwrap_or(0);

                println!(
                    "{name} ({guild_id}) wom_id={wom_id}: OK \"{}\" wom_members={} db_active={db_active} db_total={db_total}",
                    group.name, group.member_count
                );

                if options.resync {
                    info!("Re-syncing guild {guild_id} ({name})...");
                    sync_wom_by_guild(clan, &db).await;

                    let db_active_after = db
                        .clan_mates
                        .get_clan_member_count(guild_id)
                        .await
                        .unwrap_or(0);
                    println!(
                        "{name} ({guild_id}) wom_id={wom_id}: resynced, db_active={db_active_after}"
                    );
                }
            }
            Err(e) => {
                dead += 1;
                // A dead/misconfigured WOM id used to silently break the sync
                // for every clan after it, so surface it loudly and keep going.
                println!("{name} ({guild_id}) wom_id={wom_id}: DEAD/ERROR — {e}");
                error!("WOM group {wom_id} for guild {guild_id} ({name}) failed: {e:?}");
            }
        }
    }

    println!();
    println!(
        "Summary: {total_clans} clan(s) checked, {ok} OK, {dead} dead/errored, {no_wom_group} without a WOM group ({checked} with a WOM id)"
    );

    Ok(())
}

/// The clan's display name, with a stable fallback for rows whose clan name
/// was never set (the audit must still be able to point at the guild).
fn display_name(clan: &RegisteredGuildModel) -> String {
    clan.clan_name
        .clone()
        .unwrap_or_else(|| "<unnamed>".to_string())
}

/// Parses `--resync` and `--guild-id <number>`. Any other argument is an
/// error, so a typo never silently turns into a full audit/resync run.
fn parse_args(args: impl Iterator<Item = String>) -> std::result::Result<Options, String> {
    let args: Vec<String> = args.collect();
    let mut options = Options {
        resync: false,
        guild_id: None,
    };

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--resync" => options.resync = true,
            "--guild-id" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("--guild-id needs a value".to_string());
                };
                let guild_id = value
                    .parse::<u64>()
                    .map_err(|_| format!("--guild-id value '{value}' is not a number"))?;
                options.guild_id = Some(guild_id);
            }
            other => return Err(format!("Unrecognized argument: {other}")),
        }
        index += 1;
    }

    Ok(options)
}
