use crate::models::global_enums::{Activity, Boss, Skill};
use crate::models::global_types::PlayerId;
use crate::models::snapshot::{Activities, Bosses, Computed, Skills};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// [Player Type](https://docs.wiseoldman.net/players-api/player-type-definitions#enum-player-type)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlayerType {
    Unknown,
    Regular,
    Ironman,
    Hardcore,
    Ultimate,
}

/// [Player Build](https://docs.wiseoldman.net/players-api/player-type-definitions#enum-player-build)
/// WOM's wire values are snake case (`f2p_lvl3`, not `f2pLvl3`); the other
/// variants are single words, so a camel-case rename only ever worked for
/// them by coincidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerBuild {
    Main,
    F2p,
    Lvl3,
    Zerker,
    Def1,
    Hp10,
    F2pLvl3,
    /// Fallback for build codes WOM may add in the future. A single
    /// unknown build must never fail deserialization of a whole group
    /// (same rationale as the `Country` fallback below).
    #[serde(other)]
    Unknown,
}

/// [Player Status](https://docs.wiseoldman.net/players-api/player-type-definitions#enum-player-status)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlayerStatus {
    Active,
    Unranked,
    Flagged,
    Archived,
    Banned,
}

/// [Country](https://docs.wiseoldman.net/players-api/player-type-definitions#enum-country)
// Variant names mirror WOM's wire codes exactly (ISO 3166-1 alpha-2 plus
// the GB constituent-country codes), so they cannot be camel case.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Country {
    AD,
    AE,
    AF,
    AG,
    AI,
    AL,
    AM,
    AO,
    AQ,
    AR,
    AS,
    AT,
    AU,
    AW,
    AX,
    AZ,
    BA,
    BB,
    BD,
    BE,
    BF,
    BG,
    BH,
    BI,
    BJ,
    BL,
    BM,
    BN,
    BO,
    BQ,
    BR,
    BS,
    BT,
    BV,
    BW,
    BY,
    BZ,
    CA,
    CC,
    CD,
    CF,
    CG,
    CH,
    CI,
    CK,
    CL,
    CM,
    CN,
    CO,
    CR,
    CU,
    CV,
    CW,
    CX,
    CY,
    CZ,
    DE,
    DJ,
    DK,
    DM,
    DO,
    DZ,
    EC,
    EE,
    EG,
    EH,
    ER,
    ES,
    ET,
    FI,
    FJ,
    FK,
    FM,
    FO,
    FR,
    GA,
    GB,
    GB_ENG,
    GB_NIR,
    GB_SCT,
    GB_WLS,
    GD,
    GE,
    GF,
    GG,
    GH,
    GI,
    GL,
    GM,
    GN,
    GP,
    GQ,
    GR,
    GS,
    GT,
    GU,
    GW,
    GY,
    HK,
    HM,
    HN,
    HR,
    HT,
    HU,
    ID,
    IE,
    IL,
    IM,
    IN,
    IO,
    IQ,
    IR,
    IS,
    IT,
    JE,
    JM,
    JO,
    JP,
    KE,
    KG,
    KH,
    KI,
    KM,
    KN,
    KP,
    KR,
    KW,
    KY,
    KZ,
    LA,
    LB,
    LC,
    LI,
    LK,
    LR,
    LS,
    LT,
    LU,
    LV,
    LY,
    MA,
    MC,
    MD,
    ME,
    MF,
    MG,
    MH,
    MK,
    ML,
    MM,
    MN,
    MO,
    MP,
    MQ,
    MR,
    MS,
    MT,
    MU,
    MV,
    MW,
    MX,
    MY,
    MZ,
    NA,
    NC,
    NE,
    NF,
    NG,
    NI,
    NL,
    NO,
    NP,
    NR,
    NU,
    NZ,
    OM,
    PA,
    PE,
    PF,
    PG,
    PH,
    PK,
    PL,
    PM,
    PN,
    PR,
    PS,
    PT,
    PW,
    PY,
    QA,
    RE,
    RO,
    RS,
    RU,
    RW,
    SA,
    SB,
    SC,
    SD,
    SE,
    SG,
    SH,
    SI,
    SJ,
    SK,
    SL,
    SM,
    SN,
    SO,
    SR,
    SS,
    ST,
    SV,
    SX,
    SY,
    SZ,
    TC,
    TD,
    TF,
    TG,
    TH,
    TJ,
    TK,
    TL,
    TM,
    TN,
    TO,
    TR,
    TT,
    TV,
    TW,
    TZ,
    UA,
    UG,
    UM,
    US,
    UY,
    UZ,
    VA,
    VC,
    VE,
    VG,
    VI,
    VN,
    VU,
    WF,
    WS,
    YE,
    YT,
    ZA,
    ZM,
    ZW,
    /// Fallback for country codes WOM may add in the future. A single
    /// unknown code must never fail deserialization of a whole group.
    #[serde(other)]
    Unknown,
}

/// [Achievement Measure](https://docs.wiseoldman.net/players-api/player-type-definitions#enum-achievement-measure)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AchievementMeasure {
    Levels,
    Experience,
    Kills,
    Score,
    Value,
}

/// [SnapShot Data Values](https://docs.wiseoldman.net/players-api/player-type-definitions#object-snapshot-data-values)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapShotData {
    pub skills: Skills,
    pub bosses: Bosses,
    pub activities: Activities,
    pub computed: Computed,
}

/// [SnapShot](https://docs.wiseoldman.net/players-api/player-type-definitions#object-snapshot)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapShot {
    id: i64,
    player_id: i64,
    created_at: DateTime<Utc>,
    imported_at: Option<DateTime<Utc>>,
    data: SnapShotData,
}

/// [Player](https://docs.wiseoldman.net/players-api/player-type-definitions#object-player)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub id: PlayerId,
    pub username: String,
    pub display_name: String,
    #[serde(rename = "type")]
    pub player_type: PlayerType,
    pub build: PlayerBuild,
    pub country: Option<Country>,
    pub status: PlayerStatus,
    pub patron: bool,
    pub exp: i64,
    pub ehp: f64,
    pub ehb: f64,
    pub ttm: f64,
    pub tt200m: f64,
    pub registered_at: String,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_changed_at: Option<DateTime<Utc>>,
    pub last_imported_at: Option<DateTime<Utc>>,
}

/// [Player Details](https://docs.wiseoldman.net/players-api/player-type-definitions#object-player-details)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDetails {
    pub id: PlayerId,
    pub username: String,
    pub display_name: String,
    #[serde(rename = "type")]
    pub player_type: PlayerType,
    pub build: PlayerBuild,
    pub country: Option<Country>,
    pub status: PlayerStatus,
    pub patron: bool,
    pub exp: i64,
    pub ehp: f64,
    pub ehb: f64,
    pub ttm: f64,
    pub tt200m: f64,
    pub registered_at: String,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_changed_at: Option<DateTime<Utc>>,
    pub last_imported_at: Option<DateTime<Utc>>,
    pub combat_level: i64,
    pub archive: Option<PlayerArchive>,
    pub latest_snapshot: Option<SnapShot>,
}

/// [Achievement](https://docs.wiseoldman.net/players-api/player-type-definitions#object-achievement)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    player_id: i64,
    name: String,
    metric: String,
    measure: AchievementMeasure,
    threshold: i64,
    created_at: DateTime<Utc>,
    accuracy: Option<i64>,
}

/// [Achievement Progress](https://docs.wiseoldman.net/players-api/player-type-definitions#object-achievement-progress)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementProgress {
    player_id: PlayerId,
    name: String,
    metric: String,
    measure: AchievementMeasure,
    threshold: i64,
    created_at: Option<DateTime<Utc>>,
    accuracy: Option<i64>,
    current_value: i64,
    absolute_progress: f64,
    relative_progress: f64,
}

/// [Player Archive](https://docs.wiseoldman.net/players-api/player-type-definitions#object-player-archive)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerArchive {
    pub player_id: PlayerId,
    pub previous_username: String,
    pub archive_username: String,
    pub restored_username: Option<String>,
    pub created_at: DateTime<Utc>,
    pub restored_at: Option<DateTime<Utc>>,
    /// [Player Archive With Player](https://docs.wiseoldman.net/players-api/player-type-definitions#object-player-archive-with-player)
    pub player: Option<Player>,
}

/// Same as [Player] but has changed to signify if the player was changed with assertion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssertPlayerType {
    pub player: Player,
    pub changed: bool,
}

/// Represents a player's gains
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerGain {
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub data: PlayerGainData,
}

/// The data of the player's gains
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerGainData {
    pub skills: SkillGains,
    pub bosses: BossGains,
    pub activities: ActivityGains,
}

/// The gains of a player's skills
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillGains {
    pub overall: SkillGain,
    pub attack: SkillGain,
    pub defence: SkillGain,
    pub strength: SkillGain,
    pub hitpoints: SkillGain,
    pub ranged: SkillGain,
    pub prayer: SkillGain,
    pub magic: SkillGain,
    pub cooking: SkillGain,
    pub woodcutting: SkillGain,
    pub fletching: SkillGain,
    pub fishing: SkillGain,
    pub firemaking: SkillGain,
    pub crafting: SkillGain,
    pub smithing: SkillGain,
    pub mining: SkillGain,
    pub herblore: SkillGain,
    pub agility: SkillGain,
    pub thieving: SkillGain,
    pub slayer: SkillGain,
    pub farming: SkillGain,
    pub runecrafting: SkillGain,
    pub hunter: SkillGain,
    pub construction: SkillGain,
}

/// The gains of a player's skill
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillGain {
    pub metric: Skill,
    pub experience: GainProgress,
    pub ehp: GainProgress,
    pub rank: GainProgress,
    pub level: GainProgress,
}

/// The gains of a player's bosses
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BossGains {
    pub abyssal_sire: BossGain,
    pub alchemical_hydra: BossGain,
    pub artio: BossGain,
    pub barrows_chests: BossGain,
    pub bryophyta: BossGain,
    pub callisto: BossGain,
    pub calvarion: BossGain,
    pub cerberus: BossGain,
    pub chambers_of_xeric: BossGain,
    pub chambers_of_xeric_challenge_mode: BossGain,
    pub chaos_elemental: BossGain,
    pub chaos_fanatic: BossGain,
    pub commander_zilyana: BossGain,
    pub corporeal_beast: BossGain,
    pub crazy_archaeologist: BossGain,
    pub dagannoth_prime: BossGain,
    pub dagannoth_rex: BossGain,
    pub dagannoth_supreme: BossGain,
    pub deranged_archaeologist: BossGain,
    pub duke_sucellus: BossGain,
    pub general_graardor: BossGain,
    pub giant_mole: BossGain,
    pub grotesque_guardians: BossGain,
    pub hespori: BossGain,
    pub kalphite_queen: BossGain,
    pub king_black_dragon: BossGain,
    pub kraken: BossGain,
    pub kreearra: BossGain,
    pub kril_tsutsaroth: BossGain,
    pub mimic: BossGain,
    pub nex: BossGain,
    pub nightmare: BossGain,
    pub phosanis_nightmare: BossGain,
    pub obor: BossGain,
    pub phantom_muspah: BossGain,
    pub sarachnis: BossGain,
    pub scorpia: BossGain,
    pub scurrius: BossGain,
    pub skotizo: BossGain,
    pub spindel: BossGain,
    pub tempoross: BossGain,
    pub the_gauntlet: BossGain,
    pub the_corrupted_gauntlet: BossGain,
    pub the_leviathan: BossGain,
    pub the_whisperer: BossGain,
    pub theatre_of_blood: BossGain,
    pub theatre_of_blood_hard_mode: BossGain,
    pub thermonuclear_smoke_devil: BossGain,
    pub tombs_of_amascut: BossGain,
    pub tombs_of_amascut_expert: BossGain,
    pub tzkal_zuk: BossGain,
    pub tztok_jad: BossGain,
    pub vardorvis: BossGain,
    pub venenatis: BossGain,
    pub vetion: BossGain,
    pub vorkath: BossGain,
    pub wintertodt: BossGain,
    pub zalcano: BossGain,
    pub zulrah: BossGain,
}

/// The gains of a player's boss
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BossGain {
    pub metric: Boss,
    pub ehb: GainProgress,
    pub rank: GainProgress,
    pub kills: GainProgress,
}

/// The gains of a player's activities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityGains {
    pub league_points: Option<ActivityGain>,
    pub bounty_hunter_hunter: Option<ActivityGain>,
    pub bounty_hunter_rogue: Option<ActivityGain>,
    pub clue_scrolls_all: Option<ActivityGain>,
    pub clue_scrolls_beginner: Option<ActivityGain>,
    pub clue_scrolls_easy: Option<ActivityGain>,
    pub clue_scrolls_medium: Option<ActivityGain>,
    pub clue_scrolls_hard: Option<ActivityGain>,
    pub clue_scrolls_elite: Option<ActivityGain>,
    pub clue_scrolls_master: Option<ActivityGain>,
    pub last_man_standing: Option<ActivityGain>,
    pub pvp_arena: Option<ActivityGain>,
    pub soul_wars_zeal: Option<ActivityGain>,
    pub guardians_of_the_rift: Option<ActivityGain>,
}

/// The gains of a player's activity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityGain {
    pub metric: Activity,
    pub score: GainProgress,
    pub rank: GainProgress,
}

/// Each metrics gain progress.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GainProgress {
    pub gained: f64,
    pub start: f64,
    pub end: f64,
}

/// [Timeline Datapoint](https://docs.wiseoldman.net/players-api/player-type-definitions#object-timeline-datapoint)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineDatapoint {
    pub value: f64,
    pub rank: i64,
    pub date: DateTime<Utc>,
}
