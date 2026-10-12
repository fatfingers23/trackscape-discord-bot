use convert_case::{Case, Casing};
use serde::{Deserialize, Deserializer, Serialize};
use strum::{AsRefStr, Display, EnumString};

/// [Period](https://docs.wiseoldman.net/global-type-definitions#enum-period)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Period {
    #[serde(rename = "five_min")]
    FiveMin,
    Day,
    Week,
    Month,
    Year,
}

impl Period {
    pub fn as_str(&self) -> &'static str {
        match self {
            Period::FiveMin => "five_min",
            Period::Day => "day",
            Period::Week => "week",
            Period::Month => "month",
            Period::Year => "year",
        }
    }
}

/// [Metric](https://docs.wiseoldman.net/global-type-definitions#enum-metric)
#[derive(Debug, Clone, PartialEq, Serialize, EnumString)]
#[serde(rename_all = "camelCase")]
pub enum Metric {
    Skill(Skill),
    Boss(Boss),
    Activity(Activity),
    ComputedMetric(ComputedMetricEnum),
}

impl Metric {
    pub fn to_string(&self) -> String {
        let metric_string = match self {
            Metric::Skill(skill) => skill.to_string(),
            Metric::Boss(boss) => boss.to_string(),
            Metric::Activity(activity) => activity.to_string(),
            Metric::ComputedMetric(computed_metric) => computed_metric.to_string(),
        };
        return metric_string.to_case(Case::Snake);
    }
}

impl<'de> Deserialize<'de> for Metric {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let metric_str: String = Deserialize::deserialize(deserializer)?;
        let camel_cased = metric_str.to_case(Case::Title).replace(" ", "");

        if let Ok(skill) = Skill::try_from(camel_cased.as_str()) {
            return Ok(Metric::Skill(skill));
        }

        if let Ok(boss) = Boss::try_from(camel_cased.as_str()) {
            return Ok(Metric::Boss(boss));
        }

        if let Ok(activity) = Activity::try_from(camel_cased.as_str()) {
            return Ok(Metric::Activity(activity));
        }

        if let Ok(computed_metric) = ComputedMetricEnum::try_from(camel_cased.as_str()) {
            return Ok(Metric::ComputedMetric(computed_metric));
        }

        Err(serde::de::Error::custom(format!(
            "Unknown metric type: {:?}",
            metric_str
        )))
    }
}

/// [Skill](https://docs.wiseoldman.net/global-type-definitions#enum-skill)
#[derive(
    Debug, Default, Clone, PartialEq, Serialize, Deserialize, EnumString, AsRefStr, Display,
)]
#[serde(rename_all = "camelCase")]
pub enum Skill {
    #[default]
    None,
    Overall,
    Attack,
    Defence,
    Strength,
    Hitpoints,
    Ranged,
    Prayer,
    Magic,
    Cooking,
    Woodcutting,
    Fletching,
    Fishing,
    Firemaking,
    Crafting,
    Smithing,
    Mining,
    Herblore,
    Agility,
    Thieving,
    Slayer,
    Farming,
    Runecrafting,
    Hunter,
    Construction,
}

/// [Boss](https://docs.wiseoldman.net/global-type-definitions#enum-boss)
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, EnumString, Display)]
#[serde(rename_all = "snake_case")]
pub enum Boss {
    #[default]
    None,
    AbyssalSire,
    AlchemicalHydra,
    Artio,
    BarrowsChests,
    Bryophyta,
    Callisto,
    Calvarion,
    Cerberus,
    ChambersOfXeric,
    ChambersOfXericChallengeMode,
    ChaosElemental,
    ChaosFanatic,
    CommanderZilyana,
    CorporealBeast,
    CrazyArchaeologist,
    DagannothPrime,
    DagannothRex,
    DagannothSupreme,
    DerangedArchaeologist,
    DukeSucellus,
    GeneralGraardor,
    GiantMole,
    GrotesqueGuardians,
    Hespori,
    KalphiteQueen,
    KingBlackDragon,
    Kraken,
    Kreearra,
    KrilTsutsaroth,
    Mimic,
    Nex,
    Nightmare,
    PhosanisNightmare,
    Obor,
    PhantomMuspah,
    Sarachnis,
    Scorpia,
    Scurrius,
    Skotizo,
    Spindel,
    Tempoross,
    TheGauntlet,
    TheCorruptedGauntlet,
    TheLeviathan,
    TheWhisperer,
    TheatreOfBlood,
    TheatreOfBloodHardMode,
    ThermonuclearSmokeDevil,
    TombsOfAmascut,
    TombsOfAmascutExpert,
    TzkalZuk,
    TztokJad,
    Vardorvis,
    Venenatis,
    Vetion,
    Vorkath,
    Wintertodt,
    Zalcano,
    Zulrah,
}

/// [Computed Metric](https://docs.wiseoldman.net/global-type-definitions#enum-computed-metric)
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, EnumString, Display)]
#[serde(rename_all = "camelCase")]
pub enum ComputedMetricEnum {
    #[default]
    None,
    Ehp,
    Ehb,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, EnumString, Display)]
#[serde(rename_all = "snake_case")]
pub enum Activity {
    #[default]
    None,
    LeaguePoints,
    BountyHunterHunter,
    BountyHunterRogue,
    ClueScrollsAll,
    ClueScrollsBeginner,
    ClueScrollsEasy,
    ClueScrollsMedium,
    ClueScrollsHard,
    ClueScrollsElite,
    ClueScrollsMaster,
    LastManStanding,
    PvpArena,
    SoulWarsZeal,
    GuardiansOfTheRift,
}
