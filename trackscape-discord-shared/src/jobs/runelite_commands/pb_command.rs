use super::get_runelite_api_url;
use crate::{database::clan_mates::ClanMates, jobs::job_helpers::get_mongodb};
use anyhow::{anyhow, Ok};
use capitalize::Capitalize;
use rand::Rng;
use reqwest::StatusCode;
use tokio::time::sleep;

pub async fn get_pb(message: String, player: String, guild_id: u64) -> Result<(), anyhow::Error> {
    //Let their message make it to runelite caches before we try to get the pb
    sleep(tokio::time::Duration::from_secs(5)).await;

    let boss = get_boss_long_name(&message);
    if boss.is_empty() {
        println!("Could not find boss name in message: {}", message);
        return Err(anyhow!("Could not find boss name in message: {}", message));
    }
    let trimmed_boss = boss.trim();
    //Should match whats in RL with spaces and each capitalized.
    println!("Long Boss name is: {}", trimmed_boss);

    //Easter egg
    if trimmed_boss.to_lowercase() == "nerdicus" {
        let random_pb: f64 = rand::thread_rng().gen_range(0.50..10_000.00);
        let random_pb_round = (random_pb * 100f64).trunc() / 100.0;
        let _ = log_pb(trimmed_boss, player, guild_id, random_pb_round).await?; // Await the log_pb function call and assign the result to a variable
        return Ok(());
    }

    let runelite_api_url = get_runelite_api_url().await?;
    let full_url = format!(
        "{}/chat/pb?name={}&boss={}",
        runelite_api_url, player, trimmed_boss
    );
    let pb_request: reqwest::Response = reqwest::get(full_url).await?;
    if pb_request.status() != StatusCode::OK {
        println!(
            "Failed to get pb from runelite api: {}",
            pb_request.status()
        );
        return Err(anyhow!(
            "Failed to get pb from runelite api: {}",
            pb_request.status()
        ));
    }
    let pb_time = pb_request.text().await?.parse::<f64>()?;
    println!("PB: {}", pb_time);
    log_pb(trimmed_boss, player, guild_id, pb_time).await?;
    Ok(())
}

async fn log_pb(
    boss: &str,
    player: String,
    guild_id: u64,
    pb_time: f64,
) -> Result<(), anyhow::Error> {
    let db = get_mongodb().await;
    let activity = db
        .pb_activities
        .create_or_get_activity(boss.to_string())
        .await?;
    let clan_mate = db
        .clan_mates
        .find_or_create_clan_mate(guild_id, player)
        .await;
    let _ = db
        .pb_records
        .create_or_update_pb_record(clan_mate.unwrap().id, activity.id, guild_id, pb_time)
        .await?;
    Ok(())
}

fn get_boss_long_name(message: &String) -> String {
    let binding = message.chars().skip(3).collect::<String>();
    let boss = binding.trim();
    let lower_case = boss.to_lowercase();
    let mut match_found = true;
    let matched = match lower_case.as_str() {
        "corp" => "Corporeal Beast",
        "jad" | "tzhaar fight cave" => "TzTok-Jad",
        "kq" => "Kalphite Queen",
        "chaos ele" => "Chaos Elemental",
        "dusk" | "dawn" | "gargs" | "ggs" | "gg" => "Grotesque Guardians",
        "crazy arch" => "Crazy Archaeologist",
        "deranged arch" => "Deranged Archaeologist",
        "mole" => "Giant Mole",
        "vetion" => "Vet'ion",
        "calvarion" | "calv" => "Calvar'ion",
        "vene" => "Venenatis",
        "kbd" => "King Black Dragon",
        "vork" => "Vorkath",
        "sire" => "Abyssal Sire",
        "smoke devil" | "thermy" => "Thermonuclear Smoke Devil",
        "cerb" => "Cerberus",
        "zuk" | "inferno" => "TzKal-Zuk",
        "hydra" => "Alchemical Hydra",

        // gwd
        "sara" | "saradomin" | "zilyana" | "zily" => "Commander Zilyana",
        "zammy" | "zamorak" | "kril" | "kril tsutsaroth" => "K'ril Tsutsaroth",
        "arma" | "kree" | "kreearra" | "armadyl" => "Kree'arra",
        "bando" | "bandos" | "graardor" => "General Graardor",

        // dks
        "supreme" => "Dagannoth Supreme",
        "rex" => "Dagannoth Rex",
        "prime" => "Dagannoth Prime",

        "wt" => "Wintertodt",
        "barrows" => "Barrows Chests",
        "herbi" => "Herbiboar",

        // Chambers of Xeric
        "cox" | "xeric" | "chambers" | "olm" | "raids" => "Chambers of Xeric",
        "cox 1" | "cox solo" => "Chambers of Xeric Solo",
        "cox 2" | "cox duo" => "Chambers of Xeric 2 players",
        "cox 3" => "Chambers of Xeric 3 players",
        "cox 4" => "Chambers of Xeric 4 players",
        "cox 5" => "Chambers of Xeric 5 players",
        "cox 6" => "Chambers of Xeric 6 players",
        "cox 7" => "Chambers of Xeric 7 players",
        "cox 8" => "Chambers of Xeric 8 players",
        "cox 9" => "Chambers of Xeric 9 players",
        "cox 10" => "Chambers of Xeric 10 players",
        "cox 11-15" | "cox 11" | "cox 12" | "cox 13" | "cox 14" | "cox 15" => {
            "Chambers of Xeric 11-15 players"
        }
        "cox 16-23" | "cox 16" | "cox 17" | "cox 18" | "cox 19" | "cox 20" | "cox 21"
        | "cox 22" | "cox 23" => "Chambers of Xeric 16-23 players",
        "cox 24" | "cox 24+" => "Chambers of Xeric 24+ players",

        // Chambers of Xeric Challenge Mode
        "chambers of xeric| challenge mode"
        | "cox cm"
        | "xeric cm"
        | "chambers cm"
        | "olm cm"
        | "raids cm"
        | "chambers of xeric - challenge mode" => "Chambers of Xeric Challenge Mode",
        "cox cm 1" | "cox cm solo" => "Chambers of Xeric Challenge Mode Solo",
        "cox cm 2" | "cox cm duo" => "Chambers of Xeric Challenge Mode 2 players",
        "cox cm 3" => "Chambers of Xeric Challenge Mode 3 players",
        "cox cm 4" => "Chambers of Xeric Challenge Mode 4 players",
        "cox cm 5" => "Chambers of Xeric Challenge Mode 5 players",
        "cox cm 6" => "Chambers of Xeric Challenge Mode 6 players",
        "cox cm 7" => "Chambers of Xeric Challenge Mode 7 players",
        "cox cm 8" => "Chambers of Xeric Challenge Mode 8 players",
        "cox cm 9" => "Chambers of Xeric Challenge Mode 9 players",
        "cox cm 10" => "Chambers of Xeric Challenge Mode 10 players",
        "cox cm 11-15" | "cox cm 11" | "cox cm 12" | "cox cm 13" | "cox cm 14" | "cox cm 15" => {
            "Chambers of Xeric Challenge Mode 11-15 players"
        }
        "cox cm 16-23" | "cox cm 16" | "cox cm 17" | "cox cm 18" | "cox cm 19" | "cox cm 20"
        | "cox cm 21" | "cox cm 22" | "cox cm 23" => {
            "Chambers of Xeric Challenge Mode 16-23 players"
        }
        "cox cm 24" | "cox cm 24+" => "Chambers of Xeric Challenge Mode 24+ players",

        // Theatre of Blood
        "tob" | "theatre" | "verzik" | "verzik vitur" | "raids 2" => "Theatre of Blood",
        "tob 1" | "tob solo" => "Theatre of Blood Solo",
        "tob 2" | "tob duo" => "Theatre of Blood 2 players",
        "tob 3" => "Theatre of Blood 3 players",
        "tob 4" => "Theatre of Blood 4 players",
        "tob 5" => "Theatre of Blood 5 players",

        // Theatre of Blood Entry Mode
        "theatre of blood| story mode"
        | "tob sm"
        | "tob story mode"
        | "tob story"
        | "theatre of blood| entry mode"
        | "tob em"
        | "tob entry mode"
        | "tob entry" => "Theatre of Blood Entry Mode",

        // Theatre of Blood Hard Mode
        "theatre of blood| hard mode"
        | "tob cm"
        | "tob hm"
        | "tob hard mode"
        | "tob hard"
        | "hmt" => "Theatre of Blood Hard Mode",
        "hmt 1" | "hmt solo" => "Theatre of Blood Hard Mode Solo",
        "hmt 2" | "hmt duo" => "Theatre of Blood Hard Mode 2 players",
        "hmt 3" => "Theatre of Blood Hard Mode 3 players",
        "hmt 4" => "Theatre of Blood Hard Mode 4 players",
        "hmt 5" => "Theatre of Blood Hard Mode 5 players",

        // Tombs of Amascut
        "toa" | "tombs" | "amascut" | "warden" | "wardens" | "raids 3" => "Tombs of Amascut",
        "toa 1" | "toa solo" => "Tombs of Amascut Solo",
        "toa 2" | "toa duo" => "Tombs of Amascut 2 players",
        "toa 3" => "Tombs of Amascut 3 players",
        "toa 4" => "Tombs of Amascut 4 players",
        "toa 5" => "Tombs of Amascut 5 players",
        "toa 6" => "Tombs of Amascut 6 players",
        "toa 7" => "Tombs of Amascut 7 players",
        "toa 8" => "Tombs of Amascut 8 players",
        "toa entry" | "tombs of amascut - entry" | "toa entry mode" => {
            "Tombs of Amascut Entry Mode"
        }
        "toa entry 1" | "toa entry solo" => "Tombs of Amascut Entry Mode Solo",
        "toa entry 2" | "toa entry duo" => "Tombs of Amascut Entry Mode 2 players",
        "toa entry 3" => "Tombs of Amascut Entry Mode 3 players",
        "toa entry 4" => "Tombs of Amascut Entry Mode 4 players",
        "toa entry 5" => "Tombs of Amascut Entry Mode 5 players",
        "toa entry 6" => "Tombs of Amascut Entry Mode 6 players",
        "toa entry 7" => "Tombs of Amascut Entry Mode 7 players",
        "toa entry 8" => "Tombs of Amascut Entry Mode 8 players",
        "tombs of amascut| expert mode"
        | "toa expert"
        | "tombs of amascut - expert"
        | "toa expert mode" => "Tombs of Amascut Expert Mode",
        "toa expert 1" | "toa expert solo" => "Tombs of Amascut Expert Mode Solo",
        "toa expert 2" | "toa expert duo" => "Tombs of Amascut Expert Mode 2 players",
        "toa expert 3" => "Tombs of Amascut Expert Mode 3 players",
        "toa expert 4" => "Tombs of Amascut Expert Mode 4 players",
        "toa expert 5" => "Tombs of Amascut Expert Mode 5 players",
        "toa expert 6" => "Tombs of Amascut Expert Mode 6 players",
        "toa expert 7" => "Tombs of Amascut Expert Mode 7 players",
        "toa expert 8" => "Tombs of Amascut Expert Mode 8 players",
        // The Gauntlet
        "gaunt" | "gauntlet" | "the gauntlet" => "Gauntlet",
        // Corrupted Gauntlet
        "cgaunt" | "cgauntlet" | "the corrupted gauntlet" | "cg" => "Corrupted Gauntlet",
        // The Nightmare
        "nm" | "tnm" | "nmare" | "the nightmare" => "Nightmare",

        // Phosani's Nightmare
        "pnm" | "phosani" | "phosanis" | "phosani nm" | "phosani nightmare"
        | "phosanis nightmare" => "Phosani's Nightmare",

        // Hallowed Sepulchre
        "hs" | "sepulchre" | "ghc" => "Hallowed Sepulchre",
        "hs1" | "hs 1" => "Hallowed Sepulchre Floor 1",
        "hs2" | "hs 2" => "Hallowed Sepulchre Floor 2",
        "hs3" | "hs 3" => "Hallowed Sepulchre Floor 3",
        "hs4" | "hs 4" => "Hallowed Sepulchre Floor 4",
        "hs5" | "hs 5" => "Hallowed Sepulchre Floor 5",

        // Prifddinas Agility Course
        "prif" | "prifddinas" => "Prifddinas Agility Course",

        // Shayzien Basic Agility Course
        "shayb" | "sbac" | "shayzienbasic" | "shayzien basic" => "Shayzien Basic Agility Course",

        // Shayzien Advanced Agility Course
        "shaya" | "saac" | "shayadv" | "shayadvanced" | "shayzien advanced" => {
            "Shayzien Advanced Agility Course"
        }

        // Ape Atoll Agility
        "aa" | "ape atoll" => "Ape Atoll Agility",

        // Draynor Village Rooftop Course
        "draynor" | "draynor agility" => "Draynor Village Rooftop",

        // Al-Kharid Rooftop Course
        "al kharid" | "al kharid agility" | "al-kharid" | "al-kharid agility" | "alkharid"
        | "alkharid agility" => "Al Kharid Rooftop",

        // Varrock Rooftop Course
        "varrock" | "varrock agility" => "Varrock Rooftop",

        // Canifis Rooftop Course
        "canifis" | "canifis agility" => "Canifis Rooftop",

        // Falador Rooftop Course
        "fally" | "fally agility" | "falador" | "falador agility" => "Falador Rooftop",

        // Seers' Village Rooftop Course
        "seers"
        | "seers agility"
        | "seers village"
        | "seers village agility"
        | "seers'"
        | "seers' agility"
        | "seers' village"
        | "seers' village agility"
        | "seer's"
        | "seer's agility"
        | "seer's village"
        | "seer's village agility" => "Seers' Village Rooftop",

        // Pollnivneach Rooftop Course
        "pollnivneach" | "pollnivneach agility" => "Pollnivneach Rooftop",

        // Rellekka Rooftop Course
        "rellekka" | "rellekka agility" => "Rellekka Rooftop",

        // Ardougne Rooftop Course
        "ardy" | "ardy agility" | "ardy rooftop" | "ardougne" | "ardougne agility" => {
            "Ardougne Rooftop"
        }

        // Agility Pyramid
        "ap" | "pyramid" => "Agility Pyramid",

        // Barbarian Outpost
        "barb" | "barb outpost" => "Barbarian Outpost",

        // Brimhaven Agility Arena
        "brimhaven" | "brimhaven agility" => "Agility Arena",

        // Dorgesh-Kaan Agility Course
        "dorg" | "dorgesh kaan" | "dorgesh-kaan" => "Dorgesh-Kaan Agility",

        // Gnome Stronghold Agility Course
        "gnome stronghold" => "Gnome Stronghold Agility",

        // Penguin Agility
        "penguin" => "Penguin Agility",

        // Werewolf Agility
        "werewolf" => "Werewolf Agility",

        // Skullball
        "skullball" => "Werewolf Skullball",

        // Wilderness Agility Course
        "wildy" | "wildy agility" => "Wilderness Agility",

        // Jad challenge
        "jad 1" => "TzHaar-Ket-Rak's First Challenge",
        "jad 2" => "TzHaar-Ket-Rak's Second Challenge",
        "jad 3" => "TzHaar-Ket-Rak's Third Challenge",
        "jad 4" => "TzHaar-Ket-Rak's Fourth Challenge",
        "jad 5" => "TzHaar-Ket-Rak's Fifth Challenge",
        "jad 6" => "TzHaar-Ket-Rak's Sixth Challenge",

        // Guardians of the Rift
        "gotr" | "runetodt" | "rifts closed" => "Guardians of the Rift",

        // Tempoross
        "fishingtodt" | "fishtodt" => "Tempoross",

        // Phantom Muspah
        "phantom" | "muspah" | "pm" => "Phantom Muspah",

        // Desert Treasure 2 bosses
        "the leviathan" | "levi" => "Leviathan",
        "duke" => "Duke Sucellus",
        "the whisperer" | "whisp" | "wisp" => "Whisperer",
        "vard" => "Vardorvis",

        // dt2 awakened variants
        "leviathan awakened" | "the leviathan awakened" | "levi awakened" => "Leviathan (awakened)",
        "duke sucellus awakened" | "duke awakened" => "Duke Sucellus (awakened)",
        "whisperer awakened" | "the whisperer awakened" | "whisp awakened" | "wisp awakened" => {
            "Whisperer (awakened)"
        }
        "vardorvis awakened" | "vard awakened" => "Vardorvis (awakened)",

        // lunar chest variants
        "lunar chests" | "perilous moons" | "perilous moon" | "moons of peril" => "Lunar Chest",

        //TODO have to return and captlize the first letter of each word
        _ => {
            match_found = false;
            ""
        }
    };
    if match_found {
        matched.to_string()
    } else {
        let new_boss_name = boss
            .split(" ")
            .map(|word| format!("{} ", word.capitalize()))
            .collect();

        new_boss_name
    }
}
