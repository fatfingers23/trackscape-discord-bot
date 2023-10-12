use log::error;
use num_format::{Locale, ToFormattedString};
use trackscape_discord_shared::database::{DropLogs, RegisteredGuildModel};
use trackscape_discord_shared::ge_api::ge_api::{get_item_value_by_id, GeItemMapping};
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::{
    diary_completed_broadcast_extractor, drop_broadcast_extractor, get_broadcast_type,
    invite_broadcast_extractor, levelmilestone_broadcast_extractor, pet_broadcast_extractor,
    pk_broadcast_extractor, quest_completed_broadcast_extractor, raid_broadcast_extractor,
    xpmilestone_broadcast_extractor, BroadcastType, ClanMessage,
};
use trackscape_discord_shared::wiki_api::wiki_api::WikiQuest;

#[derive(Debug, Clone)]
pub struct BroadcastMessageToDiscord {
    pub player_it_happened_to: String,
    pub type_of_broadcast: BroadcastType,
    pub message: String,
    pub icon_url: Option<String>,
    pub title: String,
    // Since this is not for just drops any more can be anything
    //GP, xp, kc, etc
    pub item_quantity: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct OSRSBroadcastHandler<T: DropLogs> {
    clan_message: ClanMessage,
    item_mapping: Option<GeItemMapping>,
    quests: Option<Vec<WikiQuest>>,
    registered_guild: RegisteredGuildModel,
    db: T,
}

impl<T: DropLogs> OSRSBroadcastHandler<T> {
    pub fn new(
        clan_message: ClanMessage,
        item_mapping_from_state: Result<GeItemMapping, ()>,
        quests_from_state: Result<Vec<WikiQuest>, ()>,
        register_guild: RegisteredGuildModel,
        db: T,
    ) -> Self {
        Self {
            clan_message,
            item_mapping: match item_mapping_from_state {
                Ok(item_mapping) => Some(item_mapping),
                Err(_) => None,
            },
            quests: match quests_from_state {
                Ok(quests) => Some(quests),
                Err(_) => None,
            },
            registered_guild: register_guild,
            db: db,
        }
    }

    pub async fn extract_message(&self) -> Option<BroadcastMessageToDiscord> {
        let broadcast_type = get_broadcast_type(self.clan_message.message.clone());
        match broadcast_type {
            BroadcastType::RaidDrop => {
                let drop_item = raid_broadcast_extractor(self.clan_message.message.clone());
                match drop_item {
                    None => {
                        error!(
                            "Failed to extract drop item from message: {}",
                            self.clan_message.message.clone()
                        );
                        None
                    }
                    Some(mut drop_item) => {
                        match &self.item_mapping {
                            Some(item_mapping) => {
                                for item in item_mapping {
                                    if item.name == drop_item.item_name {
                                        let price_check = get_item_value_by_id(item.id).await;
                                        match price_check {
                                            Ok(price) => {
                                                if price.high > 0 {
                                                    drop_item.item_value = Some(price.high);
                                                }
                                            }
                                            Err(_) => {}
                                        }
                                    }
                                }
                            }
                            None => {}
                        }
                        self.db
                            .new_drop_log(drop_item.clone(), self.registered_guild.guild_id)
                            .await;
                        let is_disallowed = self
                            .registered_guild
                            .disallowed_broadcast_types
                            .iter()
                            .find(|&x| {
                                if let BroadcastType::RaidDrop = x {
                                    return true;
                                }
                                false
                            });
                        if is_disallowed.is_some() {
                            return None;
                        }

                        Some(BroadcastMessageToDiscord {
                            player_it_happened_to: drop_item.player_it_happened_to.clone(),
                            type_of_broadcast: BroadcastType::RaidDrop,
                            message: match drop_item.item_value {
                                None => {
                                    format!(
                                        "{} received special loot from a raid: {}.",
                                        drop_item.player_it_happened_to, drop_item.item_name
                                    )
                                }
                                Some(item_value) => {
                                    format!(
                                        "{} received special loot from a raid: {} ({} coins).",
                                        drop_item.player_it_happened_to,
                                        drop_item.item_name,
                                        item_value.to_formatted_string(&Locale::en)
                                    )
                                }
                            },
                            icon_url: drop_item.item_icon,
                            title: ":tada: New raid drop!".to_string(),
                            item_quantity: None,
                        })
                    }
                }
            }
            BroadcastType::ItemDrop => self.drop_item_handler().await,
            BroadcastType::PetDrop => {
                let pet_drop_item = pet_broadcast_extractor(self.clan_message.message.clone());
                match pet_drop_item {
                    None => {
                        error!(
                            "Failed to extract pet drop from message: {}",
                            self.clan_message.message.clone()
                        );
                        None
                    }
                    Some(pet_drop) => {
                        let is_disallowed = self
                            .registered_guild
                            .disallowed_broadcast_types
                            .iter()
                            .find(|&x| {
                                if let BroadcastType::Quest = x {
                                    return true;
                                }
                                false
                            });
                        if is_disallowed.is_some() {
                            return None;
                        }

                        Some(BroadcastMessageToDiscord {
                            type_of_broadcast: BroadcastType::PetDrop,
                            player_it_happened_to: pet_drop.player_it_happened_to,
                            message: self.clan_message.message.clone(),
                            icon_url: pet_drop.pet_icon,
                            title: ":tada: New Pet drop!".to_string(),
                            item_quantity: None,
                        })
                    }
                }
            }
            BroadcastType::Diary => self.diary_handler(),
            BroadcastType::Quest => self.quest_handler(),
            BroadcastType::Pk => self.pk_handler(),
            BroadcastType::Invite => {
                let possible_invite_broadcast =
                    invite_broadcast_extractor(self.clan_message.message.clone());
                match possible_invite_broadcast {
                    None => {
                        error!(
                            "Failed to extract invite info from message: {}",
                            self.clan_message.message.clone()
                        );
                        None
                    }
                    Some(invite_broadcast) => {
                        let is_disallowed = self
                            .registered_guild
                            .disallowed_broadcast_types
                            .iter()
                            .find(|&x| {
                                if let BroadcastType::Invite = x {
                                    return true;
                                }
                                false
                            });
                        if is_disallowed.is_some() {
                            return None;
                        }
                        Some(BroadcastMessageToDiscord {
                            type_of_broadcast: BroadcastType::Invite,
                            player_it_happened_to: invite_broadcast.clan_mate,
                            message: self.clan_message.message.clone(),
                            icon_url: Some(
                                "https://oldschool.runescape.wiki/images/Your_Clan_icon.png"
                                    .to_string(),
                            ),
                            title: ":wave: New Invite!".to_string(),
                            item_quantity: None,
                        })
                    }
                }
            }
            BroadcastType::LevelMilestone => {
                let possible_levelmilestone_broadcast =
                    levelmilestone_broadcast_extractor(self.clan_message.message.clone());
                match possible_levelmilestone_broadcast {
                    None => {
                        error!(
                            "Failed to extract level milestone info from message: {}",
                            self.clan_message.message.clone()
                        );
                        None
                    }
                    Some(levelmilestone_broadcast) => {
                        let is_disallowed = self
                            .registered_guild
                            .disallowed_broadcast_types
                            .iter()
                            .find(|&x| {
                                if let BroadcastType::LevelMilestone = x {
                                    return true;
                                }
                                false
                            });
                        if is_disallowed.is_some() {
                            return None;
                        }
                        Some(BroadcastMessageToDiscord {
                            type_of_broadcast: BroadcastType::LevelMilestone,
                            player_it_happened_to: levelmilestone_broadcast.clan_mate,
                            message: self.clan_message.message.clone(),
                            icon_url: levelmilestone_broadcast.skill_icon,
                            title: ":tada: New Level Milestone reached!".to_string(),
                            item_quantity: None,
                        })
                    }
                }
            }
            BroadcastType::XPMilestone => {
                let possible_xpmilestone_broadcast =
                    xpmilestone_broadcast_extractor(self.clan_message.message.clone());
                match possible_xpmilestone_broadcast {
                    None => {
                        error!(
                            "Failed to extract xp milestone info from message: {}",
                            self.clan_message.message.clone()
                        );
                        None
                    }
                    Some(xpmilestone_broadcast) => {
                        let is_disallowed = self
                            .registered_guild
                            .disallowed_broadcast_types
                            .iter()
                            .find(|&x| {
                                if let BroadcastType::XPMilestone = x {
                                    return true;
                                }
                                false
                            });
                        if is_disallowed.is_some() {
                            return None;
                        }
                        Some(BroadcastMessageToDiscord {
                            type_of_broadcast: BroadcastType::XPMilestone,
                            player_it_happened_to: xpmilestone_broadcast.clan_mate,
                            message: self.clan_message.message.clone(),
                            icon_url: xpmilestone_broadcast.skill_icon,
                            title: ":tada: New XP Milestone reached!".to_string(),
                            item_quantity: None,
                        })
                    }
                }
            }
            _ => None,
        }
    }

    async fn drop_item_handler(&self) -> Option<BroadcastMessageToDiscord> {
        let drop_item = drop_broadcast_extractor(self.clan_message.message.clone());

        match drop_item {
            None => {
                error!(
                    "Failed to extract drop item from message: {}",
                    self.clan_message.message.clone()
                );
                None
            }
            Some(drop_item) => {
                self.db
                    .new_drop_log(drop_item.clone(), self.registered_guild.guild_id)
                    .await;
                let is_disallowed = self
                    .registered_guild
                    .disallowed_broadcast_types
                    .iter()
                    .find(|&x| {
                        if let BroadcastType::ItemDrop = x {
                            return true;
                        }
                        false
                    });
                if is_disallowed.is_some() {
                    return None;
                }
                if self.registered_guild.drop_price_threshold.is_some() {
                    if drop_item.item_value.is_some() {
                        if self.registered_guild.drop_price_threshold.unwrap()
                            > drop_item.item_value.unwrap()
                        {
                            return None;
                        }
                    }
                }

                Some(BroadcastMessageToDiscord {
                    player_it_happened_to: drop_item.player_it_happened_to.clone(),
                    type_of_broadcast: BroadcastType::ItemDrop,
                    title: ":tada: New High Value drop!".to_string(),
                    message: match drop_item.item_quantity {
                        //If there is only one of the items dropped
                        1 => match drop_item.item_value {
                            //If the item has a value with it
                            None => format!(
                                "{} received a drop: {}.",
                                drop_item.player_it_happened_to, drop_item.item_name
                            ),
                            _ => format!(
                                "{} received a drop: {} ({} coins).",
                                drop_item.player_it_happened_to,
                                drop_item.item_name,
                                drop_item
                                    .item_value
                                    .unwrap()
                                    .to_formatted_string(&Locale::en)
                            ),
                        },
                        _ => match drop_item.item_value {
                            //If the item has a value with it
                            None => format!(
                                "{} received a drop: {} x {}",
                                drop_item.player_it_happened_to,
                                drop_item.item_name,
                                drop_item.item_quantity
                            ),
                            _ => format!(
                                "{} received a drop: {} x {} ({} coins).",
                                drop_item.player_it_happened_to,
                                drop_item.item_quantity,
                                drop_item.item_name,
                                drop_item
                                    .item_value
                                    .unwrap()
                                    .to_formatted_string(&Locale::en)
                            ),
                        },
                    },
                    icon_url: drop_item.item_icon,
                    item_quantity: drop_item.item_value,
                })
            }
        }
    }
    fn pk_handler(&self) -> Option<BroadcastMessageToDiscord> {
        let possible_pk_broadcast = pk_broadcast_extractor(self.clan_message.message.clone());
        match possible_pk_broadcast {
            None => {
                error!(
                    "Failed to extract pk info from message: {}",
                    self.clan_message.message.clone()
                );
                None
            }
            Some(pk_broadcast) => {
                if self.registered_guild.pk_value_threshold.is_some() {
                    if pk_broadcast.gp_exchanged.is_some() {
                        if self.registered_guild.pk_value_threshold.unwrap()
                            > pk_broadcast.gp_exchanged.unwrap()
                        {
                            return None;
                        }
                    }
                }

                Some(BroadcastMessageToDiscord {
                    type_of_broadcast: BroadcastType::Pk,
                    player_it_happened_to: pk_broadcast.winner,
                    message: self.clan_message.message.clone(),
                    icon_url: Some("https://oldschool.runescape.wiki/images/Skull.png".to_string()),
                    title: ":crossed_swords: New PK!".to_string(),
                    item_quantity: None,
                })
            }
        }
    }

    fn quest_handler(&self) -> Option<BroadcastMessageToDiscord> {
        let quest_completed =
            quest_completed_broadcast_extractor(self.clan_message.message.clone());

        let possible_quests = self.quests.clone();

        match quest_completed {
            None => {
                error!(
                    "Failed to extract Quest info from message: {}",
                    self.clan_message.message.clone()
                );
                None
            }

            Some(exported_data) => {
                let is_disallowed = self
                    .registered_guild
                    .disallowed_broadcast_types
                    .iter()
                    .find(|&x| {
                        if let BroadcastType::Quest = x {
                            return true;
                        }
                        false
                    });
                if is_disallowed.is_some() {
                    return None;
                }

                if self.registered_guild.min_quest_difficulty.is_some()
                    && possible_quests.is_some()
                    && self.quests.is_some()
                {
                    let quest_name = exported_data.quest_name;

                    let quests = &possible_quests.unwrap();
                    let possible_difficulty = quests.iter().find(|&x| x.name == quest_name);
                    if possible_difficulty.is_none() {
                        return None;
                    }
                    let min_quest_difficulty =
                        self.registered_guild.clone().min_quest_difficulty.unwrap();
                    if possible_difficulty.unwrap().difficulty.ranking()
                        < min_quest_difficulty.ranking()
                    {
                        return None;
                    }
                }
                Some(BroadcastMessageToDiscord {
                    type_of_broadcast: BroadcastType::Quest,
                    player_it_happened_to: exported_data.player_it_happened_to,
                    message: self.clan_message.message.clone(),
                    icon_url: exported_data.quest_reward_scroll_icon,
                    title: ":tada: New quest completed!".to_string(),
                    item_quantity: None,
                })
            }
        }
    }

    fn diary_handler(&self) -> Option<BroadcastMessageToDiscord> {
        let diary_completed =
            diary_completed_broadcast_extractor(self.clan_message.message.clone());
        match diary_completed {
            None => {
                error!(
                    "Failed to extract Diary info from message: {}",
                    self.clan_message.message.clone()
                );
                None
            }
            Some(exported_data) => {
                let is_disallowed = self
                    .registered_guild
                    .disallowed_broadcast_types
                    .iter()
                    .find(|&x| {
                        if let BroadcastType::Diary = x {
                            return true;
                        }
                        false
                    });
                if is_disallowed.is_some() {
                    return None;
                }
                if self.registered_guild.min_diary_tier.is_some() {
                    let min_diary_tier = self.registered_guild.clone().min_diary_tier.unwrap();
                    if exported_data.diary_tier.ranking() < min_diary_tier.ranking() {
                        return None;
                    }
                }
                Some(BroadcastMessageToDiscord {
                    type_of_broadcast: BroadcastType::Diary,
                    player_it_happened_to: exported_data.player_it_happened_to,
                    message: self.clan_message.message.clone(),
                    icon_url: Some(
                        "https://oldschool.runescape.wiki/images/Achievement_Diaries.png"
                            .to_string(),
                    ),
                    title: ":tada: New diary completed!".to_string(),
                    item_quantity: None,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::info;
    use trackscape_discord_shared::database::MockDropLogs;
    use trackscape_discord_shared::ge_api::ge_api::GetItem;
    use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::{
        DiaryTier, QuestDifficulty,
    };

    #[tokio::test]
    async fn test_drop_item_handler_no_message_sent() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player: bob received a drop: Abyssal whip (1,456,814 coins)."
                .to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
        };

        let mut registered_guild = RegisteredGuildModel::new(123);
        registered_guild.drop_price_threshold = Some(20_000_000);
        let ge_item_mapping: Vec<GetItem> = Vec::new();
        let get_item_mapping = Ok(ge_item_mapping);

        //Saintly checker do not know how to do mock in rust yet. So this makes sure the above message
        //Is valid to trip the extractor and give the expect result
        let sanity_check = drop_broadcast_extractor(clan_message.message.clone());
        match sanity_check {
            None => {
                info!("Sanity check failed. The message is not valid or the extractor is broken and that unit test should also be failing");
                assert_eq!(true, false);
            }
            Some(_) => {}
        }

        let quests = Ok(Vec::new());

        let mut db_mock = MockDropLogs::new();
        db_mock.expect_new_drop_log().returning(|_, _| {
            info!("Should not be calling this function");
        });
        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            db_mock,
        );

        let extracted_message = handler.drop_item_handler().await;
        info!("Extracted message: {:?}", extracted_message);
        match extracted_message {
            None => {
                assert_eq!(true, true);
            }
            Some(result) => {
                info!("Threshold should of been hit. Should not be sending a message.");
                assert_eq!(true, false);
            }
        }
    }

    #[tokio::test]
    async fn test_drop_item_handler_message_sent() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player: bob received a drop: Cool Item (20,456,814 coins)."
                .to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
        };

        let mut registered_guild = RegisteredGuildModel::new(123);
        registered_guild.drop_price_threshold = Some(20_000_000);
        let ge_item_mapping: Vec<GetItem> = Vec::new();
        let get_item_mapping = Ok(ge_item_mapping);

        //Saintly checker do not know how to do mock in rust yet. So this makes sure the above message
        //Is valid to trip the extractor and give the expect result
        let sanity_check = drop_broadcast_extractor(clan_message.message.clone());
        match sanity_check {
            None => {
                info!("Sanity check failed. The message is not valid or the extractor is broken and that unit test should also be failing");
                assert_eq!(true, false);
            }
            Some(_) => {}
        }
        let quests = Ok(Vec::new());

        let mut db_mock = MockDropLogs::new();
        db_mock.expect_new_drop_log().returning(|_, _| {
            info!("Should not be calling this function");
        });

        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            db_mock,
        );

        let extracted_message = handler.drop_item_handler().await;
        info!("Extracted message: {:?}", extracted_message);
        match extracted_message {
            None => {
                info!("Threshold should of not been hit. Should be sending a message.");
                assert_eq!(true, false);
            }
            Some(_) => {
                assert_eq!(true, true);
            }
        }
    }

    #[test]
    fn test_quest_handler_message_sent() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player has completed a quest: The Fremennik Isles".to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
        };

        let mut registered_guild = RegisteredGuildModel::new(123);
        registered_guild.min_quest_difficulty = Some(QuestDifficulty::Intermediate);
        let ge_item_mapping: Vec<GetItem> = Vec::new();
        let get_item_mapping = Ok(ge_item_mapping);

        let quests = Ok(vec![WikiQuest {
            name: "The Fremennik Isles".to_string(),
            difficulty: QuestDifficulty::Intermediate,
        }]);
        //Saintly checker do not know how to do mock in rust yet. So this makes sure the above message
        //Is valid to trip the extractor and give the expect result
        let sanity_check = quest_completed_broadcast_extractor(clan_message.message.clone());
        match sanity_check {
            None => {
                info!("Sanity check failed. The message is not valid or the extractor is broken and that unit test should also be failing");
                assert_eq!(true, false);
            }
            Some(_) => {
                assert_eq!(true, true);
            }
        }

        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            MockDropLogs::new(),
        );

        let extracted_message = handler.quest_handler();
        info!("Extracted message: {:?}", extracted_message);
        match extracted_message {
            None => {
                info!("Threshold should of not been hit. Should  be sending a message.");
                assert_eq!(true, false);
            }
            Some(_) => {
                assert_eq!(true, true);
            }
        }
    }

    #[test]
    fn test_quest_handler_message_not_sent() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player has completed a quest: Cook's Assistant".to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
        };

        let mut registered_guild = RegisteredGuildModel::new(123);
        registered_guild.min_quest_difficulty = Some(QuestDifficulty::Master);
        let ge_item_mapping: Vec<GetItem> = Vec::new();
        let get_item_mapping = Ok(ge_item_mapping);

        let quests = Ok(vec![WikiQuest {
            name: "Cook's Assistant".to_string(),
            difficulty: QuestDifficulty::Novice,
        }]);
        //Saintly checker do not know how to do mock in rust yet. So this makes sure the above message
        //Is valid to trip the extractor and give the expect result
        let sanity_check = quest_completed_broadcast_extractor(clan_message.message.clone());
        match sanity_check {
            None => {
                info!("Sanity check failed. The message is not valid or the extractor is broken and that unit test should also be failing");
                assert_eq!(true, false);
            }
            Some(_) => {
                assert_eq!(true, true);
            }
        }

        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            MockDropLogs::new(),
        );

        let extracted_message = handler.quest_handler();
        info!("Extracted message: {:?}", extracted_message);
        match extracted_message {
            None => {
                assert_eq!(true, true);
            }
            Some(_) => {
                info!("Threshold should of been hit. Should not be sending a message.");
                assert_eq!(true, false);
            }
        }
    }

    #[test]
    fn test_pk_value_handler_low() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "HolidayPanda has been defeated by next trial in The Wilderness and lost (33,601 coins) worth of loot."
                .to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
        };

        let mut registered_guild = RegisteredGuildModel::new(123);
        registered_guild.pk_value_threshold = Some(1_000_000);
        let ge_item_mapping: Vec<GetItem> = Vec::new();
        let get_item_mapping = Ok(ge_item_mapping);

        //Saintly checker do not know how to do mock in rust yet. So this makes sure the above message
        //Is valid to trip the extractor and give the expect result
        let sanity_check = pk_broadcast_extractor(clan_message.message.clone());
        match sanity_check {
            None => {
                println!("Sanity check failed. The message is not valid or the extractor is broken and that unit test should also be failing");
                assert_eq!(true, false);
            }
            Some(_) => {
                println!("Sanity check success");
            }
        }
        let quests = Ok(Vec::new());

        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            MockDropLogs::new(),
        );

        let extracted_message = handler.pk_handler();
        println!("Extracted message: {:?}", extracted_message);

        match extracted_message {
            None => {
                println!("Successfully stopped message from sending");
            }
            Some(result) => {
                println!("Should not be sending a message.");
                assert_eq!(true, false);
            }
        }
    }

    #[test]
    fn test_diary_handler_message_sent() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player has completed the Hard Ardougne diary.".to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
        };

        let mut registered_guild = RegisteredGuildModel::new(123);
        registered_guild.min_diary_tier = Some(DiaryTier::Hard);
        let ge_item_mapping: Vec<GetItem> = Vec::new();
        let get_item_mapping = Ok(ge_item_mapping);

        let quests = Ok(Vec::new());
        //Saintly checker do not know how to do mock in rust yet. So this makes sure the above message
        //Is valid to trip the extractor and give the expect result
        let sanity_check = diary_completed_broadcast_extractor(clan_message.message.clone());
        match sanity_check {
            None => {
                info!("Sanity check failed. The message is not valid or the extractor is broken and that unit test should also be failing");
                assert_eq!(true, false);
            }
            Some(_) => {
                assert_eq!(true, true);
            }
        }

        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            MockDropLogs::new(),
        );

        let extracted_message = handler.diary_handler();
        info!("Extracted message: {:?}", extracted_message);
        match extracted_message {
            None => {
                info!("Threshold should of not been hit. Should  be sending a message.");
                assert_eq!(true, false);
            }
            Some(_) => {
                assert_eq!(true, true);
            }
        }
    }

    #[test]
    fn test_diary_handler_message_not_sent() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player has completed the Easy Ardougne diary.".to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
        };

        let mut registered_guild = RegisteredGuildModel::new(123);
        registered_guild.min_diary_tier = Some(DiaryTier::Hard);
        let ge_item_mapping: Vec<GetItem> = Vec::new();
        let get_item_mapping = Ok(ge_item_mapping);

        let quests = Ok(Vec::new());
        //Saintly checker do not know how to do mock in rust yet. So this makes sure the above message
        //Is valid to trip the extractor and give the expect result
        let sanity_check = diary_completed_broadcast_extractor(clan_message.message.clone());
        match sanity_check {
            None => {
                info!("Sanity check failed. The message is not valid or the extractor is broken and that unit test should also be failing");
                assert_eq!(true, false);
            }
            Some(_) => {
                assert_eq!(true, true);
            }
        }

        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            MockDropLogs::new(),
        );

        let extracted_message = handler.diary_handler();
        info!("Extracted message: {:?}", extracted_message);
        match extracted_message {
            None => {
                assert_eq!(true, true);
            }
            Some(_) => {
                info!("Threshold should of been hit. Should not be sending a message.");
                assert_eq!(true, false);
            }
        }
    }

    // #[tokio::test]
    // async fn test_extract_message_drop_single_item() {
    //     let possible_drop_broadcasts = get_drop_messages();
    //     for possible_drop_broadcast in possible_drop_broadcasts {
    //         let message = possible_drop_broadcast.message.clone();
    //         let ge_item_mapping: Vec<GetItem> = Vec::new();
    //         let get_item_mapping = Ok(ge_item_mapping);
    //         let extracted_message = osrs_broadcast_extractor::extract_message(
    //             ClanMessage {
    //                 sender: "Insomniacs".to_string(),
    //                 message: possible_drop_broadcast.message.clone(),
    //                 clan_name: "Insomniacs".to_string(),
    //                 rank: "Recruit".to_string(),
    //                 icon_id: None,
    //             },
    //             get_item_mapping,
    //         )
    //         .await;
    //         match extracted_message {
    //             None => {
    //                 info!("Failed to extract drop item from message: {}", message);
    //                 assert!(false);
    //             }
    //             Some(extracted_message) => {
    //                 assert_eq!(
    //                     extracted_message.player_it_happened_to,
    //                     possible_drop_broadcast.player_it_happened_to
    //                 );
    //                 assert_eq!(
    //                     extracted_message.message,
    //                     possible_drop_broadcast.discord_message
    //                 );
    //                 assert_eq!(
    //                     extracted_message.icon_url,
    //                     possible_drop_broadcast.item_icon
    //                 );
    //             }
    //         }
    //     }
    // }
}