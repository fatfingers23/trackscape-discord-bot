use num_format::{Locale, ToFormattedString};


pub mod osrs_broadcast_extractor {
    use std::collections::HashMap;
    use num_format::{Locale, ToFormattedString};

    pub struct ClanMessage {
        pub author: String,
        pub message: String,
    }

    pub struct BroadcastMessageToDiscord {
        pub player_it_happened_to: String,
        pub type_of_broadcast: BroadcastType,
        pub message: String,
        pub icon_url: Option<String>,
        pub title: String,
    }

    pub struct DropItem {
        pub player_it_happened_to: String,
        pub item_name: String,
        pub item_quantity: i64,
        pub item_value: Option<i64>,
        pub item_icon: Option<String>,
    }

    #[derive(PartialEq)]
    pub enum BroadcastType {
        ItemDrop,
        PetDrop,
        Quest,
        Pk,
        RaidDrop,
        Unknown,
    }

    pub fn extract_message(message: ClanMessage) -> Option<BroadcastMessageToDiscord> {
        let broadcast_type = get_broadcast_type(message.message.clone());
        match broadcast_type {
            BroadcastType::RaidDrop => {
                let drop_item = raid_broadcast_extractor(message.message.clone());
                match drop_item {
                    None => {
                        println!("Failed to extract drop item from message: {}", message.message.clone());
                        None
                    }
                    Some(drop_item) => {
                        Some(BroadcastMessageToDiscord {
                            player_it_happened_to: drop_item.player_it_happened_to.clone(),
                            type_of_broadcast: BroadcastType::RaidDrop,
                            message: format!("{} received special loot from a raid: {}.", drop_item.player_it_happened_to, drop_item.item_name),
                            icon_url: drop_item.item_icon,
                            title: ":tada: New raid drop!".to_string(),
                        })
                    }
                }
            }
            BroadcastType::ItemDrop => {
                let drop_item = drop_broadcast_extractor(message.message.clone());
                match drop_item {
                    None => {
                        println!("Failed to extract drop item from message: {}", message.message.clone());
                        None
                    }
                    Some(drop_item) => {
                        Some(BroadcastMessageToDiscord {
                            player_it_happened_to: drop_item.player_it_happened_to.clone(),
                            type_of_broadcast: BroadcastType::ItemDrop,
                            title: ":tada: New High Value drop!".to_string(),
                            message: match drop_item.item_quantity {
                                //If there is only one of the items dropped
                                1 => match drop_item.item_value {
                                    //If the item has a value with it
                                    None => format!("{} received a drop: {}.",
                                                    drop_item.player_it_happened_to,
                                                    drop_item.item_name),
                                    _ => format!("{} received a drop: {} ({} coins).",
                                                 drop_item.player_it_happened_to,
                                                 drop_item.item_name,
                                                 drop_item.item_value.unwrap().to_formatted_string(&Locale::en)),
                                }
                                _ => match drop_item.item_value {
                                    //If the item has a value with it
                                    None => format!("{} received a drop: {} x {}", drop_item.player_it_happened_to, drop_item.item_name, drop_item.item_quantity),
                                    _ => format!("{} received a drop: {} x {} ({} coins).",
                                                 drop_item.player_it_happened_to,
                                                 drop_item.item_quantity,
                                                 drop_item.item_name,
                                                 drop_item.item_value.unwrap().to_formatted_string(&Locale::en)),
                                }
                            },
                            icon_url: drop_item.item_icon,
                        })
                    }
                }
            }
            BroadcastType::PetDrop => {
                Some(BroadcastMessageToDiscord {
                    type_of_broadcast: BroadcastType::PetDrop,
                    player_it_happened_to: "UNKNOW".to_string(),
                    message: message.message.clone(),
                    icon_url: None,
                    title: ":tada: New Pet drop!".to_string(),
                })
            }
            _ => None
        }
    }

    pub fn raid_broadcast_extractor(message: String) -> Option<DropItem> {
        let re = regex::Regex::new(r#"^(?P<player_name>.*?) received special loot from a raid: (?P<item>.*?)([.]|$)"#).unwrap();

        return if let Some(caps) = re.captures(message.as_str()) {
            let player_name = caps.name("player_name").unwrap().as_str();
            let item = caps.name("item").unwrap().as_str();

            Some(DropItem {
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

    pub fn drop_broadcast_extractor(message: String) -> Option<DropItem> {
        let re = regex::Regex::new(r#"^(?P<player_name>.*?) received a drop: (?:((?P<quantity>[,\d]+) x )?)(?P<item>.*?)(?: \((?P<value>[,\d]+) coins\))?[.]?$"#).unwrap();

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


            Some(DropItem {
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

    pub fn get_broadcast_type(message_content: String) -> BroadcastType {
        if message_content.contains("received a drop:") {
            return BroadcastType::ItemDrop;
        }
        if message_content.contains("received special loot from a raid:") {
            return BroadcastType::RaidDrop;
        }
        if message_content.contains("has a funny feeling") && message_content.contains("followed:") {
            return BroadcastType::PetDrop;
        }
        return BroadcastType::Unknown;
    }

    fn get_wiki_image_url(item_name: String) -> String {
        let replace_spaces = item_name.replace(" ", "_");
        let encoded_item_name = urlencoding::encode(replace_spaces.as_str());
        format!("https://oldschool.runescape.wiki/images/{}.png", encoded_item_name)
    }
}


#[cfg(test)]
mod tests {
    use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::{ClanMessage, DropItem};
    use super::*;

    #[test]
    fn test_get_drop_type_broadcast() {
        let possible_drop_broadcasts = get_drop_messages();
        for possible_drop_broadcast in possible_drop_broadcasts {
            let broadcast_type = osrs_broadcast_extractor::get_broadcast_type(possible_drop_broadcast.message);
            assert!(matches!(broadcast_type, osrs_broadcast_extractor::BroadcastType::ItemDrop));
        }
    }

    #[test]
    fn test_get_raid_drop_type_broadcast() {
        let possible_raid_broadcasts = get_raid_messages();
        for possible_raid_broadcast in possible_raid_broadcasts {
            let broadcast_type = osrs_broadcast_extractor::get_broadcast_type(possible_raid_broadcast.message);
            assert!(matches!(broadcast_type, osrs_broadcast_extractor::BroadcastType::RaidDrop));
        }
    }

    #[test]
    fn test_pet_broadcast() {
        let possible_pet_broadcasts = get_pet_messages();
        for possible_pet_broadcast in possible_pet_broadcasts {
            let broadcast_type = osrs_broadcast_extractor::get_broadcast_type(possible_pet_broadcast);
            assert!(matches!(broadcast_type, osrs_broadcast_extractor::BroadcastType::PetDrop));
        }
    }


    #[test]
    fn test_raid_extractor() {
        let possible_raid_broadcasts = get_raid_messages();
        for possible_raid_broadcast in possible_raid_broadcasts {
            let message = possible_raid_broadcast.message.clone();
            let possible_drop_item = osrs_broadcast_extractor::raid_broadcast_extractor(possible_raid_broadcast.message);
            match possible_drop_item {
                None => {
                    println!("Failed to extract drop item from message: {}", message);
                    assert!(false);
                }
                Some(drop_item) => {
                    assert_eq!(drop_item.item_name, possible_raid_broadcast.item_name);
                    assert_eq!(drop_item.item_quantity, possible_raid_broadcast.item_quantity);
                    assert_eq!(drop_item.item_value, possible_raid_broadcast.item_value);
                    assert_eq!(drop_item.player_it_happened_to, possible_raid_broadcast.player_it_happened_to);
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
            let possible_drop_item = osrs_broadcast_extractor::drop_broadcast_extractor(possible_drop_broadcast.message);
            match possible_drop_item {
                None => {
                    println!("Failed to extract drop item from message: {}", message);
                    assert!(false);
                }
                Some(drop_item) => {
                    assert_eq!(drop_item.item_name, possible_drop_broadcast.item_name);
                    assert_eq!(drop_item.item_quantity, possible_drop_broadcast.item_quantity);
                    assert_eq!(drop_item.item_value, possible_drop_broadcast.item_value);
                    assert_eq!(drop_item.player_it_happened_to, possible_drop_broadcast.player_it_happened_to);
                    assert_eq!(drop_item.item_icon, possible_drop_broadcast.item_icon);
                }
            }
        }
    }

    #[test]
    fn test_extract_message_drop_single_item() {
        let possible_drop_broadcasts = get_drop_messages();
        for possible_drop_broadcast in possible_drop_broadcasts {
            let message = possible_drop_broadcast.message.clone();
            let extracted_message = osrs_broadcast_extractor::extract_message(ClanMessage {
                author: "Insomniacs".to_string(),
                message: possible_drop_broadcast.message.clone(),
            });
            match extracted_message {
                None => {
                    println!("Failed to extract drop item from message: {}", message);
                    assert!(false);
                }
                Some(extracted_message) => {
                    assert_eq!(extracted_message.player_it_happened_to, possible_drop_broadcast.player_it_happened_to);
                    assert_eq!(extracted_message.message, possible_drop_broadcast.discord_message);
                    assert_eq!(extracted_message.icon_url, possible_drop_broadcast.item_icon);
                }
            }
        }
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

    fn get_raid_messages() -> Vec<ItemMessageTest> {
        let mut possible_raid_broadcasts: Vec<ItemMessageTest> = Vec::new();
        possible_raid_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received special loot from a raid: Twisted buckler.".to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Twisted buckler".to_string(),
            item_quantity: 1,
            item_value: None,
            item_icon: Some("https://oldschool.runescape.wiki/images/Twisted_buckler.png".to_string()),
            discord_message: "RuneScape Player received special loot from a raid: Twisted buckler.".to_string(),
        });
        possible_raid_broadcasts.push(ItemMessageTest {
            message: "Player received special loot from a raid: Twisted bow.".to_string(),
            player_it_happened_to: "Player".to_string(),
            item_name: "Twisted bow".to_string(),
            item_quantity: 1,
            item_value: None,
            item_icon: Some("https://oldschool.runescape.wiki/images/Twisted_bow.png".to_string()),
            discord_message: "Player received special loot from a raid: Twisted bow.".to_string(),
        });
        possible_raid_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received special loot from a raid: Tumeken's shadow (uncharged)".to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Tumeken's shadow (uncharged)".to_string(),
            item_quantity: 1,
            item_value: None,
            item_icon: Some("https://oldschool.runescape.wiki/images/Tumeken%27s_shadow_%28uncharged%29.png".to_string()),
            discord_message: "RuneScape Player received special loot from a raid: Tumeken's shadow (uncharged)".to_string(),
        });
        possible_raid_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received special loot from a raid: Justiciar legguards.".to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Justiciar legguards".to_string(),
            item_quantity: 1,
            item_value: None,
            item_icon: Some("https://oldschool.runescape.wiki/images/Justiciar_legguards.png".to_string()),
            discord_message: "RuneScape Player received special loot from a raid: Justiciar legguards.".to_string(),
        });

        // possible_raid_broadcasts.push("RuneScape  received special loot from a raid: Twisted buckler.".to_string());
        // possible_raid_broadcasts.push("Player received special loot from a raid: Twisted bow.".to_string());
        // possible_raid_broadcasts.push("RuneScape Player received special loot from a raid: Tumeken's shadow (uncharged)".to_string());
        // possible_raid_broadcasts.push("RuneScape Player received special loot from a raid: Justiciar legguards.".to_string());
        return possible_raid_broadcasts;
    }

    fn get_drop_messages() -> Vec<ItemMessageTest> {
        let mut possible_drop_broadcasts: Vec<ItemMessageTest> = Vec::new();

        possible_drop_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received a drop: Abyssal whip (1,456,814 coins).".to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Abyssal whip".to_string(),
            item_quantity: 1,
            item_value: Some(1_456_814),
            item_icon: Some("https://oldschool.runescape.wiki/images/Abyssal_whip.png".to_string()),
            discord_message: "RuneScape Player received a drop: Abyssal whip (1,456,814 coins).".to_string(),
        });

        possible_drop_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received a drop: Unknown Item.".to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Unknown Item".to_string(),
            item_quantity: 1,
            item_value: Some(0),
            item_icon: Some("https://oldschool.runescape.wiki/images/Unknown_Item.png".to_string()),
            discord_message: "RuneScape Player received a drop: Unknown Item (0 coins).".to_string(),
        });

        possible_drop_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received a drop: 587 x Cannonball (111,530 coins).".to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Cannonball".to_string(),
            item_quantity: 587,
            item_value: Some(111530),
            item_icon: Some("https://oldschool.runescape.wiki/images/Cannonball.png".to_string()),
            discord_message: "RuneScape Player received a drop: 587 x Cannonball (111,530 coins).".to_string(),
        });

        possible_drop_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received a drop: Awakener's orb (2,238,871 coins).".to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Awakener's orb".to_string(),
            item_quantity: 1,
            item_value: Some(2_238_871),
            item_icon: Some("https://oldschool.runescape.wiki/images/Awakener%27s_orb.png".to_string()),
            discord_message: "RuneScape Player received a drop: Awakener's orb (2,238,871 coins).".to_string(),
        });

        return possible_drop_broadcasts;
    }

    fn get_pet_messages() -> Vec<String> {
        let mut possible_pet_broadcasts: Vec<String> = Vec::new();
        possible_pet_broadcasts.push("Op Rausta has a funny feeling like he's being followed: Butch at 194 kills.".to_string());
        possible_pet_broadcasts.push("Runescape Vision has a funny feeling like she would have been followed: Heron at 11,212,255 XP.".to_string());

        return possible_pet_broadcasts;
    }
}