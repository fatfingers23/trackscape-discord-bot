use crate::models::global_types::{GroupId, PlayerId};
use crate::models::player::Player;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// [Group Role](https://docs.wiseoldman.net/groups-api/group-type-definitions#enum-group-role)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupRole {
    Achiever,
    Adamant,
    Adept,
    Administrator,
    Admiral,
    Adventurer,
    Air,
    Anchor,
    Apothecary,
    Archer,
    Armadylean,
    Artillery,
    Artisan,
    Asgarnian,
    Assassin,
    Assistant,
    Astral,
    Athlete,
    Attacker,
    Bandit,
    Bandosian,
    Barbarian,
    Battlemage,
    Beast,
    Berserker,
    Blisterwood,
    Blood,
    Blue,
    Bob,
    Body,
    Brassican,
    Brawler,
    Brigadier,
    Brigand,
    Bronze,
    Bruiser,
    Bulwark,
    Burglar,
    Burnt,
    Cadet,
    Captain,
    Carry,
    Champion,
    Chaos,
    Cleric,
    Collector,
    Colonel,
    Commander,
    Competitor,
    Completionist,
    Constructor,
    Cook,
    Coordinator,
    Corporal,
    Cosmic,
    Councillor,
    Crafter,
    Crew,
    Crusader,
    Cutpurse,
    Death,
    Defender,
    Defiler,
    DeputyOwner,
    Destroyer,
    Diamond,
    Diseased,
    Doctor,
    Dogsbody,
    Dragon,
    Dragonstone,
    Druid,
    Duellist,
    Earth,
    Elite,
    Emerald,
    Enforcer,
    Epic,
    Executive,
    Expert,
    Explorer,
    Farmer,
    Feeder,
    Fighter,
    Fire,
    Firemaker,
    Firestarter,
    Fisher,
    Fletcher,
    Forager,
    Fremennik,
    Gamer,
    Gatherer,
    General,
    GnomeChild,
    GnomeElder,
    Goblin,
    Gold,
    Goon,
    Green,
    Grey,
    Guardian,
    Guthixian,
    Harpoon,
    Healer,
    Hellcat,
    Helper,
    Herbologist,
    Hero,
    Holy,
    Hoarder,
    Hunter,
    Ignitor,
    Illusionist,
    Imp,
    Infantry,
    Inquisitor,
    Iron,
    Jade,
    Justiciar,
    Kandarin,
    Karamjan,
    Kharidian,
    Kitten,
    Knight,
    Labourer,
    Law,
    Leader,
    Learner,
    Legacy,
    Legend,
    Legionnaire,
    Lieutenant,
    Looter,
    Lumberjack,
    Magic,
    Magician,
    Major,
    Maple,
    Marshal,
    Master,
    Maxed,
    Mediator,
    Medic,
    Mentor,
    Member,
    Merchant,
    Mind,
    Miner,
    Minion,
    Misthalinian,
    Mithril,
    Moderator,
    Monarch,
    Morytanian,
    Mystic,
    Myth,
    Natural,
    Nature,
    Necromancer,
    Ninja,
    Noble,
    Novice,
    Nurse,
    Oak,
    Officer,
    Onyx,
    Opal,
    Oracle,
    Orange,
    Owner,
    Page,
    Paladin,
    Pawn,
    Pilgrim,
    Pine,
    Pink,
    Prefect,
    Priest,
    Private,
    Prodigy,
    Proselyte,
    Prospector,
    Protector,
    Pure,
    Purple,
    Pyromancer,
    Quester,
    Racer,
    Raider,
    Ranger,
    RecordChaser,
    Recruit,
    Recruiter,
    RedTopaz,
    Red,
    Rogue,
    Ruby,
    Rune,
    Runecrafter,
    Sage,
    Sapphire,
    Saradominist,
    Saviour,
    Scavenger,
    Scholar,
    Scourge,
    Scout,
    Scribe,
    Seer,
    Senator,
    Sentry,
    Serenist,
    Sergeant,
    Shaman,
    Sheriff,
    ShortGreenGuy,
    Skiller,
    Skulled,
    Slayer,
    Smiter,
    Smith,
    Smuggler,
    Sniper,
    Soul,
    Specialist,
    SpeedRunner,
    Spellcaster,
    Squire,
    Staff,
    Steel,
    Strider,
    Striker,
    Summoner,
    Superior,
    Supervisor,
    Teacher,
    Templar,
    Therapist,
    Thief,
    Tirannian,
    Trialist,
    Trickster,
    Tzkal,
    Tztok,
    Unholy,
    Vagrant,
    Vanguard,
    Walker,
    Wanderer,
    Warden,
    Warlock,
    Warrior,
    Water,
    Wild,
    Willow,
    Wily,
    Wintumber,
    Witch,
    Wizard,
    Worker,
    Wrath,
    Xerician,
    Yellow,
    Yew,
    Zamorakian,
    Zarosian,
    Zealot,
    Zenyte,
}

/// [Activity Type](https://docs.wiseoldman.net/groups-api/group-type-definitions#enum-activity-type)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActivityType {
    Joined,
    Left,
    #[serde(rename = "changed_role")]
    ChangedRole,
}

/// [Group Social Links](https://docs.wiseoldman.net/groups-api/group-type-definitions#object-groupsociallinks)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupSocialLinks {
    pub website: Option<String>,
    pub discord: Option<String>,
    pub twitter: Option<String>,
    pub youtube: Option<String>,
    pub twitch: Option<String>,
}

/// [Group](https://docs.wiseoldman.net/groups-api/group-type-definitions#object-group)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub clan_chat: Option<String>,
    pub description: Option<String>,
    pub homeworld: Option<i64>,
    pub verified: bool,
    pub patron: bool,
    pub profile_image: Option<String>,
    pub banner_image: Option<String>,
    pub score: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub member_count: i64,
}

/// [Group Details](https://docs.wiseoldman.net/groups-api/group-type-definitions#object-group-details)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupDetail {
    pub id: GroupId,
    pub name: String,
    pub clan_chat: Option<String>,
    pub description: Option<String>,
    pub homeworld: Option<i64>,
    pub verified: bool,
    pub patron: bool,
    pub profile_image: Option<String>,
    pub banner_image: Option<String>,
    pub score: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub member_count: i64,
    pub memberships: Vec<GroupMemberShip>,
    pub social_links: Option<GroupSocialLinks>,
}

/// [Membership](https://docs.wiseoldman.net/groups-api/group-type-definitions#object-membership)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Membership {
    pub player_id: PlayerId,
    pub group_id: GroupId,
    pub role: Option<GroupRole>,
    /// The date at which the player was added as a member to the group.
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// [Group Membership](https://docs.wiseoldman.net/groups-api/group-type-definitions#object-group-membership)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMemberShip {
    pub player_id: PlayerId,
    pub group_id: GroupId,
    pub role: Option<GroupRole>,
    /// The date at which the player was added as a member to the group.
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub player: Player,
}

/// [Player Membership](https://docs.wiseoldman.net/groups-api/group-type-definitions#object-player-membership)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerMembership {
    pub player_id: PlayerId,
    pub group_id: GroupId,
    pub role: Option<GroupRole>,
    /// The date at which the player was added as a member to the group.
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub group: Group,
}

// [Group Member Fragment](https://docs.wiseoldman.net/groups-api/group-type-definitions#object-group-member-fragment)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMemberFragment {
    pub username: PlayerId,
    pub role: Option<GroupRole>,
}

/// [Body to create a new group](https://docs.wiseoldman.net/groups-api/group-endpoints#create-group)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clan_chat: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homeworld: Option<i64>,
    pub members: Vec<GroupMemberFragment>,
}

impl CreateGroupRequest {
    pub fn new(name: String) -> Self {
        Self {
            name,
            clan_chat: None,
            description: None,
            homeworld: None,
            members: vec![],
        }
    }

    pub fn new_with_clan_chat(group_name: String, clan_chat: String) -> Self {
        Self {
            name: group_name,
            clan_chat: Some(clan_chat),
            description: None,
            homeworld: None,
            members: vec![],
        }
    }
}

/// Group Create Response
/// [Create Group](https://docs.wiseoldman.net/groups-api/group-endpoints#create-group)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupCreateResponse {
    pub group: GroupDetail,
    pub verification_code: String,
}
