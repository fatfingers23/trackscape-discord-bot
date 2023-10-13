pub mod osrs_broadcast_extractor {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct ClanMessage {
        pub sender: String,
        pub message: String,
        pub clan_name: String,
        pub rank: String,
        // iconids for account type
        // PLAYER_MODERATOR(0),
        // JAGEX_MODERATOR(1),
        // IRONMAN(2),
        // ULTIMATE_IRONMAN(3),
        // DMM_SKULL_5_KEYS(4),
        // DMM_SKULL_4_KEYS(5),
        // DMM_SKULL_3_KEYS(6),
        // DMM_SKULL_2_KEYS(7),
        // DMM_SKULL_1_KEYS(8),
        // SKULL(9),
        // HARDCORE_IRONMAN(10),
        // NO_ENTRY(11),
        // CHAIN_LINK(12),
        // BOUNTY_HUNTER_EMBLEM(20),
        // LEAGUE(22);
        pub icon_id: Option<i64>,
        pub is_league_world: Option<bool>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct DropItemBroadcast {
        pub player_it_happened_to: String,
        pub item_name: String,
        pub item_quantity: i64,
        pub item_value: Option<i64>,
        pub item_icon: Option<String>,
    }

    pub struct PetDropBroadcast {
        pub player_it_happened_to: String,
        pub pet_name: String,
        pub pet_icon: Option<String>,
        //Could be kc, or task count
        pub actions_optioned_at: Option<i64>,
        //Could be kc, or task,  rift searches, permits, xp, etc
        pub action_for_pet: Option<String>,
    }

    pub struct QuestCompletedBroadcast {
        pub player_it_happened_to: String,
        pub quest_name: String,
        pub quest_reward_scroll_icon: Option<String>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum QuestDifficulty {
        Novice,
        Intermediate,
        Experienced,
        Master,
        Grandmaster,
    }

    impl QuestDifficulty {
        pub fn from_string(quest_difficulty: String) -> QuestDifficulty {
            match quest_difficulty.as_str() {
                "Novice" => QuestDifficulty::Novice,
                "Intermediate" => QuestDifficulty::Intermediate,
                "Experienced" => QuestDifficulty::Experienced,
                "Master" => QuestDifficulty::Master,
                "Grandmaster" => QuestDifficulty::Grandmaster,
                _ => QuestDifficulty::Novice,
            }
        }

        pub fn to_string(&self) -> String {
            match self {
                QuestDifficulty::Novice => "Novice".to_string(),
                QuestDifficulty::Intermediate => "Intermediate".to_string(),
                QuestDifficulty::Experienced => "Experienced".to_string(),
                QuestDifficulty::Master => "Master".to_string(),
                QuestDifficulty::Grandmaster => "Grandmaster".to_string(),
            }
        }

        pub fn iter() -> Vec<QuestDifficulty> {
            vec![
                QuestDifficulty::Novice,
                QuestDifficulty::Intermediate,
                QuestDifficulty::Experienced,
                QuestDifficulty::Master,
                QuestDifficulty::Grandmaster,
            ]
        }

        pub fn ranking(&self) -> usize {
            match self {
                QuestDifficulty::Novice => 1,
                QuestDifficulty::Intermediate => 2,
                QuestDifficulty::Experienced => 3,
                QuestDifficulty::Master => 4,
                QuestDifficulty::Grandmaster => 5,
            }
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum DiaryTier {
        Easy,
        Medium,
        Hard,
        Elite,
    }

    impl DiaryTier {
        pub fn from_string(diary_tier: String) -> DiaryTier {
            match diary_tier.as_str() {
                "Easy" => DiaryTier::Easy,
                "Medium" => DiaryTier::Medium,
                "Hard" => DiaryTier::Hard,
                "Elite" => DiaryTier::Elite,
                _ => DiaryTier::Easy,
            }
        }

        pub fn to_string(&self) -> String {
            match self {
                DiaryTier::Easy => "Easy".to_string(),
                DiaryTier::Medium => "Medium".to_string(),
                DiaryTier::Hard => "Hard".to_string(),
                DiaryTier::Elite => "Elite".to_string(),
            }
        }

        pub fn ranking(&self) -> usize {
            match self {
                DiaryTier::Easy => 1,
                DiaryTier::Medium => 2,
                DiaryTier::Hard => 3,
                DiaryTier::Elite => 4,
            }
        }
    }

    pub struct DiaryCompletedBroadcast {
        pub player_it_happened_to: String,
        pub diary_name: String,
        pub diary_tier: DiaryTier,
    }

    // KANlEL OUTIS has been defeated by Veljenpojat in The Wilderness and lost (953,005 coins) worth of loot.
    // KANlEL OUTIS has defeated Emperor KB and received (972,728 coins) worth of loot!
    // Main Dangler has been defeated by Koishi Fumo in The Wilderness.
    // tikkok ALT has been defeated by WhatsA Dad in The Wilderness and lost (462,128 coins) worth of loot. Clearly tikkok ALT struggles with clicking.
    pub struct PkBroadcast {
        pub winner: String,
        pub loser: String,
        //Name of the clan mate it happen to, this will also be the winner or loser
        pub clan_mate: String,
        //Amount of gp exchanged or gained
        pub gp_exchanged: Option<i64>,
        //If the clan mate was the winner or loser. True for winner, false for a lost
        pub clan_mate_won: bool,
    }

    // Victor Locke has been invited into the clan by IRuneNakey.
    // KingConley has been invited into the clan by kanga roe.
    // RUKAl has been invited into the clan by l cant see.
    pub struct InviteBroadcast {
        //name of clan mate that sent the clan invite
        pub clan_mate: String,
        //name of new account invited to clan
        pub new_clan_mate: String,
    }

    // Th3TRiPPyOn3 has reached Defence level 70.
    // MechaPanzer has reached combat level 104.
    // I Vision I has reached a total level of 2225.
    // Zillamanjaro has reached the highest possible combat level of 126!
    // Sad Bug has reached the highest possible total level of 2277!
    pub struct LevelMilestoneBroadcast {
        //name of clan mate that levelled up
        pub clan_mate: String,
        //name of skill that was levelled
        pub skill_levelled: String,
        //new level of the skill that was levelled up
        pub new_skill_level: String,
        //icon for skill levelled
        pub skill_icon: Option<String>,
    }

    // Noble Five has reached 78,000,000 XP in Fishing.
    // Matrese has reached 15,000,000 XP in Fishing.
    // Marsel has reached 200,000,000 XP in Cooking.
    pub struct XPMilestoneBroadcast {
        //name of clan mate that levelled up
        pub clan_mate: String,
        //name of skill that was levelled
        pub skill: String,
        //new level of the skill that was levelled up
        pub new_skill_xp: String,
        //icon for skill
        pub skill_icon: Option<String>,
    }

    // KANlEL OUTIS has opened a loot key worth 1,148,040 coins!
    // Med-iocore has opened a loot key worth 489,181 coins!
    pub struct LootKey {
        pub player: String,
        pub value: i64,
    }

    #[derive(PartialEq, Deserialize, Serialize, Debug, Clone)]
    pub enum BroadcastType {
        ItemDrop,
        PetDrop,
        Quest,
        Diary,
        RaidDrop,
        Pk,
        Invite,
        LootKey,
        XPMilestone,
        LevelMilestone,
        Unknown,
    }

    impl BroadcastType {
        pub fn to_string(&self) -> String {
            match self {
                BroadcastType::ItemDrop => "Item Drop".to_string(),
                BroadcastType::PetDrop => "Pet Drop".to_string(),
                BroadcastType::Quest => "Quest".to_string(),
                BroadcastType::Diary => "Diary".to_string(),
                BroadcastType::RaidDrop => "Raid Drop".to_string(),
                BroadcastType::Pk => "Pk".to_string(),
                BroadcastType::Invite => "Invite".to_string(),
                BroadcastType::LootKey => "Loot Key".to_string(),
                BroadcastType::XPMilestone => "XP Milestone".to_string(),
                BroadcastType::LevelMilestone => "Level Milestone".to_string(),
                BroadcastType::Unknown => "Unknown".to_string(),
            }
        }

        pub fn from_string(broadcast_type: String) -> BroadcastType {
            match broadcast_type.as_str() {
                "Item Drop" => BroadcastType::ItemDrop,
                "Pet Drop" => BroadcastType::PetDrop,
                "Quest" => BroadcastType::Quest,
                "Diary" => BroadcastType::Diary,
                "Raid Drop" => BroadcastType::RaidDrop,
                "Pk" => BroadcastType::Pk,
                "Invite" => BroadcastType::Invite,
                "Loot Key" => BroadcastType::LootKey,
                "XP Milestone" => BroadcastType::XPMilestone,
                "Level Milestone" => BroadcastType::LevelMilestone,
                _ => BroadcastType::Unknown,
            }
        }

        pub fn to_slug(&self) -> String {
            match self {
                _ => self.to_string().replace(" ", "_"),
            }
        }
    }

    pub fn raid_broadcast_extractor(message: String) -> Option<DropItemBroadcast> {
        let re = regex::Regex::new(
            r#"^(?P<player_name>.*?) received special loot from a raid: (?P<item>.*?)([.]|$)"#,
        )
        .unwrap();

        return if let Some(caps) = re.captures(message.as_str()) {
            let player_name = caps.name("player_name").unwrap().as_str();
            let item = caps.name("item").unwrap().as_str();

            Some(DropItemBroadcast {
                player_it_happened_to: player_name.to_string(),
                item_name: item.to_string(),
                item_quantity: 1,
                item_value: None,
                item_icon: Some(get_wiki_image_url(item.to_string())),
            })
        } else {
            None
        };
    }

    pub fn drop_broadcast_extractor(message: String) -> Option<DropItemBroadcast> {
        let re = regex::Regex::new(r#"^(?P<player_name>.*?) received a drop: (?:((?P<quantity>[,\d]+) x )?)(?P<item>.*?)(?: \((?P<value>[,\d]+) coins\))?(?: from .*?)?[.]?$"#).unwrap();

        return if let Some(caps) = re.captures(message.as_str()) {
            let player_name = caps.name("player_name").unwrap().as_str();
            let item_name = caps.name("item").unwrap().as_str();
            // Extract and parse quantity
            let quantity_str = caps.name("quantity").map_or("", |m| m.as_str());
            let quantity: i64 = if !quantity_str.is_empty() {
                quantity_str.replace(",", "").parse().unwrap_or(0)
            } else {
                1
            };
            let value_with_commas = caps.name("value").map_or("", |m| m.as_str());
            let value: i64 = value_with_commas.replace(",", "").parse().unwrap_or(0);

            Some(DropItemBroadcast {
                player_it_happened_to: player_name.to_string(),
                item_name: item_name.to_string(),
                item_quantity: quantity,
                item_value: Some(value),
                item_icon: Some(get_wiki_image_url(item_name.to_string())),
            })
        } else {
            None
        };
    }

    pub fn pet_broadcast_extractor(message: String) -> Option<PetDropBroadcast> {
        let re = regex::Regex::new(r#"^(?P<player_name>.*?) (?:has a funny feeling.*?|feels something weird sneaking into (?P<pronoun>her|his) backpack): (?P<pet_name>.*?) at (?P<count>[,\d]+) (?P<count_type>.*?)[.]$"#).unwrap();

        if let Some(caps) = re.captures(message.as_str()) {
            let player_name = caps.name("player_name").unwrap().as_str();
            let pet_name = caps.name("pet_name").unwrap().as_str();
            let count = caps.name("count").unwrap().as_str().replace(",", "");
            let count_type = caps.name("count_type").unwrap().as_str();
            Some(PetDropBroadcast {
                player_it_happened_to: player_name.to_string(),
                pet_name: pet_name.to_string(),
                pet_icon: get_wiki_image_url(pet_name.to_string()).parse().ok(),
                actions_optioned_at: count.parse().ok(),
                action_for_pet: count_type.parse().ok(),
            })
        } else {
            None
        }
    }

    pub fn quest_completed_broadcast_extractor(message: String) -> Option<QuestCompletedBroadcast> {
        let re = regex::Regex::new(
            r#"^(?P<player_name>.*?) has completed a quest: (?P<quest_name>.+)$"#,
        )
        .unwrap();

        return if let Some(caps) = re.captures(message.as_str()) {
            let player_name = caps.name("player_name").unwrap().as_str();
            let quest_name = caps.name("quest_name").unwrap().as_str();

            Some(QuestCompletedBroadcast {
                player_it_happened_to: player_name.to_string(),
                quest_name: quest_name.to_string(),
                quest_reward_scroll_icon: Some(get_quest_reward_scroll(quest_name.to_string())),
            })
        } else {
            None
        };
    }

    pub fn diary_completed_broadcast_extractor(message: String) -> Option<DiaryCompletedBroadcast> {
        let re = regex::Regex::new(r#"^(?P<player_name>.*?) has completed the (?P<diary_tier>Easy|Medium|Hard|Elite) (?P<diary_name>.*?).$"#).unwrap();

        return if let Some(caps) = re.captures(message.as_str()) {
            let player_name = caps.name("player_name").unwrap().as_str();
            let diary_name = caps.name("diary_name").unwrap().as_str();
            let diary_tier = caps.name("diary_tier").unwrap().as_str();

            Some(DiaryCompletedBroadcast {
                player_it_happened_to: player_name.to_string(),
                diary_name: diary_name.to_string(),
                diary_tier: DiaryTier::from_string(diary_tier.to_string()),
            })
        } else {
            None
        };
    }

    pub fn pk_broadcast_extractor(message: String) -> Option<PkBroadcast> {
        let mut re = regex::Regex::new(r#"^(?P<winner_name>.*?) has defeated (?P<loser_name>.*?) and received \((?P<gp_value>[0-9,]+) coins\) worth of loot!"#).unwrap();
        if message.contains("defeated by") {
            re = regex::Regex::new(r#"^(?P<loser_name>.*?) has been defeated by (?P<winner_name>.*?)(?: in (?P<location>The Wilderness))?(?: and lost \((?P<gp_value>[0-9,]+) coins\) worth of loot)?[!.]"#).unwrap();
        };
        return if let Some(caps) = re.captures(message.as_str()) {
            let winner_name = caps.name("winner_name").unwrap().as_str();
            let loser_name = caps.name("loser_name").unwrap().as_str();
            let mut clan_mate_name = caps.name("winner_name").unwrap().as_str();
            let mut clan_mate_winner = true;
            if message.contains("defeated by") {
                clan_mate_winner = false;
                clan_mate_name = caps.name("loser_name").unwrap().as_str();
            };
            let gp_value_str = caps.name("gp_value").map_or("", |m| m.as_str());
            let int_value: i64 = gp_value_str.replace(",", "").parse().unwrap_or(0);
            let gp_value = if int_value == 0 {
                None
            } else {
                Some(int_value)
            };
            Some(PkBroadcast {
                winner: winner_name.to_string(),
                loser: loser_name.to_string(),
                clan_mate: clan_mate_name.to_string(),
                gp_exchanged: gp_value,
                clan_mate_won: clan_mate_winner,
            })
        } else {
            None
        };
    }

    pub fn invite_broadcast_extractor(message: String) -> Option<InviteBroadcast> {
        let re = regex::Regex::new(
            r#"^(?P<clan_joiner>.*?) has been invited into the clan by (?P<clan_inviter>.*?).$"#,
        )
        .unwrap();

        return if let Some(caps) = re.captures(message.as_str()) {
            let clan_mate = caps.name("clan_inviter").unwrap().as_str();
            let new_clan_mate = caps.name("clan_joiner").unwrap().as_str();
            Some(InviteBroadcast {
                clan_mate: clan_mate.to_string(),
                new_clan_mate: new_clan_mate.to_string(),
            })
        } else {
            None
        };
    }

    pub fn levelmilestone_broadcast_extractor(message: String) -> Option<LevelMilestoneBroadcast> {
        let re = regex::Regex::new(r#"^(?P<clan_mate>.*?) has reached (?:a )?(?:the highest possible )?(?P<skill>.*?) level(?: of)? (?P<level>.*?)[!.]"#).unwrap();

        return if let Some(caps) = re.captures(message.as_str()) {
            let clan_mate = caps.name("clan_mate").unwrap().as_str();
            let skill_levelled = caps.name("skill").unwrap().as_str();
            let new_skill_level = caps.name("level").unwrap().as_str();
            Some(LevelMilestoneBroadcast {
                clan_mate: clan_mate.to_string(),
                skill_levelled: skill_levelled.to_string(),
                new_skill_level: new_skill_level.to_string(),
                skill_icon: Some(get_skill_icon(skill_levelled.to_string())),
            })
        } else {
            None
        };
    }

    pub fn xpmilestone_broadcast_extractor(message: String) -> Option<XPMilestoneBroadcast> {
        let re = regex::Regex::new(
            r#"^(?P<clan_member>.*?) has reached (?P<xp>.*?) XP in (?P<skill>.*?)[!.]"#,
        )
        .unwrap();

        return if let Some(caps) = re.captures(message.as_str()) {
            let clan_mate = caps.name("clan_member").unwrap().as_str();
            let skill = caps.name("skill").unwrap().as_str();
            let new_skill_xp = caps.name("xp").unwrap().as_str();
            Some(XPMilestoneBroadcast {
                clan_mate: clan_mate.to_string(),
                skill: skill.to_string(),
                new_skill_xp: new_skill_xp.to_string(),
                skill_icon: Some(get_skill_icon(skill.to_string())),
            })
        } else {
            None
        };
    }

    pub fn get_broadcast_type(message_content: String) -> BroadcastType {
        if message_content.contains("received a drop:") {
            return BroadcastType::ItemDrop;
        }
        if message_content.contains("received special loot from a raid:") {
            return BroadcastType::RaidDrop;
        }
        if message_content.contains("has a funny feeling") && message_content.contains("followed:")
            || message_content.contains("feels something weird sneaking into")
                && message_content.contains("backpack:")
        {
            return BroadcastType::PetDrop;
        }
        if message_content.contains("has completed a quest:") {
            return BroadcastType::Quest;
        }
        if message_content.contains("has completed the") && message_content.contains("diary") {
            return BroadcastType::Diary;
        }
        if message_content.contains("has defeated") || message_content.contains("defeated by") {
            return BroadcastType::Pk;
        }
        if message_content.contains("has been invited") {
            return BroadcastType::Invite;
        }
        if message_content.contains("has reached") && message_content.contains("level") {
            return BroadcastType::LevelMilestone;
        }
        if message_content.contains("has reached") && message_content.contains("XP in") {
            return BroadcastType::XPMilestone;
        }
        return BroadcastType::Unknown;
    }

    fn format_wiki_image_name(item_name: String) -> String {
        let replace_spaces = item_name.replace(" ", "_");
        let encoded_item_name = urlencoding::encode(replace_spaces.as_str());
        encoded_item_name.parse().unwrap()
    }

    fn get_wiki_image_url(item_name: String) -> String {
        let image_name = format_wiki_image_name(item_name);
        format!(
            "https://oldschool.runescape.wiki/images/{}_detail.png",
            image_name
        )
    }

    pub fn get_wiki_clan_rank_image_url(rank: String) -> String {
        let image_picture_name: String = match rank.as_str() {
            "Deputy Owner" => "Deputy_owner".to_string(),
            _ => format_wiki_image_name(rank.clone()),
        };

        format!(
            "https://oldschool.runescape.wiki/images/Clan_icon_-_{}.png",
            image_picture_name
        )
    }

    pub fn get_quest_reward_scroll(quest: String) -> String {
        let image_name = format_wiki_image_name(quest);
        format!(
            "https://oldschool.runescape.wiki/images/{}_reward_scroll.png",
            image_name
        )
    }

    pub fn get_skill_icon(skill: String) -> String {
        let image_name = format_wiki_image_name(skill);
        format!(
            "https://oldschool.runescape.wiki/images/{}_icon_(detail).png",
            image_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::{
        get_wiki_clan_rank_image_url, DiaryCompletedBroadcast, DiaryTier, InviteBroadcast,
        LevelMilestoneBroadcast, PetDropBroadcast, PkBroadcast, QuestCompletedBroadcast,
        XPMilestoneBroadcast,
    };
    use tracing::info;

    #[test]
    fn test_get_drop_type_broadcast() {
        let possible_drop_broadcasts = get_drop_messages();
        for possible_drop_broadcast in possible_drop_broadcasts {
            let broadcast_type =
                osrs_broadcast_extractor::get_broadcast_type(possible_drop_broadcast.message);
            assert!(matches!(
                broadcast_type,
                osrs_broadcast_extractor::BroadcastType::ItemDrop
            ));
        }
    }

    #[test]
    fn test_get_raid_drop_type_broadcast() {
        let possible_raid_broadcasts = get_raid_messages();
        for possible_raid_broadcast in possible_raid_broadcasts {
            let broadcast_type =
                osrs_broadcast_extractor::get_broadcast_type(possible_raid_broadcast.message);
            assert!(matches!(
                broadcast_type,
                osrs_broadcast_extractor::BroadcastType::RaidDrop
            ));
        }
    }

    #[test]
    fn test_get_pet_type_broadcast() {
        let possible_pet_broadcasts = get_pet_messages();
        for possible_pet_broadcast in possible_pet_broadcasts {
            let broadcast_type =
                osrs_broadcast_extractor::get_broadcast_type(possible_pet_broadcast.message);
            assert!(matches!(
                broadcast_type,
                osrs_broadcast_extractor::BroadcastType::PetDrop
            ));
        }
    }

    #[test]
    fn test_get_quest_completed_type_broadcast() {
        let possible_quest_completed_broadcasts = get_quest_completed_messages();
        for possible_quest_completed_broadcast in possible_quest_completed_broadcasts {
            let broadcast_type = osrs_broadcast_extractor::get_broadcast_type(
                possible_quest_completed_broadcast.message,
            );
            assert!(matches!(
                broadcast_type,
                osrs_broadcast_extractor::BroadcastType::Quest
            ));
        }
    }

    #[test]
    fn test_get_achievement_diary_completed_type_broadcast() {
        let possible_achievement_diary_completed_broadcasts = get_diary_completed_messages();
        for possible_achievement_diary_completed_broadcast in
            possible_achievement_diary_completed_broadcasts
        {
            let broadcast_type = osrs_broadcast_extractor::get_broadcast_type(
                possible_achievement_diary_completed_broadcast.message,
            );
            assert!(matches!(
                broadcast_type,
                osrs_broadcast_extractor::BroadcastType::Diary
            ));
        }
    }

    #[test]
    fn test_get_pk_type_broadcast() {
        let possible_pk_broadcasts = get_pk_messages();
        for possible_pk_broadcast in possible_pk_broadcasts {
            let broadcast_type =
                osrs_broadcast_extractor::get_broadcast_type(possible_pk_broadcast.message);
            assert!(matches!(
                broadcast_type,
                osrs_broadcast_extractor::BroadcastType::Pk
            ));
        }
    }

    #[test]
    fn test_get_invite_type_broadcast() {
        let possible_invite_broadcasts = get_invite_messages();
        for possible_invite_broadcast in possible_invite_broadcasts {
            let broadcast_type =
                osrs_broadcast_extractor::get_broadcast_type(possible_invite_broadcast.message);
            assert!(matches!(
                broadcast_type,
                osrs_broadcast_extractor::BroadcastType::Invite
            ));
        }
    }

    #[test]
    fn test_get_levelmilestone_type_broadcast() {
        let possible_levelmilestone_broadcasts = get_levelmilestone_messages();
        for possible_levelmilestone_broadcast in possible_levelmilestone_broadcasts {
            let broadcast_type = osrs_broadcast_extractor::get_broadcast_type(
                possible_levelmilestone_broadcast.message,
            );
            assert!(matches!(
                broadcast_type,
                osrs_broadcast_extractor::BroadcastType::LevelMilestone
            ));
        }
    }

    #[test]
    fn test_get_xpmilestone_type_broadcast() {
        let possible_xpmilestone_broadcasts = get_xpmilestone_messages();
        for possible_xpmilestone_broadcast in possible_xpmilestone_broadcasts {
            let broadcast_type = osrs_broadcast_extractor::get_broadcast_type(
                possible_xpmilestone_broadcast.message,
            );
            assert!(matches!(
                broadcast_type,
                osrs_broadcast_extractor::BroadcastType::XPMilestone
            ));
        }
    }

    #[test]
    fn test_raid_extractor() {
        let possible_raid_broadcasts = get_raid_messages();
        for possible_raid_broadcast in possible_raid_broadcasts {
            let message = possible_raid_broadcast.message.clone();
            let possible_drop_item =
                osrs_broadcast_extractor::raid_broadcast_extractor(possible_raid_broadcast.message);
            match possible_drop_item {
                None => {
                    info!("Failed to extract drop item from message: {}", message);
                    assert!(false);
                }
                Some(drop_item) => {
                    assert_eq!(drop_item.item_name, possible_raid_broadcast.item_name);
                    assert_eq!(
                        drop_item.item_quantity,
                        possible_raid_broadcast.item_quantity
                    );
                    assert_eq!(drop_item.item_value, possible_raid_broadcast.item_value);
                    assert_eq!(
                        drop_item.player_it_happened_to,
                        possible_raid_broadcast.player_it_happened_to
                    );
                    assert_eq!(drop_item.item_icon, possible_raid_broadcast.item_icon)
                }
            }
        }
    }

    #[test]
    fn test_drop_extractor() {
        let possible_drop_broadcasts = get_drop_messages();
        for possible_drop_broadcast in possible_drop_broadcasts {
            let message = possible_drop_broadcast.message.clone();
            let possible_drop_item =
                osrs_broadcast_extractor::drop_broadcast_extractor(possible_drop_broadcast.message);
            match possible_drop_item {
                None => {
                    info!("Failed to extract drop item from message: {}", message);
                    assert!(false);
                }
                Some(drop_item) => {
                    assert_eq!(drop_item.item_name, possible_drop_broadcast.item_name);
                    assert_eq!(
                        drop_item.item_quantity,
                        possible_drop_broadcast.item_quantity
                    );
                    assert_eq!(drop_item.item_value, possible_drop_broadcast.item_value);
                    assert_eq!(
                        drop_item.player_it_happened_to,
                        possible_drop_broadcast.player_it_happened_to
                    );
                    assert_eq!(drop_item.item_icon, possible_drop_broadcast.item_icon);
                }
            }
        }
    }

    #[test]
    fn test_pet_extractor() {
        let possible_pet_broadcasts = get_pet_messages();
        for possible_pet_broadcast in possible_pet_broadcasts {
            let broadcast_type = osrs_broadcast_extractor::pet_broadcast_extractor(
                possible_pet_broadcast.message.clone(),
            );
            match broadcast_type {
                None => {
                    info!(
                        "Failed to extract pet drop from message: {}",
                        possible_pet_broadcast.message.clone()
                    );
                    assert!(false);
                }
                Some(pet_broadcast) => {
                    assert_eq!(
                        pet_broadcast.player_it_happened_to,
                        possible_pet_broadcast.pet_drop.player_it_happened_to
                    );
                    assert_eq!(
                        pet_broadcast.pet_name,
                        possible_pet_broadcast.pet_drop.pet_name
                    );
                    assert_eq!(
                        pet_broadcast.pet_icon,
                        possible_pet_broadcast.pet_drop.pet_icon
                    );
                    assert_eq!(
                        pet_broadcast.actions_optioned_at,
                        possible_pet_broadcast.pet_drop.actions_optioned_at
                    );
                    assert_eq!(
                        pet_broadcast.action_for_pet,
                        possible_pet_broadcast.pet_drop.action_for_pet
                    );
                }
            }
        }
    }

    #[test]
    fn test_quest_extractor() {
        let possible_quest_completed_broadcasts = get_quest_completed_messages();
        for possible_quest_completed_broadcast in possible_quest_completed_broadcasts {
            let possible_quest_extract =
                osrs_broadcast_extractor::quest_completed_broadcast_extractor(
                    possible_quest_completed_broadcast.message.clone(),
                );
            match possible_quest_extract {
                None => {
                    info!(
                        "Failed to extract quest completed from message: {}",
                        possible_quest_completed_broadcast.message.clone()
                    );
                    assert!(false);
                }
                Some(quest_completed_broadcast) => {
                    assert_eq!(
                        quest_completed_broadcast.player_it_happened_to,
                        possible_quest_completed_broadcast
                            .quest_completed
                            .player_it_happened_to
                    );
                    assert_eq!(
                        quest_completed_broadcast.quest_name,
                        possible_quest_completed_broadcast
                            .quest_completed
                            .quest_name
                    );

                    assert_eq!(
                        quest_completed_broadcast.quest_reward_scroll_icon,
                        possible_quest_completed_broadcast
                            .quest_completed
                            .quest_reward_scroll_icon
                    );
                }
            }
        }
    }

    #[test]
    fn test_diary_extractor() {
        let possible_diary_completed_broadcasts = get_diary_completed_messages();
        for possible_diary_completed_broadcast in possible_diary_completed_broadcasts {
            let possible_diary_extract =
                osrs_broadcast_extractor::diary_completed_broadcast_extractor(
                    possible_diary_completed_broadcast.message.clone(),
                );
            match possible_diary_extract {
                None => {
                    info!(
                        "Failed to extract diary completed from message: {}",
                        possible_diary_completed_broadcast.message.clone()
                    );
                    assert!(false);
                }
                Some(diary_completed_broadcast) => {
                    assert_eq!(
                        diary_completed_broadcast.player_it_happened_to,
                        possible_diary_completed_broadcast
                            .diary_completed
                            .player_it_happened_to
                    );
                    assert_eq!(
                        diary_completed_broadcast.diary_name,
                        possible_diary_completed_broadcast
                            .diary_completed
                            .diary_name
                    );
                    assert_eq!(
                        diary_completed_broadcast.diary_tier.to_string(),
                        possible_diary_completed_broadcast
                            .diary_completed
                            .diary_tier
                            .to_string()
                    );
                }
            }
        }
    }

    #[test]
    fn test_pk_broadcast_extractor() {
        let test_pk_broadcasts = get_pk_messages();
        for test_pk_broadcast in test_pk_broadcasts {
            let possible_pk_extract =
                osrs_broadcast_extractor::pk_broadcast_extractor(test_pk_broadcast.message.clone());
            match possible_pk_extract {
                None => {
                    info!(
                        "Failed to extract pk from message: {}",
                        test_pk_broadcast.message.clone()
                    );
                    assert!(false);
                }
                Some(pk_broadcast) => {
                    assert_eq!(pk_broadcast.winner, test_pk_broadcast.pk_broadcast.winner);
                    assert_eq!(pk_broadcast.loser, test_pk_broadcast.pk_broadcast.loser);
                    assert_eq!(
                        pk_broadcast.clan_mate,
                        test_pk_broadcast.pk_broadcast.clan_mate
                    );
                    assert_eq!(
                        pk_broadcast.gp_exchanged,
                        test_pk_broadcast.pk_broadcast.gp_exchanged
                    );
                    assert_eq!(
                        pk_broadcast.clan_mate_won,
                        test_pk_broadcast.pk_broadcast.clan_mate_won
                    );
                }
            }
        }
    }

    #[test]
    fn test_invite_broadcast_extractor() {
        let test_invite_broadcasts = get_invite_messages();
        for test_invite_broadcast in test_invite_broadcasts {
            let possible_invite_extract = osrs_broadcast_extractor::invite_broadcast_extractor(
                test_invite_broadcast.message.clone(),
            );
            match possible_invite_extract {
                None => {
                    info!(
                        "Failed to extract invite from message: {}",
                        test_invite_broadcast.message.clone()
                    );
                    assert!(false);
                }
                Some(invite_broadcast) => {
                    assert_eq!(
                        invite_broadcast.clan_mate,
                        test_invite_broadcast.invite_broadcast.clan_mate
                    );
                    assert_eq!(
                        invite_broadcast.new_clan_mate,
                        test_invite_broadcast.invite_broadcast.new_clan_mate
                    );
                }
            }
        }
    }

    #[test]
    fn test_levelmilestone_broadcast_extractor() {
        let test_levelmilestone_broadcasts = get_levelmilestone_messages();
        for test_levelmilestone_broadcast in test_levelmilestone_broadcasts {
            let possible_levelmilestone_extract =
                osrs_broadcast_extractor::levelmilestone_broadcast_extractor(
                    test_levelmilestone_broadcast.message.clone(),
                );
            match possible_levelmilestone_extract {
                None => {
                    info!(
                        "Failed to extract level milestone from message: {}",
                        test_levelmilestone_broadcast.message.clone()
                    );
                    assert!(false);
                }
                Some(levelmilestone_broadcast) => {
                    assert_eq!(
                        levelmilestone_broadcast.clan_mate,
                        test_levelmilestone_broadcast
                            .levelmilestone_broadcast
                            .clan_mate
                    );
                    assert_eq!(
                        levelmilestone_broadcast.skill_levelled,
                        test_levelmilestone_broadcast
                            .levelmilestone_broadcast
                            .skill_levelled
                    );
                    assert_eq!(
                        levelmilestone_broadcast.new_skill_level,
                        test_levelmilestone_broadcast
                            .levelmilestone_broadcast
                            .new_skill_level
                    );
                    assert_eq!(
                        levelmilestone_broadcast.skill_icon,
                        test_levelmilestone_broadcast
                            .levelmilestone_broadcast
                            .skill_icon
                    );
                }
            }
        }
    }

    #[test]
    fn test_xpmilestone_broadcast_extractor() {
        let test_xpmilestone_broadcasts = get_xpmilestone_messages();
        for test_xpmilestone_broadcast in test_xpmilestone_broadcasts {
            let possible_xpmilestone_extract =
                osrs_broadcast_extractor::xpmilestone_broadcast_extractor(
                    test_xpmilestone_broadcast.message.clone(),
                );
            match possible_xpmilestone_extract {
                None => {
                    info!(
                        "Failed to extract xp milestone from message: {}",
                        test_xpmilestone_broadcast.message.clone()
                    );
                    assert!(false);
                }
                Some(xpmilestone_broadcast) => {
                    assert_eq!(
                        xpmilestone_broadcast.clan_mate,
                        test_xpmilestone_broadcast.xpmilestone_broadcast.clan_mate
                    );
                    assert_eq!(
                        xpmilestone_broadcast.skill,
                        test_xpmilestone_broadcast.xpmilestone_broadcast.skill
                    );
                    assert_eq!(
                        xpmilestone_broadcast.new_skill_xp,
                        test_xpmilestone_broadcast
                            .xpmilestone_broadcast
                            .new_skill_xp
                    );
                    assert_eq!(
                        xpmilestone_broadcast.skill_icon,
                        test_xpmilestone_broadcast.xpmilestone_broadcast.skill_icon
                    );
                }
            }
        }
    }

    #[test]
    fn test_rank_is_proper_wiki_image() {
        let rank = "Deputy Owner";
        let rank_image = get_wiki_clan_rank_image_url(rank.to_string());
        assert_eq!(
            rank_image,
            "https://oldschool.runescape.wiki/images/Clan_icon_-_Deputy_owner.png".to_string()
        );
    }

    //Test data setup
    struct ItemMessageTest {
        message: String,
        player_it_happened_to: String,
        //This can be a raid drop, a pet, or a normal drop
        item_name: String,
        //This can be the amount of the item
        item_quantity: i64,
        //The value of the item
        item_value: Option<i64>,
        item_icon: Option<String>,
        discord_message: String,
    }

    struct PetDropTest {
        message: String,
        pet_drop: PetDropBroadcast,
    }

    struct QuestCompletedBroadcastTest {
        message: String,
        quest_completed: QuestCompletedBroadcast,
    }

    struct DiaryCompletedBroadcastTest {
        message: String,
        diary_completed: DiaryCompletedBroadcast,
    }

    struct PkBroadcastTest {
        message: String,
        pk_broadcast: PkBroadcast,
    }

    struct InviteBroadcastTest {
        message: String,
        invite_broadcast: InviteBroadcast,
    }

    struct LevelMilestoneBroadcastTest {
        message: String,
        levelmilestone_broadcast: LevelMilestoneBroadcast,
    }

    struct XPMilestoneBroadcastTest {
        message: String,
        xpmilestone_broadcast: XPMilestoneBroadcast,
    }

    fn get_raid_messages() -> Vec<ItemMessageTest> {
        let mut possible_raid_broadcasts: Vec<ItemMessageTest> = Vec::new();
        possible_raid_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received special loot from a raid: Twisted buckler."
                .to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Twisted buckler".to_string(),
            item_quantity: 1,
            item_value: None,
            item_icon: Some(
                "https://oldschool.runescape.wiki/images/Twisted_buckler_detail.png".to_string(),
            ),
            discord_message: "RuneScape Player received special loot from a raid: Twisted buckler."
                .to_string(),
        });
        possible_raid_broadcasts.push(ItemMessageTest {
            message: "Player received special loot from a raid: Twisted bow.".to_string(),
            player_it_happened_to: "Player".to_string(),
            item_name: "Twisted bow".to_string(),
            item_quantity: 1,
            item_value: None,
            item_icon: Some(
                "https://oldschool.runescape.wiki/images/Twisted_bow_detail.png".to_string(),
            ),
            discord_message: "Player received special loot from a raid: Twisted bow.".to_string(),
        });
        possible_raid_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received special loot from a raid: Tumeken's shadow (uncharged)".to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Tumeken's shadow (uncharged)".to_string(),
            item_quantity: 1,
            item_value: None,
            item_icon: Some("https://oldschool.runescape.wiki/images/Tumeken%27s_shadow_%28uncharged%29_detail.png".to_string()),
            discord_message: "RuneScape Player received special loot from a raid: Tumeken's shadow (uncharged)".to_string(),
        });
        possible_raid_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received special loot from a raid: Justiciar legguards."
                .to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Justiciar legguards".to_string(),
            item_quantity: 1,
            item_value: None,
            item_icon: Some(
                "https://oldschool.runescape.wiki/images/Justiciar_legguards_detail.png"
                    .to_string(),
            ),
            discord_message:
                "RuneScape Player received special loot from a raid: Justiciar legguards."
                    .to_string(),
        });
        return possible_raid_broadcasts;
    }

    fn get_pet_messages() -> Vec<PetDropTest> {
        let mut possible_pet_broadcasts: Vec<PetDropTest> = Vec::new();
        possible_pet_broadcasts.push(PetDropTest {
            message:
                "Runescape Player has a funny feeling like he's being followed: Butch at 194 kills."
                    .to_string(),
            pet_drop: PetDropBroadcast {
                player_it_happened_to: "Runescape Player".to_string(),
                pet_name: "Butch".to_string(),
                pet_icon: Some(
                    "https://oldschool.runescape.wiki/images/Butch_detail.png".to_string(),
                ),
                actions_optioned_at: Some(194),
                action_for_pet: Some("kills".to_string()),
            },
        });

        possible_pet_broadcasts.push(PetDropTest {
            message:
            "Runescape Vision has a funny feeling like she would have been followed: Heron at 11,212,255 XP."
                .to_string(),
            pet_drop: PetDropBroadcast {
                player_it_happened_to: "Runescape Vision".to_string(),
                pet_name: "Heron".to_string(),
                pet_icon: Some("https://oldschool.runescape.wiki/images/Heron_detail.png".to_string()),
                actions_optioned_at: Some(11_212_255),
                action_for_pet: Some("XP".to_string()),
            },
        });

        possible_pet_broadcasts.push(PetDropTest {
            message: "Runescape Player feels something weird sneaking into her backpack: Abyssal protector at 543 rift searches.".to_string(),
            pet_drop: PetDropBroadcast {
                player_it_happened_to: "Runescape Player".to_string(),
                pet_name: "Abyssal protector".to_string(),
                pet_icon: Some("https://oldschool.runescape.wiki/images/Abyssal_protector_detail.png".to_string()),
                actions_optioned_at: Some(543),
                action_for_pet: Some("rift searches".to_string()),
            },
        });

        possible_pet_broadcasts.push(PetDropTest {
            message: "Runescape Player has a funny feeling like she's being followed: Tiny tempor at 1,061 permits.".to_string(),
            pet_drop: PetDropBroadcast {
                player_it_happened_to: "Runescape Player".to_string(),
                pet_name: "Tiny tempor".to_string(),
                pet_icon: Some("https://oldschool.runescape.wiki/images/Tiny_tempor_detail.png".to_string()),
                actions_optioned_at: Some(1_061),
                action_for_pet: Some("permits".to_string()),
            },
        });

        possible_pet_broadcasts.push(PetDropTest {
            message: "Runescape Player has a funny feeling like she's being followed: Unknown Pet at 1,061 Fake Currency.".to_string(),
            pet_drop: PetDropBroadcast {
                player_it_happened_to: "Runescape Player".to_string(),
                pet_name: "Unknown Pet".to_string(),
                pet_icon: Some("https://oldschool.runescape.wiki/images/Unknown_Pet_detail.png".to_string()),
                actions_optioned_at: Some(1_061),
                action_for_pet: Some("Fake Currency".to_string()),
            },
        });
        return possible_pet_broadcasts;
    }

    fn get_drop_messages() -> Vec<ItemMessageTest> {
        let mut possible_drop_broadcasts: Vec<ItemMessageTest> = Vec::new();

        possible_drop_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received a drop: Abyssal whip (1,456,814 coins)."
                .to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Abyssal whip".to_string(),
            item_quantity: 1,
            item_value: Some(1_456_814),
            item_icon: Some(
                "https://oldschool.runescape.wiki/images/Abyssal_whip_detail.png".to_string(),
            ),
            discord_message: "RuneScape Player received a drop: Abyssal whip (1,456,814 coins)."
                .to_string(),
        });

        possible_drop_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received a drop: Unknown Item.".to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Unknown Item".to_string(),
            item_quantity: 1,
            item_value: Some(0),
            item_icon: Some(
                "https://oldschool.runescape.wiki/images/Unknown_Item_detail.png".to_string(),
            ),
            discord_message: "RuneScape Player received a drop: Unknown Item (0 coins)."
                .to_string(),
        });

        possible_drop_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received a drop: 587 x Cannonball (111,530 coins)."
                .to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Cannonball".to_string(),
            item_quantity: 587,
            item_value: Some(111530),
            item_icon: Some(
                "https://oldschool.runescape.wiki/images/Cannonball_detail.png".to_string(),
            ),
            discord_message: "RuneScape Player received a drop: 587 x Cannonball (111,530 coins)."
                .to_string(),
        });

        possible_drop_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received a drop: Awakener's orb (2,238,871 coins)."
                .to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Awakener's orb".to_string(),
            item_quantity: 1,
            item_value: Some(2_238_871),
            item_icon: Some(
                "https://oldschool.runescape.wiki/images/Awakener%27s_orb_detail.png".to_string(),
            ),
            discord_message: "RuneScape Player received a drop: Awakener's orb (2,238,871 coins)."
                .to_string(),
        });

        possible_drop_broadcasts.push(ItemMessageTest {
            message:
                "RuneScape Player received a drop: Voidwaker blade (39,648,370 coins) from Vet'ion."
                    .to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Voidwaker blade".to_string(),
            item_quantity: 1,
            item_value: Some(39_648_370),
            item_icon: Some(
                "https://oldschool.runescape.wiki/images/Voidwaker_blade_detail.png".to_string(),
            ),
            discord_message:
                "RuneScape Player received a drop: Voidwaker blade (39,648,370 coins).".to_string(),
        });

        return possible_drop_broadcasts;
    }

    fn get_quest_completed_messages() -> Vec<QuestCompletedBroadcastTest> {
        let mut possible_quest_completed_broadcasts: Vec<QuestCompletedBroadcastTest> = Vec::new();

        possible_quest_completed_broadcasts.push(QuestCompletedBroadcastTest {
            message: "RuneScape Player has completed a quest: The Fremennik Isles".to_string(),
            quest_completed: QuestCompletedBroadcast {
                player_it_happened_to: "RuneScape Player".to_string(),
                quest_name: "The Fremennik Isles".to_string(),
                quest_reward_scroll_icon: Some(
                    "https://oldschool.runescape.wiki/images/The_Fremennik_Isles_reward_scroll.png"
                        .to_string(),
                ),
            },
        });

        possible_quest_completed_broadcasts.push(QuestCompletedBroadcastTest {
            message: "RuneScapePlayer has completed a quest: Death Plateau".to_string(),
            quest_completed: QuestCompletedBroadcast {
                player_it_happened_to: "RuneScapePlayer".to_string(),
                quest_name: "Death Plateau".to_string(),
                quest_reward_scroll_icon: Some(
                    "https://oldschool.runescape.wiki/images/Death_Plateau_reward_scroll.png"
                        .to_string(),
                ),
            },
        });

        possible_quest_completed_broadcasts.push(QuestCompletedBroadcastTest {
            message: "RuneScape Player has completed a quest: The Fremennik Isles".to_string(),
            quest_completed: QuestCompletedBroadcast {
                player_it_happened_to: "RuneScape Player".to_string(),
                quest_name: "The Fremennik Isles".to_string(),
                quest_reward_scroll_icon: Some(
                    "https://oldschool.runescape.wiki/images/The_Fremennik_Isles_reward_scroll.png"
                        .to_string(),
                ),
            },
        });

        possible_quest_completed_broadcasts.push(QuestCompletedBroadcastTest {
            message: "RuneScape Player has completed a quest: The Tourist Trap".to_string(),
            quest_completed: QuestCompletedBroadcast {
                player_it_happened_to: "RuneScape Player".to_string(),
                quest_name: "The Tourist Trap".to_string(),
                quest_reward_scroll_icon: Some(
                    "https://oldschool.runescape.wiki/images/The_Tourist_Trap_reward_scroll.png"
                        .to_string(),
                ),
            },
        });

        possible_quest_completed_broadcasts.push(QuestCompletedBroadcastTest {
            message: "Player has completed a quest: Dragon Slayer II".to_string(),
            quest_completed: QuestCompletedBroadcast {
                player_it_happened_to: "Player".to_string(),
                quest_name: "Dragon Slayer II".to_string(),
                quest_reward_scroll_icon: Some(
                    "https://oldschool.runescape.wiki/images/Dragon_Slayer_II_reward_scroll.png"
                        .to_string(),
                ),
            },
        });

        possible_quest_completed_broadcasts
    }

    fn get_diary_completed_messages() -> Vec<DiaryCompletedBroadcastTest> {
        let mut possible_diary_completed_broadcasts: Vec<DiaryCompletedBroadcastTest> = Vec::new();

        possible_diary_completed_broadcasts.push(DiaryCompletedBroadcastTest {
            message: "RuneScape Player has completed the Easy Ardougne diary.".to_string(),
            diary_completed: DiaryCompletedBroadcast {
                player_it_happened_to: "RuneScape Player".to_string(),
                diary_name: "Ardougne diary".to_string(),
                diary_tier: DiaryTier::Easy,
            },
        });

        possible_diary_completed_broadcasts.push(DiaryCompletedBroadcastTest {
            message: "RuneScape Player has completed the Medium Karamja diary.".to_string(),
            diary_completed: DiaryCompletedBroadcast {
                player_it_happened_to: "RuneScape Player".to_string(),
                diary_name: "Karamja diary".to_string(),
                diary_tier: DiaryTier::Medium,
            },
        });

        possible_diary_completed_broadcasts.push(DiaryCompletedBroadcastTest {
            message: "RuneScape Player has completed the Hard Ardougne diary.".to_string(),
            diary_completed: DiaryCompletedBroadcast {
                player_it_happened_to: "RuneScape Player".to_string(),
                diary_name: "Ardougne diary".to_string(),
                diary_tier: DiaryTier::Hard,
            },
        });

        possible_diary_completed_broadcasts.push(DiaryCompletedBroadcastTest {
            message: "RuneScape Player has completed the Elite Lumbridge & Draynor diary."
                .to_string(),
            diary_completed: DiaryCompletedBroadcast {
                player_it_happened_to: "RuneScape Player".to_string(),
                diary_name: "Lumbridge & Draynor diary".to_string(),
                diary_tier: DiaryTier::Elite,
            },
        });

        possible_diary_completed_broadcasts
    }

    fn get_pk_messages() -> Vec<PkBroadcastTest> {
        let mut possible_pk_broadcasts: Vec<PkBroadcastTest> = Vec::new();
        // KANlEL OUTIS has been defeated by Veljenpojat in The Wilderness and lost (953,005 coins) worth of loot.
        // KANlEL OUTIS has defeated Emperor KB and received (972,728 coins) worth of loot!
        // Main Dangler has been defeated by Koishi Fumo in The Wilderness.
        // tikkok ALT has been defeated by WhatsA Dad in The Wilderness and lost (462,128 coins) worth of loot. Clearly tikkok ALT struggles with clicking.
        possible_pk_broadcasts.push(PkBroadcastTest{
            message: "KANlEL OUTIS has been defeated by Veljenpojat in The Wilderness and lost (953,005 coins) worth of loot.".to_string(),
            pk_broadcast: PkBroadcast {
                winner: "Veljenpojat".to_string(),
                loser: "KANlEL OUTIS".to_string(),
                clan_mate: "KANlEL OUTIS".to_string(),
                gp_exchanged: Some(953_005),
                clan_mate_won: false,
            }
        });

        possible_pk_broadcasts.push(PkBroadcastTest {
            message:
                "KANlEL OUTIS has defeated Emperor KB and received (972,728 coins) worth of loot!"
                    .to_string(),
            pk_broadcast: PkBroadcast {
                winner: "KANlEL OUTIS".to_string(),
                loser: "Emperor KB".to_string(),
                clan_mate: "KANlEL OUTIS".to_string(),
                gp_exchanged: Some(972_728),
                clan_mate_won: true,
            },
        });

        possible_pk_broadcasts.push(PkBroadcastTest {
            message: "Main Dangler has been defeated by Koishi Fumo in The Wilderness.".to_string(),
            pk_broadcast: PkBroadcast {
                winner: "Koishi Fumo".to_string(),
                loser: "Main Dangler".to_string(),
                clan_mate: "Main Dangler".to_string(),
                gp_exchanged: None,
                clan_mate_won: false,
            },
        });

        possible_pk_broadcasts.push(PkBroadcastTest {
            message:
                "tikkok ALT has been defeated by WhatsA Dad in The Wilderness and lost (462,128 coins) worth of loot. Clearly tikkok ALT struggles with clicking.".to_string(),
            pk_broadcast: PkBroadcast {
                winner: "WhatsA Dad".to_string(),
                loser: "tikkok ALT".to_string(),
                clan_mate: "tikkok ALT".to_string(),
                gp_exchanged: Some(462_128),
                clan_mate_won: false,
            },
        });

        possible_pk_broadcasts.push(PkBroadcastTest {
            message:
                "KANlEL OUTIS has been defeated by sha huss in The Wilderness and lost (948,980 coins) worth of loot....and now everyone knows.".to_string(),
            pk_broadcast: PkBroadcast {
                winner: "sha huss".to_string(),
                loser: "KANlEL OUTIS".to_string(),
                clan_mate: "KANlEL OUTIS".to_string(),
                gp_exchanged: Some(948_980),
                clan_mate_won: false,
            },
        });

        possible_pk_broadcasts.push(PkBroadcastTest {
            message:
                "KANlEL OUTIS has been defeated by Omar and lost (14,548,386 coins) worth of loot."
                    .to_string(),
            pk_broadcast: PkBroadcast {
                winner: "Omar".to_string(),
                loser: "KANlEL OUTIS".to_string(),
                clan_mate: "KANlEL OUTIS".to_string(),
                gp_exchanged: Some(14_548_386),
                clan_mate_won: false,
            },
        });

        possible_pk_broadcasts
    }

    fn get_invite_messages() -> Vec<InviteBroadcastTest> {
        let mut possible_invite_broadcasts: Vec<InviteBroadcastTest> = Vec::new();
        // Victor Locke has been invited into the clan by IRuneNakey.
        // KingConley has been invited into the clan by kanga roe.
        // RUKAl has been invited into the clan by l cant see.
        possible_invite_broadcasts.push(InviteBroadcastTest {
            message: "Victor Locke has been invited into the clan by IRuneNakey.".to_string(),
            invite_broadcast: InviteBroadcast {
                clan_mate: "IRuneNakey".to_string(),
                new_clan_mate: "Victor Locke".to_string(),
            },
        });

        possible_invite_broadcasts.push(InviteBroadcastTest {
            message: "KingConley has been invited into the clan by kanga roe.".to_string(),
            invite_broadcast: InviteBroadcast {
                clan_mate: "kanga roe".to_string(),
                new_clan_mate: "KingConley".to_string(),
            },
        });

        possible_invite_broadcasts.push(InviteBroadcastTest {
            message: "RUKAl has been invited into the clan by l cant see.".to_string(),
            invite_broadcast: InviteBroadcast {
                clan_mate: "l cant see".to_string(),
                new_clan_mate: "RUKAl".to_string(),
            },
        });

        possible_invite_broadcasts
    }

    fn get_levelmilestone_messages() -> Vec<LevelMilestoneBroadcastTest> {
        let mut possible_levelmilestone_broadcasts: Vec<LevelMilestoneBroadcastTest> = Vec::new();
        // Th3TRiPPyOn3 has reached Defence level 70.
        // MechaPanzer has reached combat level 104.
        // I Vision I has reached a total level of 2225.
        // Zillamanjaro has reached the highest possible combat level of 126!
        // Sad Bug has reached the highest possible total level of 2277!
        possible_levelmilestone_broadcasts.push(LevelMilestoneBroadcastTest {
            message: "Th3TRiPPyOn3 has reached Defence level 70.".to_string(),
            levelmilestone_broadcast: LevelMilestoneBroadcast {
                clan_mate: "Th3TRiPPyOn3".to_string(),
                skill_levelled: "Defence".to_string(),
                new_skill_level: "70".to_string(),
                skill_icon: Some(
                    "https://oldschool.runescape.wiki/images/Defence_icon_(detail).png".to_string(),
                ),
            },
        });

        possible_levelmilestone_broadcasts.push(LevelMilestoneBroadcastTest {
            message: "MechaPanzer has reached combat level 104.".to_string(),
            levelmilestone_broadcast: LevelMilestoneBroadcast {
                clan_mate: "MechaPanzer".to_string(),
                skill_levelled: "combat".to_string(),
                new_skill_level: "104".to_string(),
                skill_icon: Some(
                    "https://oldschool.runescape.wiki/images/combat_icon_(detail).png".to_string(),
                ),
            },
        });

        possible_levelmilestone_broadcasts.push(LevelMilestoneBroadcastTest {
            message: "I Vision I has reached a total level of 2225.".to_string(),
            levelmilestone_broadcast: LevelMilestoneBroadcast {
                clan_mate: "I Vision I".to_string(),
                skill_levelled: "total".to_string(),
                new_skill_level: "2225".to_string(),
                skill_icon: Some(
                    "https://oldschool.runescape.wiki/images/total_icon_(detail).png".to_string(),
                ),
            },
        });

        possible_levelmilestone_broadcasts.push(LevelMilestoneBroadcastTest {
            message: "Zillamanjaro has reached the highest possible combat level of 126!"
                .to_string(),
            levelmilestone_broadcast: LevelMilestoneBroadcast {
                clan_mate: "Zillamanjaro".to_string(),
                skill_levelled: "combat".to_string(),
                new_skill_level: "126".to_string(),
                skill_icon: Some(
                    "https://oldschool.runescape.wiki/images/combat_icon_(detail).png".to_string(),
                ),
            },
        });

        possible_levelmilestone_broadcasts.push(LevelMilestoneBroadcastTest {
            message: "Sad Bug has reached the highest possible total level of 2277!".to_string(),
            levelmilestone_broadcast: LevelMilestoneBroadcast {
                clan_mate: "Sad Bug".to_string(),
                skill_levelled: "total".to_string(),
                new_skill_level: "2277".to_string(),
                skill_icon: Some(
                    "https://oldschool.runescape.wiki/images/total_icon_(detail).png".to_string(),
                ),
            },
        });

        possible_levelmilestone_broadcasts
    }

    fn get_xpmilestone_messages() -> Vec<XPMilestoneBroadcastTest> {
        let mut possible_xpmilestone_broadcasts: Vec<XPMilestoneBroadcastTest> = Vec::new();
        // Noble Five has reached 78,000,000 XP in Fishing.
        // Matrese has reached 15,000,000 XP in Fishing.
        // Marsel has reached 200,000,000 XP in Cooking.
        possible_xpmilestone_broadcasts.push(XPMilestoneBroadcastTest {
            message: "Noble Five has reached 78,000,000 XP in Fishing.".to_string(),
            xpmilestone_broadcast: XPMilestoneBroadcast {
                clan_mate: "Noble Five".to_string(),
                skill: "Fishing".to_string(),
                new_skill_xp: "78,000,000".to_string(),
                skill_icon: Some(
                    "https://oldschool.runescape.wiki/images/Fishing_icon_(detail).png".to_string(),
                ),
            },
        });

        possible_xpmilestone_broadcasts.push(XPMilestoneBroadcastTest {
            message: "Matrese has reached 15,000,000 XP in Fishing.".to_string(),
            xpmilestone_broadcast: XPMilestoneBroadcast {
                clan_mate: "Matrese".to_string(),
                skill: "Fishing".to_string(),
                new_skill_xp: "15,000,000".to_string(),
                skill_icon: Some(
                    "https://oldschool.runescape.wiki/images/Fishing_icon_(detail).png".to_string(),
                ),
            },
        });

        possible_xpmilestone_broadcasts.push(XPMilestoneBroadcastTest {
            message: "Marsel has reached 200,000,000 XP in Cooking.".to_string(),
            xpmilestone_broadcast: XPMilestoneBroadcast {
                clan_mate: "Marsel".to_string(),
                skill: "Cooking".to_string(),
                new_skill_xp: "200,000,000".to_string(),
                skill_icon: Some(
                    "https://oldschool.runescape.wiki/images/Cooking_icon_(detail).png".to_string(),
                ),
            },
        });

        possible_xpmilestone_broadcasts
    }
}
