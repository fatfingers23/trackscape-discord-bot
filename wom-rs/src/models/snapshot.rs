use crate::models::global_enums::{Activity, Boss, ComputedMetricEnum, Skill};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skills {
    pub overall: SkillMetric,
    pub attack: SkillMetric,
    pub defence: SkillMetric,
    pub strength: SkillMetric,
    pub hitpoints: SkillMetric,
    pub ranged: SkillMetric,
    pub prayer: SkillMetric,
    pub magic: SkillMetric,
    pub cooking: SkillMetric,
    pub woodcutting: SkillMetric,
    pub fletching: SkillMetric,
    pub fishing: SkillMetric,
    pub firemaking: SkillMetric,
    pub crafting: SkillMetric,
    pub smithing: SkillMetric,
    pub mining: SkillMetric,
    pub herblore: SkillMetric,
    pub agility: SkillMetric,
    pub thieving: SkillMetric,
    pub slayer: SkillMetric,
    pub farming: SkillMetric,
    pub runecrafting: SkillMetric,
    pub hunter: SkillMetric,
    pub construction: SkillMetric,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillMetric {
    pub metric: Skill,
    pub experience: i64,
    pub rank: i64,
    pub level: i64,
    pub ehp: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Bosses {
    pub abyssal_sire: BossMetric,
    pub alchemical_hydra: BossMetric,
    pub artio: BossMetric,
    pub barrows_chests: BossMetric,
    pub bryophyta: BossMetric,
    pub callisto: BossMetric,
    pub calvarion: BossMetric,
    pub cerberus: BossMetric,
    pub chambers_of_xeric: BossMetric,
    pub chambers_of_xeric_challenge_mode: BossMetric,
    pub chaos_elemental: BossMetric,
    pub chaos_fanatic: BossMetric,
    pub commander_zilyana: BossMetric,
    pub corporeal_beast: BossMetric,
    pub crazy_archaeologist: BossMetric,
    pub dagannoth_prime: BossMetric,
    pub dagannoth_rex: BossMetric,
    pub dagannoth_supreme: BossMetric,
    pub deranged_archaeologist: BossMetric,
    pub duke_sucellus: BossMetric,
    pub general_graardor: BossMetric,
    pub giant_mole: BossMetric,
    pub grotesque_guardians: BossMetric,
    pub hespori: BossMetric,
    pub kalphite_queen: BossMetric,
    pub king_black_dragon: BossMetric,
    pub kraken: BossMetric,
    pub kreearra: BossMetric,
    pub kril_tsutsaroth: BossMetric,
    pub mimic: BossMetric,
    pub nex: BossMetric,
    pub nightmare: BossMetric,
    pub phosanis_nightmare: BossMetric,
    pub obor: BossMetric,
    pub phantom_muspah: BossMetric,
    pub sarachnis: BossMetric,
    pub scorpia: BossMetric,
    pub scurrius: BossMetric,
    pub skotizo: BossMetric,
    pub spindel: BossMetric,
    pub tempoross: BossMetric,
    pub the_gauntlet: BossMetric,
    pub the_corrupted_gauntlet: BossMetric,
    pub the_leviathan: BossMetric,
    pub the_whisperer: BossMetric,
    pub theatre_of_blood: BossMetric,
    pub theatre_of_blood_hard_mode: BossMetric,
    pub thermonuclear_smoke_devil: BossMetric,
    pub tombs_of_amascut: BossMetric,
    pub tombs_of_amascut_expert: BossMetric,
    pub tzkal_zuk: BossMetric,
    pub tztok_jad: BossMetric,
    pub vardorvis: BossMetric,
    pub venenatis: BossMetric,
    pub vetion: BossMetric,
    pub vorkath: BossMetric,
    pub wintertodt: BossMetric,
    pub zalcano: BossMetric,
    pub zulrah: BossMetric,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BossMetric {
    pub metric: Boss,
    pub kills: i64,
    pub rank: i64,
    pub ehb: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Activities {
    pub league_points: ActivityMetric,
    pub bounty_hunter_hunter: ActivityMetric,
    pub bounty_hunter_rogue: ActivityMetric,
    pub clue_scrolls_all: ActivityMetric,
    pub clue_scrolls_beginner: ActivityMetric,
    pub clue_scrolls_easy: ActivityMetric,
    pub clue_scrolls_medium: ActivityMetric,
    pub clue_scrolls_hard: ActivityMetric,
    pub clue_scrolls_elite: ActivityMetric,
    pub clue_scrolls_master: ActivityMetric,
    pub last_man_standing: ActivityMetric,
    pub pvp_arena: ActivityMetric,
    pub soul_wars_zeal: ActivityMetric,
    pub guardians_of_the_rift: ActivityMetric,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityMetric {
    pub metric: Activity,
    pub score: i64,
    pub rank: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Computed {
    pub ehp: ComputedMetric,
    pub ehb: ComputedMetric,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputedMetric {
    pub metric: ComputedMetricEnum,
    pub value: f64,
    pub rank: i64,
}
