pub mod osrs_broadcast_extractor {
    use regex::Regex;
    use once_cell::sync::Lazy;
    use serde::{Deserialize, Serialize};

    static RAID_BROADCAST_EXTRACTOR: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(?P<player_name>.*?) received special loot from a raid: (?P<item>.*?)([.]|$)"#,).unwrap());
    static DROP_BROADCAST_EXTRACTOR: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(?P<player_name>.*?) received a drop: (?:((?P<quantity>[,\d]+) x )?)(?P<item>.*?)(?: \((?P<value>[,\d]+) coins\))?(?: from .*?)?[.]?$"#).unwrap());
    static PET_BROADCAST_EXTRACTOR: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(?P<player_name>.*?) (?:has a funny feeling.*?|feels something weird sneaking into (?P<pronoun>her|his) backpack): (?P<pet_name>.*?) at (?P<count>[,\d]+) (?P<count_type>.*?)[.]$"#).unwrap());
    static QUEST_COMPLETED_BROADCAST_EXTRACTOR: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(?P<player_name>.*?) has completed a quest: (?P<quest_name>.+)$"#,).unwrap());
    static DIARY_COMPLETED_BROADCAST_EXTRACTOR: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(?P<player_name>.*?) has completed the (?P<diary_tier>Easy|Medium|Hard|Elite) (?P<diary_name>.*?).$"#).unwrap());
    static PK_BROADCAST_EXTRACTOR_WINNER: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(?P<winner_name>.*?) has defeated (?P<loser_name>.*?) and received \((?P<gp_value>[0-9,]+) coins\) worth of loot!"#).unwrap());
    static PK_BROADCAST_EXTRACTOR_LOSER: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(?P<loser_name>.*?) has been defeated by (?P<winner_name>.*?)(?: in (?P<location>The Wilderness))?(?: and lost \((?P<gp_value>[0-9,]+) coins\) worth of loot)?[!.]"#).unwrap());
    static INVITE_BROADCAST_EXTRACTOR: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(?P<clan_joiner>.*?) has been invited into the clan by (?P<clan_inviter>.*?).$"#,).unwrap());
    static LEVELMILESTONE_BROADCAST_EXTRACTOR: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(?P<clan_mate>.*?) has reached (?:a )?(?:the highest possible )?(?P<skill>.*?) level(?: of)? (?P<level>.*?)[!.]"#).unwrap());
    static XPMILESTONE_BROADCAST_EXTRACTOR: Lazy<Regex>  =  Lazy::new(|| Regex::new(r#"^(?P<clan_member>.*?) has reached (?P<xp>.*?) XP in (?P<skill>.*?)[!.]"#,).unwrap());
    static COLLECTION_LOG_BROADCAST_EXTRACTOR: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?P<name>[\w\s]+) received a new collection log item: (?P<item>.+?) \((?P<number>\d+)/\d+\)").unwrap());
    static LEFT_THE_CLAN_BROADCAST_EXTRACTOR: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?P<player>[\w\s]+) has left the clan.$").unwrap());
    static EXPELLED_FROM_CLAN_BROADCAST_EXTRACTOR: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?P<mod>[\w\s]+) has expelled (?P<player>[\w\s]+) from the clan.$").unwrap());
    static COFFER_DONATION_BROADCAST_EXTRACTOR: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?P<player>[\w\s]+) has (withdrawn|deposited) (?P<gp>[0-9,]+) coins (from|into) the coffer.").unwrap());
    static COFFER_WITHDRAWAL_BROADCAST_EXTRACTOR: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?P<player>[\w\s]+) has (withdrawn|deposited) (?P<gp>[0-9,]+) coins (from|into) the coffer.").unwrap());
    // RuneScape Player has achieved a new Vorkath personal best: 2:28
    static PERSONAL_BEST_BROADCAST_EXTRACTOR: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(?P<player>[\w\s]+) has achieved a new (?P<activity>[\w\s\-'\.]+) personal best: (?<time>[\d:]+)"#,).unwrap());
    static PERSONAL_BEST_BROADCAST_EXTRACTOR_RAID: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(?P<player>[\w\s]+) has achieved a new (?P<raid>[\w\s]+(?:\: [\w\s]+)?) \([Tt]eam [Ss]ize: (?P<team_size>[\w\s]+)\)(?:(?P<variant>[\w\s]+)?) personal best: (?<time>[\d:]+(?:\.\d{2})?)"#,).unwrap());

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

    pub struct CollectionLogBroadcast {
        pub player_it_happened_to: String,
        pub item_name: String,
        pub log_slots: i64,
        pub item_icon: Option<String>,
    }

    pub enum CofferTransaction {
        Withdrawal,
        Donation,
    }

    pub struct CofferTransactionBroadcast {
        pub player: String,
        pub gp: i64,
        pub transaction_type: CofferTransaction,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct PersonalBestBroadcast {
        pub player: String,
        pub activity: String,
        pub variant: Option<String>,
        pub time_in_seconds: f64,
    }

    //Leagues broadcasts
    //TODO may come back and add more descriptive broadcasts for Leagues, but right now just going the "firehose" route
    // #[derive(Clone, Debug, Serialize, Deserialize)]
    // pub struct AreaUnlockBroadcast {
    //     pub area: String,
    //     pub player: String
    // }

    // #[derive(Clone, Debug, Serialize, Deserialize)]
    // pub struct LeaguesRankBroadcast {
    //     pub area: String,
    //     pub player: String
    // }

    // #[derive(Clone, Debug, Serialize, Deserialize)]
    // pub struct CombatMasteriesBroadcast {
    //     pub player: String,
    //     pub combat_mastery: String,
    //     pub rank: i64,
    // }
    
    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
    pub enum LeaguesBroadCastType {
        AreaUnlock,
        LeaguesRank,
        CombatMasteries,
        RelicTier,
        //Normal broadcast but for leagues, like drops
        NormalBroadCast
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
        CollectionLog,
        LeftTheClan,
        ExpelledFromClan,
        CofferDonation,
        CofferWithdrawal,
        PersonalBest,        
        //Leagues Broadcasts        
        AreaUnlock,
        LeaguesRank,
        CombatMasteries,
        RelicTier,
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
                BroadcastType::CollectionLog => "Collection Log".to_string(),
                BroadcastType::Unknown => "Unknown".to_string(),
                BroadcastType::LeftTheClan => "Left The Clan".to_string(),
                BroadcastType::ExpelledFromClan => "Expelled From Clan".to_string(),
                BroadcastType::CofferDonation => "Coffer Donation".to_string(),
                BroadcastType::CofferWithdrawal => "Coffer Withdrawal".to_string(),
                BroadcastType::PersonalBest => "Personal Best".to_string(),                
                //Leagues Broadcasts
                BroadcastType::AreaUnlock => "Area Unlock".to_string(),
                BroadcastType::LeaguesRank => "Leagues Rank".to_string(),
                BroadcastType::CombatMasteries => "Combat Masteries".to_string(),
                BroadcastType::RelicTier => "Relic Tier".to_string(),
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
                "Collection Log" => BroadcastType::CollectionLog,
                "Personal Best" => BroadcastType::PersonalBest,
                //Leagues Broadcasts
                "Area Unlock" => BroadcastType::AreaUnlock,
                "Leagues Rank" => BroadcastType::LeaguesRank,
                "Combat Masteries" => BroadcastType::CombatMasteries,
                "Relic Tier" => BroadcastType::RelicTier,
                _ => BroadcastType::Unknown,
            }
        }

        pub fn iter() -> Vec<BroadcastType> {
            vec![
                BroadcastType::ItemDrop,
                BroadcastType::PetDrop,
                BroadcastType::Quest,
                BroadcastType::Diary,
                BroadcastType::RaidDrop,
                BroadcastType::Pk,
                BroadcastType::Invite,
                BroadcastType::LootKey,
                BroadcastType::XPMilestone,
                BroadcastType::LevelMilestone,
                BroadcastType::CollectionLog,
                BroadcastType::Unknown,
                BroadcastType::LeftTheClan,
                BroadcastType::ExpelledFromClan,
                BroadcastType::CofferDonation,
                BroadcastType::CofferWithdrawal,
                BroadcastType::PersonalBest,
                //Leagues Broadcasts
                BroadcastType::AreaUnlock,
                BroadcastType::LeaguesRank,
                BroadcastType::CombatMasteries,
                BroadcastType::RelicTier,
            ]
        }

        pub fn to_slug(&self) -> String {
            match self {
                _ => self.to_string().replace(" ", "_"),
            }
        }
    }

    pub fn raid_broadcast_extractor(message: String) -> Option<DropItemBroadcast> {
        if let Some(caps) = RAID_BROADCAST_EXTRACTOR.captures(message.as_str()) {
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
        }
    }

    pub fn drop_broadcast_extractor(message: String) -> Option<DropItemBroadcast> {
        if let Some(caps) = DROP_BROADCAST_EXTRACTOR.captures(message.as_str()) {
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
        }
    }

    pub fn pet_broadcast_extractor(message: String) -> Option<PetDropBroadcast> {
        if let Some(caps) = PET_BROADCAST_EXTRACTOR.captures(message.as_str()) {
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
        if let Some(caps) = QUEST_COMPLETED_BROADCAST_EXTRACTOR.captures(message.as_str()) {
            let player_name = caps.name("player_name").unwrap().as_str();
            let quest_name = caps.name("quest_name").unwrap().as_str();

            Some(QuestCompletedBroadcast {
                player_it_happened_to: player_name.to_string(),
                quest_name: quest_name.to_string(),
                quest_reward_scroll_icon: Some(get_quest_reward_scroll(quest_name.to_string())),
            })
        } else {
            None
        }
    }

    pub fn diary_completed_broadcast_extractor(message: String) -> Option<DiaryCompletedBroadcast> {
        if let Some(caps) = DIARY_COMPLETED_BROADCAST_EXTRACTOR.captures(message.as_str()) {
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
        }
    }

    pub fn pk_broadcast_extractor(message: String) -> Option<PkBroadcast> {
        let re = if message.contains("defeated by") {
            &PK_BROADCAST_EXTRACTOR_LOSER
        } else {
            &PK_BROADCAST_EXTRACTOR_WINNER
        };

        if let Some(caps) = re.captures(message.as_str()) {
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
        }
    }

    pub fn invite_broadcast_extractor(message: String) -> Option<InviteBroadcast> {
        if let Some(caps) = INVITE_BROADCAST_EXTRACTOR.captures(message.as_str()) {
            let clan_mate = caps.name("clan_inviter").unwrap().as_str();
            let new_clan_mate = caps.name("clan_joiner").unwrap().as_str();
            Some(InviteBroadcast {
                clan_mate: clan_mate.to_string(),
                new_clan_mate: new_clan_mate.to_string(),
            })
        } else {
            None
        }
    }

    pub fn levelmilestone_broadcast_extractor(message: String) -> Option<LevelMilestoneBroadcast> {
        if let Some(caps) = LEVELMILESTONE_BROADCAST_EXTRACTOR.captures(message.as_str()) {
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
        }
    }

    pub fn xpmilestone_broadcast_extractor(message: String) -> Option<XPMilestoneBroadcast> {
        if let Some(caps) = XPMILESTONE_BROADCAST_EXTRACTOR.captures(message.as_str()) {
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
        }
    }

    pub fn collection_log_broadcast_extractor(message: String) -> Option<CollectionLogBroadcast> {
        if let Some(captures) = COLLECTION_LOG_BROADCAST_EXTRACTOR.captures(message.as_str()) {
            let name = captures.name("name").unwrap().as_str();
            let item = captures.name("item").unwrap().as_str();
            let number = captures.name("number").unwrap().as_str();

            Some(CollectionLogBroadcast {
                player_it_happened_to: name.to_string(),
                item_name: item.to_string(),
                log_slots: number.parse().unwrap(),
                item_icon: Some(get_wiki_image_url(item.to_string())),
            })
        } else {
            None
        }
    }

    pub fn left_the_clan_broadcast_extractor(message: String) -> Option<String> {
        if let Some(captures) = LEFT_THE_CLAN_BROADCAST_EXTRACTOR.captures(message.as_str()) {
            let name = captures.name("player").unwrap().as_str();

            Some(name.to_string())
        } else {
            None
        }
    }

    pub fn expelled_from_clan_broadcast_extractor(message: String) -> Option<String> {
        if let Some(captures) = EXPELLED_FROM_CLAN_BROADCAST_EXTRACTOR.captures(message.as_str()) {
            let name = captures.name("player").unwrap().as_str();

            Some(name.to_string())
        } else{
            None
        } 
    }

    pub fn coffer_donation_broadcast_extractor(
        message: String,
    ) -> Option<CofferTransactionBroadcast> {
        if let Some(captures) = COFFER_DONATION_BROADCAST_EXTRACTOR.captures(message.as_str()) {
            let player = captures.name("player").unwrap().as_str();
            let gp = captures
                .name("gp")
                .unwrap()
                .as_str()
                .replace(",", "")
                .parse()
                .unwrap();

            Some(CofferTransactionBroadcast {
                player: player.to_string(),
                gp,
                transaction_type: CofferTransaction::Donation,
            })
        } else {
            None
        }
    }

    pub fn coffer_withdrawal_broadcast_extractor(
        message: String,
    ) -> Option<CofferTransactionBroadcast> {
        if let Some(captures) = COFFER_WITHDRAWAL_BROADCAST_EXTRACTOR.captures(message.as_str()) {
            let player = captures.name("player").unwrap().as_str();
            let gp = captures
                .name("gp")
                .unwrap()
                .as_str()
                .replace(",", "")
                .parse()
                .unwrap();

            Some(CofferTransactionBroadcast {
                player: player.to_string(),
                gp,
                transaction_type: CofferTransaction::Withdrawal,
            })
        } else {
            None
        }
    }

    pub fn personal_best_broadcast_extractor(message: String) -> Option<PersonalBestBroadcast> {
        if let Some(captures) = PERSONAL_BEST_BROADCAST_EXTRACTOR.captures(message.as_str()) {
            let player = captures.name("player").unwrap().as_str();
            let activity = captures.name("activity").unwrap().as_str();
            let time = captures.name("time").unwrap().as_str();

            return Some(PersonalBestBroadcast {
                player: player.to_string(),
                activity: activity.to_string(),            
                time_in_seconds: osrs_time_parser(time),
                //Will prob need to look at hallow sepulchre and other activities that have variants here
                variant: None,
            })
        }

        if let Some(captures) = PERSONAL_BEST_BROADCAST_EXTRACTOR_RAID.captures(message.as_str()) {
            let player = captures.name("player").unwrap().as_str();
            let raid = captures.name("raid").unwrap().as_str();            
            let time = captures.name("time").unwrap().as_str();
            let team_size = captures.name("team_size").map_or("", |m| m.as_str());
            let variant = captures.name("variant").map_or("", |m| m.as_str());
            
            let full_raid: String;
            if variant.is_empty() {
                full_raid = raid.to_string();
            } else {
                full_raid = format!("{} {}", raid, variant.trim());
            }

            let raid_name = match raid {
                x if x.contains("Xeric") => "Chambers of Xeric",
                x if x.contains("Blood") => "Theatre of Blood",
                x if x.contains("Tombs") => "Tombs of Amascut",
                _ => raid,
            };

            Some(PersonalBestBroadcast {
                player: player.to_string(),
                activity: raid_name.to_string(),
                time_in_seconds: osrs_time_parser(time),
                variant: raid_name_standardize(full_raid, team_size)
            })
        } else {
            None
        }
    }

    pub fn leagues_catch_all_broadcast_extractor(
        message: String,
    ) -> Option<LeaguesBroadCastType> {
        if message.contains("has earned") && message.contains("Combat mastery") {
            return Some(LeaguesBroadCastType::CombatMasteries);
        }

        if message.contains("has unlocked") && message.contains("League area") {
            return Some(LeaguesBroadCastType::AreaUnlock);
        }

        if message.contains("has unlocked") && message.contains("League relic!") {
            return Some(LeaguesBroadCastType::RelicTier);
        }
        None
    }

    /// Parses a time string in the format of `HH:MM:SS` or `MM:SS` and returns the time in seconds
    pub fn osrs_time_parser(time: &str) -> f64 {
        let split_sub_second: Vec<&str> = time.split(".").collect();
        let sub_second_fraction = format!(".{:}", split_sub_second.get(1).unwrap_or(&"0"))
            .parse::<f64>()
            .unwrap_or(0.0);

        let split_time: Vec<&str> = split_sub_second[0].split(":").collect();

        if split_time.len() == 2 {
            let minutes_seconds = split_time[0].parse::<f64>().unwrap_or(0.0) * 60.0;
            let seconds = split_time[1].parse::<f64>().unwrap_or(0.0);
            return minutes_seconds + seconds + sub_second_fraction;
        }
        if split_time.len() == 3 {
            let hours_seconds = split_time[0].parse::<f64>().unwrap_or(0.0) * 3600.0;
            let minutes_seconds = split_time[1].parse::<f64>().unwrap_or(0.0) * 60.0;
            let seconds = split_time[2].parse::<f64>().unwrap_or(0.0);
            return hours_seconds + minutes_seconds + seconds + sub_second_fraction;
        }

        0.0
    }

    
    /// Takes the OSRS in game broadcast and makes it a bit more standard between raids for easier parsing with RL
    fn raid_name_standardize(raid_from_chat: String, team_size: &str) -> Option<String> {
  

        if raid_from_chat.contains("Chambers of Xeric Challenge Mode") {    
            return Some(format!("Chambers of Xeric Challenge Mode {}", team_size));
        }

        if raid_from_chat.contains("Chambers of Xeric") {    
            return Some(format!("Chambers of Xeric {}", team_size));
        }

        let team_size_with_players = if team_size == "1" {
            "Solo".to_string()
        }else{
            format!("{} players", team_size)
        };

        if raid_from_chat.contains("Theatre of Blood") && raid_from_chat.contains("Hard mode") {
            return Some(format!("Theatre of Blood Hard Mode {}", team_size_with_players));
        }         

        if raid_from_chat.contains("Theatre of Blood") && raid_from_chat.contains("Entry mode") {
            return Some("Theatre of Blood Entry Mode".to_string());
        }

        if raid_from_chat.contains("Theatre of Blood") {
            return Some(format!("Theatre of Blood {}", team_size_with_players));
        }

        if raid_from_chat.contains("Tombs of Amascut") && raid_from_chat.contains("Entry mode"){
            return Some(format!("Tombs of Amascut Entry Mode {}", team_size_with_players));
        }

        if raid_from_chat.contains("Tombs of Amascut") && raid_from_chat.contains("Expert mode") {
            return Some(format!("Tombs of Amascut Expert Mode {}", team_size_with_players));
        }

        if raid_from_chat.contains("Tombs of Amascut") {
            return Some(format!("Tombs of Amascut {}", team_size_with_players));
        }

        None
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
        if message_content.contains("received a new collection log item:") {
            return BroadcastType::CollectionLog;
        }
        if message_content.contains("has left the clan.") {
            return BroadcastType::LeftTheClan;
        }
        if message_content.contains("has expelled") && message_content.contains("from the clan.") {
            return BroadcastType::ExpelledFromClan;
        }
        if message_content.contains("deposited") && message_content.contains("the coffer.") {
            return BroadcastType::CofferDonation;
        }
        if message_content.contains("withdrawn") && message_content.contains("from the coffer.") {
            return BroadcastType::CofferWithdrawal;
        }
        if message_content.contains("has achieved a new")
            && message_content.contains("personal best:")
        {
            return BroadcastType::PersonalBest;
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
        get_wiki_clan_rank_image_url, CofferTransaction, CofferTransactionBroadcast,
        CollectionLogBroadcast, DiaryCompletedBroadcast, DiaryTier, InviteBroadcast,
        LevelMilestoneBroadcast, PersonalBestBroadcast, PetDropBroadcast, PkBroadcast,
        QuestCompletedBroadcast, XPMilestoneBroadcast,
    };
    use osrs_broadcast_extractor::LeaguesBroadCastType;
    use rstest::rstest;
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
    fn test_get_collection_log_type_broadcast() {
        let test_collection_logs = get_collection_log_messages();
        for test_collection_log in test_collection_logs {
            let broadcast_type =
                osrs_broadcast_extractor::get_broadcast_type(test_collection_log.message);
            assert!(matches!(
                broadcast_type,
                osrs_broadcast_extractor::BroadcastType::CollectionLog
            ));
        }
    }

    #[test]
    fn test_get_left_the_clan_type_broadcast() {
        let test_left_the_clan = get_has_left_the_clan_messages();
        for test_left_the_clan in test_left_the_clan {
            let broadcast_type =
                osrs_broadcast_extractor::get_broadcast_type(test_left_the_clan.message);
            assert!(matches!(
                broadcast_type,
                osrs_broadcast_extractor::BroadcastType::LeftTheClan
            ));
        }
    }

    #[test]
    fn test_get_expelled_from_clan_type_broadcast() {
        let test_expelled_from_clan = get_expelled_from_clan_messages();
        for test_expelled_from_clan in test_expelled_from_clan {
            let broadcast_type =
                osrs_broadcast_extractor::get_broadcast_type(test_expelled_from_clan.message);
            assert!(matches!(
                broadcast_type,
                osrs_broadcast_extractor::BroadcastType::ExpelledFromClan
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
    fn test_collection_log_extractor() {
        let test_collections = get_collection_log_messages();
        for test_collection in test_collections {
            let possible_collection_extract =
                osrs_broadcast_extractor::collection_log_broadcast_extractor(
                    test_collection.message.clone(),
                );
            let collection_extract = possible_collection_extract.unwrap();
            assert_eq!(
                collection_extract.log_slots,
                test_collection.broadcast.log_slots
            );
            assert_eq!(
                collection_extract.player_it_happened_to,
                test_collection.broadcast.player_it_happened_to
            );
            assert_eq!(
                collection_extract.item_name,
                test_collection.broadcast.item_name
            );
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

        let goblin_rank = "Goblin";
        assert_eq!(
            get_wiki_clan_rank_image_url(goblin_rank.to_string()),
            "https://oldschool.runescape.wiki/images/Clan_icon_-_Goblin.png".to_string()
        );

    }

    #[test]
    fn test_left_clan_extractor() {
        let test_left_clan = get_has_left_the_clan_messages();
        for test_left_clan in test_left_clan {
            let possible_left_clan_extract =
                osrs_broadcast_extractor::left_the_clan_broadcast_extractor(
                    test_left_clan.message.clone(),
                );
            let player_who_left = possible_left_clan_extract.unwrap();
            assert_eq!(player_who_left, test_left_clan.broadcast);
        }
    }

    #[test]
    fn test_expelled_from_clan_extractor() {
        let test_expelled_from_clan = get_expelled_from_clan_messages();
        for test_expelled_from_clan in test_expelled_from_clan {
            let possible_expelled_from_clan_extract =
                osrs_broadcast_extractor::expelled_from_clan_broadcast_extractor(
                    test_expelled_from_clan.message.clone(),
                );
            let player_who_was_expelled = possible_expelled_from_clan_extract.unwrap();
            assert_eq!(player_who_was_expelled, test_expelled_from_clan.broadcast);
        }
    }

    #[test]
    fn test_coffer_donation_extractor() {
        let test_coffer_donation = get_clan_coffer_deposit_broadcast_messages();
        for test_coffer_donation in test_coffer_donation {
            let possible_coffer_donation_extract =
                osrs_broadcast_extractor::coffer_donation_broadcast_extractor(
                    test_coffer_donation.message.clone(),
                );
            let coffer_donation = possible_coffer_donation_extract.unwrap();
            assert_eq!(
                coffer_donation.player,
                test_coffer_donation.broadcast.player
            );
            assert_eq!(coffer_donation.gp, test_coffer_donation.broadcast.gp);
        }
    }

    #[test]
    fn test_coffer_withdrawal_extractor() {
        let test_coffer_withdrawal = get_clan_coffer_withdraw_broadcast_messages();
        for test_coffer_withdrawal in test_coffer_withdrawal {
            let possible_coffer_withdrawal_extract =
                osrs_broadcast_extractor::coffer_withdrawal_broadcast_extractor(
                    test_coffer_withdrawal.message.clone(),
                );
            let coffer_withdrawal = possible_coffer_withdrawal_extract.unwrap();
            assert_eq!(
                coffer_withdrawal.player,
                test_coffer_withdrawal.broadcast.player
            );
            assert_eq!(coffer_withdrawal.gp, test_coffer_withdrawal.broadcast.gp);
        }
    }

    #[test]
    fn test_personal_best_broadcast_extractor() {
        let test_personal_best = get_pbs_broadcast_messages();
        for test_personal_best in test_personal_best {
            let possible_personal_best_extract =
                osrs_broadcast_extractor::personal_best_broadcast_extractor(
                    test_personal_best.message.clone(),
                );
            let personal_best = possible_personal_best_extract;
            match personal_best {
                None => {
                    println!("❌: {:?}", test_personal_best.message.clone());
                    assert!(false);
                }
                Some(personal_best) => {
                    assert_eq!(personal_best.player, test_personal_best.broadcast.player);
                    assert_eq!(
                        personal_best.activity,
                        test_personal_best.broadcast.activity
                    );
                    assert_eq!(
                        personal_best.time_in_seconds,
                        test_personal_best.broadcast.time_in_seconds
                    );
                    assert_eq!(personal_best.variant, test_personal_best.broadcast.variant);
                    println!("✅: {:?}", test_personal_best.message.clone());
                }
            }
        }
    }

    #[test]
    fn test_leagues_catch_all_broadcast_extractor() {
        let test_leagues_catch_all = get_leagues_catch_all_broadcast_messages();
        for test_leagues_catch_all in test_leagues_catch_all {
            let possible_leagues_catch_all_extract =
                osrs_broadcast_extractor::leagues_catch_all_broadcast_extractor(
                    test_leagues_catch_all.message.clone(),
                );
            let leagues_catch_all = possible_leagues_catch_all_extract.unwrap();
            info!("{:?}", leagues_catch_all);
            assert_eq!(leagues_catch_all, test_leagues_catch_all.broadcast);
        }
    }


    #[rstest]
    #[case("0:56.40", 56.40)]
    #[case("1:25", 85.0)]
    #[case("1:19.80", 79.8)]
    #[case("1:15.00", 75.00)]
    #[case("21:55.80", 1_315.80)]
    #[case("1:30:00", 5_400.00)]
    #[case("1:30:00.45", 5_400.45)]
    #[case("36:37.80", 2_197.80)]
    fn test_osrs_time_parser(#[case] time: &str, #[case] expected: f64) {
        let actual = osrs_broadcast_extractor::osrs_time_parser(time);
        assert_eq!(actual, expected);
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
        _discord_message: String,
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

    struct TestBroadcast<T> {
        message: String,
        broadcast: T,
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
            _discord_message:
                "RuneScape Player received special loot from a raid: Twisted buckler.".to_string(),
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
            _discord_message: "Player received special loot from a raid: Twisted bow.".to_string(),
        });
        possible_raid_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received special loot from a raid: Tumeken's shadow (uncharged)".to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Tumeken's shadow (uncharged)".to_string(),
            item_quantity: 1,
            item_value: None,
            item_icon: Some("https://oldschool.runescape.wiki/images/Tumeken%27s_shadow_%28uncharged%29_detail.png".to_string()),
            _discord_message: "RuneScape Player received special loot from a raid: Tumeken's shadow (uncharged)".to_string(),
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
            _discord_message:
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
            _discord_message: "RuneScape Player received a drop: Abyssal whip (1,456,814 coins)."
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
            _discord_message: "RuneScape Player received a drop: Unknown Item (0 coins)."
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
            _discord_message: "RuneScape Player received a drop: 587 x Cannonball (111,530 coins)."
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
            _discord_message: "RuneScape Player received a drop: Awakener's orb (2,238,871 coins)."
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
            _discord_message:
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
        possible_pk_broadcasts.push(PkBroadcastTest {
            message: "KANlEL OUTIS has been defeated by Veljenpojat in The Wilderness and lost (953,005 coins) worth of loot.".to_string(),
            pk_broadcast: PkBroadcast {
                winner: "Veljenpojat".to_string(),
                loser: "KANlEL OUTIS".to_string(),
                clan_mate: "KANlEL OUTIS".to_string(),
                gp_exchanged: Some(953_005),
                clan_mate_won: false,
            },
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

    fn get_collection_log_messages() -> Vec<TestBroadcast<CollectionLogBroadcast>> {
        let mut test_collection_messages: Vec<TestBroadcast<CollectionLogBroadcast>> = Vec::new();
        test_collection_messages.push(TestBroadcast {
            message: "KANlEL OUTIS received a new collection log item: Elite void robe (170/1477)"
                .to_string(),
            broadcast: CollectionLogBroadcast {
                player_it_happened_to: "KANlEL OUTIS".to_string(),
                item_name: "Elite void robe".to_string(),
                log_slots: 170,
                item_icon: Some(
                    "https://oldschool.runescape.wiki/images/Elite_void_robe_detail.png"
                        .to_string(),
                ),
            },
        });

        test_collection_messages.push(TestBroadcast {
            message: "S mf received a new collection log item: Charged ice (161/1477)".to_string(),
            broadcast: CollectionLogBroadcast {
                player_it_happened_to: "S mf".to_string(),
                item_name: "Charged ice".to_string(),
                log_slots: 161,
                item_icon: Some(
                    "https://oldschool.runescape.wiki/images/Charged_ice_detail.png".to_string(),
                ),
            },
        });

        test_collection_messages.push(TestBroadcast {
            message:
                "Sad Bug received a new collection log item: Adamant platebody (h1) (895/1477)"
                    .to_string(),
            broadcast: CollectionLogBroadcast {
                player_it_happened_to: "Sad Bug".to_string(),
                item_name: "Adamant platebody (h1)".to_string(),
                log_slots: 895,
                item_icon: Some(
                    "https://oldschool.runescape.wiki/images/Adamant_platebody_(h1)_detail.png"
                        .to_string(),
                ),
            },
        });

        test_collection_messages.push(TestBroadcast {
            message: "rsn received a new collection log item: Enhanced crystal teleport seed (629/1487)".to_string(),
            broadcast: CollectionLogBroadcast {
                player_it_happened_to: "rsn".to_string(),
                item_name: "Enhanced crystal teleport seed".to_string(),
                log_slots: 629,
                item_icon: Some("https://oldschool.runescape.wiki/images/Enhanced_crystal_teleport_seed_detail.png".to_string()),
            },
        });

        test_collection_messages.push(TestBroadcast {
            message: "rsn received a new collection log item: Kraken tentacle (169/1487)"
                .to_string(),
            broadcast: CollectionLogBroadcast {
                player_it_happened_to: "rsn".to_string(),
                item_name: "Kraken tentacle".to_string(),
                log_slots: 169,
                item_icon: Some(
                    "https://oldschool.runescape.wiki/images/Kraken_tentacle_detail.png"
                        .to_string(),
                ),
            },
        });

        test_collection_messages
    }

    fn get_has_left_the_clan_messages() -> Vec<TestBroadcast<String>> {
        let mut test_has_left_the_clan_messages: Vec<TestBroadcast<String>> = Vec::new();
        test_has_left_the_clan_messages.push(TestBroadcast {
            message: "RuneScape Player has left the clan.".to_string(),
            broadcast: "RuneScape Player".to_string(),
        });

        test_has_left_the_clan_messages
    }

    fn get_expelled_from_clan_messages() -> Vec<TestBroadcast<String>> {
        let mut test_expelled_from_clan_messages: Vec<TestBroadcast<String>> = Vec::new();
        test_expelled_from_clan_messages.push(TestBroadcast {
            message: "mod has expelled bob joe from the clan.".to_string(),
            broadcast: "bob joe".to_string(),
        });

        test_expelled_from_clan_messages
    }

    fn get_clan_coffer_deposit_broadcast_messages() -> Vec<TestBroadcast<CofferTransactionBroadcast>>
    {
        let mut test_clan_coffer_broadcast_messages: Vec<
            TestBroadcast<CofferTransactionBroadcast>,
        > = Vec::new();
        test_clan_coffer_broadcast_messages.push(TestBroadcast {
            message: "RuneScape Player has deposited 1,000,000 coins into the coffer.".to_string(),
            broadcast: CofferTransactionBroadcast {
                player: "RuneScape Player".to_string(),
                gp: 1_000_000,
                transaction_type: CofferTransaction::Donation,
            },
        });
        test_clan_coffer_broadcast_messages
    }

    fn get_clan_coffer_withdraw_broadcast_messages(
    ) -> Vec<TestBroadcast<CofferTransactionBroadcast>> {
        let mut test_clan_coffer_broadcast_messages: Vec<
            TestBroadcast<CofferTransactionBroadcast>,
        > = Vec::new();
        test_clan_coffer_broadcast_messages.push(TestBroadcast {
            message: "RuneScape Player has withdrawn 1,000,000 coins from the coffer.".to_string(),
            broadcast: CofferTransactionBroadcast {
                player: "RuneScape Player".to_string(),
                gp: 1_000_000,
                transaction_type: CofferTransaction::Withdrawal,
            },
        });

        test_clan_coffer_broadcast_messages.push(TestBroadcast {
            message: "RuneScape Player has withdrawn 1 coins from the coffer.".to_string(),
            broadcast: CofferTransactionBroadcast {
                player: "RuneScape Player".to_string(),
                gp: 1,
                transaction_type: CofferTransaction::Withdrawal,
            },
        });
        test_clan_coffer_broadcast_messages
    }

    fn get_pbs_broadcast_messages() -> Vec<TestBroadcast<PersonalBestBroadcast>> {
        let mut messages: Vec<TestBroadcast<PersonalBestBroadcast>> = Vec::new();

        messages.push(TestBroadcast {
            message: "RuneScape Player has achieved a new Vorkath personal best: 2:28".to_string(),
            broadcast: PersonalBestBroadcast {
                player: "RuneScape Player".to_string(),
                time_in_seconds: 148.00,
                activity: "Vorkath".to_string(),                
                variant: None,
            },
        });

        messages.push(TestBroadcast {
            message: "RuneScape Player has achieved a new Tempoross personal best: 6:05"
                .to_string(),
            broadcast: PersonalBestBroadcast {
                player: "RuneScape Player".to_string(),
                time_in_seconds: 365.00,
                activity: "Tempoross".to_string(),                
                variant: None,
            },
        });

        //Raids
        messages.push(TestBroadcast {
            message: "RuneScape Player has achieved a new Chambers of Xeric (Team Size: Solo) personal best: 1:09:52".to_string(),
            broadcast: PersonalBestBroadcast {
                player: "RuneScape Player".to_string(),
                time_in_seconds: 4_192.00,
                activity: "Chambers of Xeric".to_string(),                
                variant: Some("Chambers of Xeric Solo".to_string()),
            },
        });

        messages.push(TestBroadcast {
            message: "RuneScape Player has achieved a new Chambers of Xeric Challenge Mode (Team Size: 2 players) personal best: 46:38.40".to_string(),
            broadcast: PersonalBestBroadcast {
                player: "RuneScape Player".to_string(),
                time_in_seconds: 2_798.40,
                activity: "Chambers of Xeric".to_string(),                
                variant: Some("Chambers of Xeric Challenge Mode 2 players".to_string()),
            },
        });

        messages.push(TestBroadcast {
            message: "RuneScape Player has achieved a new Tombs of Amascut (team size: 5) Expert mode Overall personal best: 36:37.80".to_string(),
            broadcast: PersonalBestBroadcast {
                player: "RuneScape Player".to_string(),
                time_in_seconds: 2_197.80,
                activity: "Tombs of Amascut".to_string(),                
                variant: Some("Tombs of Amascut Expert Mode 5 players".to_string()),
            },
        });

        messages.push(TestBroadcast {
            message: "RuneScape Player has achieved a new Tombs of Amascut (team size: 2) Entry mode Overall personal best: 40:35".to_string(),
            broadcast: PersonalBestBroadcast {
                player: "RuneScape Player".to_string(),
                time_in_seconds: 2_435.00,
                activity: "Tombs of Amascut".to_string(),                
                variant: Some("Tombs of Amascut Entry Mode 2 players".to_string()),
            },
            
        });

        messages.push(TestBroadcast {
            message: "RuneScape Player has achieved a new Tombs of Amascut (team size: 1) Normal mode Overall personal best: 30:20".to_string(),
            broadcast: PersonalBestBroadcast {
                player: "RuneScape Player".to_string(),
                time_in_seconds: 1_820.00,
                activity: "Tombs of Amascut".to_string(),                
                variant: Some("Tombs of Amascut Solo".to_string())
            },
        });

        messages.push(TestBroadcast {
            message: "RuneScape Player has achieved a new Theatre of Blood (Team Size: 5) personal best: 17:21".to_string(),
            broadcast: PersonalBestBroadcast {
                player: "RuneScape Player".to_string(),
                time_in_seconds: 1_041.00,
                activity: "Theatre of Blood".to_string(),
                variant: Some("Theatre of Blood 5 players".to_string())
            },
        });

        messages.push(TestBroadcast {
            message: "RuneScape Player has achieved a new Theatre of Blood: Hard mode (Team Size: 3) personal best: 22:42.60".to_string(),
            broadcast: PersonalBestBroadcast {
                player: "RuneScape Player".to_string(),
                time_in_seconds: 1_362.60,
                activity: "Theatre of Blood".to_string(),
                variant: Some("Theatre of Blood Hard Mode 3 players".to_string())
            },
        });

        messages.push(TestBroadcast {
            message: "RuneScape Player has achieved a new Theatre of Blood: Entry mode (Team Size: 3) personal best: 20:28".to_string(),
            broadcast: PersonalBestBroadcast {
                player: "RuneScape Player".to_string(),
                time_in_seconds: 1_228.00,
                activity: "Theatre of Blood".to_string(),
                variant: Some("Theatre of Blood Entry Mode".to_string())
            },
        });

        //Jads
        messages.push(TestBroadcast {
            message: "RuneScape Player has achieved a new TzHaar-Ket-Rak's First Challenge personal best: 1:38".to_string(),
            broadcast: PersonalBestBroadcast {
                player: "RuneScape Player".to_string(),
                time_in_seconds: 98.00,
                activity: "TzHaar-Ket-Rak's First Challenge".to_string(),
                variant: None
            },
        });

        messages
    }

    fn get_leagues_catch_all_broadcast_messages() -> Vec<TestBroadcast<LeaguesBroadCastType>> {
        let mut messages: Vec<TestBroadcast<LeaguesBroadCastType>> = Vec::new();

        messages.push(TestBroadcast {
            message: "RuneScape Player has earned their 6th Combat mastery point!".to_string(),
            broadcast: LeaguesBroadCastType::CombatMasteries,
        });

        messages.push(TestBroadcast {
            message: "RuneScape Player has unlocked their 3rd League area!".to_string(),
            broadcast: LeaguesBroadCastType::AreaUnlock,
        });

        messages.push(TestBroadcast {
            message: "RuneScape Player has unlocked their tier 3 League relic!".to_string(),
            broadcast: LeaguesBroadCastType::RelicTier,
        });

        //TODO add rank as we find more examples

        messages
    }
}
