use crate::database::RegisteredGuild;
use crate::ge_api::ge_api::{get_item_value_by_id, GeItemMapping};
use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::*;
use num_format::{Locale, ToFormattedString};
use tracing::error;

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

pub struct OSRSBroadcastHandler {
    clan_message: ClanMessage,
    item_mapping: Option<GeItemMapping>,
    registered_guild: RegisteredGuild,
}

impl OSRSBroadcastHandler {
    pub fn new(
        clan_message: ClanMessage,
        item_mapping_from_state: Result<GeItemMapping, ()>,

        register_guild: RegisteredGuild,
    ) -> Self {
        Self {
            clan_message,
            item_mapping: match item_mapping_from_state {
                Ok(item_mapping) => Some(item_mapping),
                Err(_) => None,
            },
            registered_guild: register_guild,
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

                        Some(BroadcastMessageToDiscord {
                            player_it_happened_to: drop_item.player_it_happened_to.clone(),
                            type_of_broadcast: BroadcastType::RaidDrop,
                            // message: format!("{} received special loot from a raid: {}.", drop_item.player_it_happened_to, drop_item.item_name),
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
            BroadcastType::ItemDrop => self.drop_item_handler(),
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
                    Some(pet_drop) => Some(BroadcastMessageToDiscord {
                        type_of_broadcast: BroadcastType::PetDrop,
                        player_it_happened_to: pet_drop.player_it_happened_to,
                        message: self.clan_message.message.clone(),
                        icon_url: pet_drop.pet_icon,
                        title: ":tada: New Pet drop!".to_string(),
                        item_quantity: None,
                    }),
                }
            }
            BroadcastType::Diary => {
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
                    Some(exported_data) => Some(BroadcastMessageToDiscord {
                        type_of_broadcast: BroadcastType::Diary,
                        player_it_happened_to: exported_data.player_it_happened_to,
                        message: self.clan_message.message.clone(),
                        icon_url: Some(
                            "https://oldschool.runescape.wiki/images/Achievement_Diaries.png"
                                .to_string(),
                        ),
                        title: ":tada: New diary completed!".to_string(),
                        item_quantity: None,
                    }),
                }
            }
            BroadcastType::Quest => {
                let quest_completed =
                    quest_completed_broadcast_extractor(self.clan_message.message.clone());
                match quest_completed {
                    None => {
                        error!(
                            "Failed to extract Quest info from message: {}",
                            self.clan_message.message.clone()
                        );
                        None
                    }
                    Some(exported_data) => Some(BroadcastMessageToDiscord {
                        type_of_broadcast: BroadcastType::Quest,
                        player_it_happened_to: exported_data.player_it_happened_to,
                        message: self.clan_message.message.clone(),
                        icon_url: exported_data.quest_reward_scroll_icon,
                        title: ":tada: New quest completed!".to_string(),
                        item_quantity: None,
                    }),
                }
            }
            BroadcastType::Pk => {
                let possible_pk_broadcast =
                    pk_broadcast_extractor(self.clan_message.message.clone());
                match possible_pk_broadcast {
                    None => {
                        error!(
                            "Failed to extract pk info from message: {}",
                            self.clan_message.message.clone()
                        );
                        None
                    }
                    Some(pk_broadcast) => Some(BroadcastMessageToDiscord {
                        type_of_broadcast: BroadcastType::Pk,
                        player_it_happened_to: pk_broadcast.winner,
                        message: self.clan_message.message.clone(),
                        icon_url: Some(
                            "https://oldschool.runescape.wiki/images/Skull.png".to_string(),
                        ),
                        title: ":crossed_swords: New PK!".to_string(),
                        item_quantity: None,
                    }),
                }
            }
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
                    Some(invite_broadcast) => Some(BroadcastMessageToDiscord {
                        type_of_broadcast: BroadcastType::Invite,
                        player_it_happened_to: invite_broadcast.clan_mate,
                        message: self.clan_message.message.clone(),
                        icon_url: Some(
                            "https://oldschool.runescape.wiki/images/Your_Clan_icon.png"
                                .to_string(),
                        ),
                        title: ":wave: New Invite!".to_string(),
                        item_quantity: None,
                    }),
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
                    Some(levelmilestone_broadcast) => Some(BroadcastMessageToDiscord {
                        type_of_broadcast: BroadcastType::LevelMilestone,
                        player_it_happened_to: levelmilestone_broadcast.clan_mate,
                        message: self.clan_message.message.clone(),
                        icon_url: levelmilestone_broadcast.skill_icon,
                        title: ":tada: New Level Milestone reached!".to_string(),
                        item_quantity: None,
                    }),
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
                    Some(xpmilestone_broadcast) => Some(BroadcastMessageToDiscord {
                        type_of_broadcast: BroadcastType::XPMilestone,
                        player_it_happened_to: xpmilestone_broadcast.clan_mate,
                        message: self.clan_message.message.clone(),
                        icon_url: xpmilestone_broadcast.skill_icon,
                        title: ":tada: New XP Milestone reached!".to_string(),
                        item_quantity: None,
                    }),
                }
            }
            _ => None,
        }
    }

    fn drop_item_handler(&self) -> Option<BroadcastMessageToDiscord> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ge_api::ge_api::GetItem;
    use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::ClanMessage;
    use crate::osrs_broadcast_extractor::*;
    use log::info;
    use std::result;

    #[test]
    fn test_drop_item_handler_no_message_sent() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player: bob received a drop: Abyssal whip (1,456,814 coins)."
                .to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
        };

        let mut registered_guild = RegisteredGuild::new(123);
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

        let handler = OSRSBroadcastHandler::new(clan_message, get_item_mapping, registered_guild);

        let extracted_message = handler.drop_item_handler();
        info!("Extracted message: {:?}", extracted_message);
        match extracted_message {
            None => {}
            Some(result) => {
                info!("Threshold should of not been hit. Do not pass go. Do not collect 200.");
                assert_eq!(true, false);
            }
        }
    }

    #[test]
    fn test_drop_item_handler_message_sent() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player: bob received a drop: Cool Item (20,456,814 coins)."
                .to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
        };

        let mut registered_guild = RegisteredGuild::new(123);
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

        let handler = OSRSBroadcastHandler::new(clan_message, get_item_mapping, registered_guild);

        let extracted_message = handler.drop_item_handler();
        info!("Extracted message: {:?}", extracted_message);
        match extracted_message {
            None => {
                info!("Threshold should of been hit. Should not be sending a message.");
                assert_eq!(true, false);
            }
            Some(_) => {}
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
