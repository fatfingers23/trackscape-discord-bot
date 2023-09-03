

pub mod osrs_broadcast_extractor {
    use std::collections::HashMap;

    pub struct ClanMessage {
        pub author: String,
        pub message: String,
    }

    pub struct BroadcastMessageToDiscord {
        pub player_it_happened_to: String,
        pub type_of_broadcast: BroadcastType,
        pub message: String,
    }

    pub struct DropItem{
        pub player_it_happened_to: String,
        pub item_name: String,
        pub item_quantity: usize,
        pub item_value: Option<u64>,
        pub item_icon: Option<String>,
    }

    #[derive(PartialEq)]
    pub enum BroadcastType {
        ItemDrop,
        Level,
        PetDrop,
        XPMilestone,
        Quest,
        Pk,
        RaidDrop,
        Unknown
    }

    pub fn extract_message(message: ClanMessage){
        let broadcast_type = get_broadcast_type(message.message);
        match broadcast_type {
            BroadcastType::ItemDrop => {
                println!("Item Drop!");
            },
            BroadcastType::Level => {
                println!("Level!");
            },
            BroadcastType::PetDrop => {
                println!("Pet Drop!");
            },
            BroadcastType::XPMilestone => {
                println!("XP Milestone!");
            },
            BroadcastType::Quest => {
                println!("Quest!");
            },
            BroadcastType::Pk => {
                println!("PK!");
            },
            BroadcastType::RaidDrop => {
                println!("Raid Drop!");
            },
            BroadcastType::Unknown => {
                println!("Unknown!");
            }
        }
    }

    pub fn raid_broadcast_extractor(message: String) -> Option<DropItem>{

        let re = regex::Regex::new(r#"^(?P<player_name>.*?) received special loot from a raid: (?P<item>.*?)([.]|$)"#).unwrap();

        return if let Some(caps) = re.captures(message.as_str()) {
            let player_name = caps.name("player_name").unwrap().as_str();
            let item = caps.name("item").unwrap().as_str();

            Some(DropItem {
                player_it_happened_to: player_name.to_string(),
                item_name: item.to_string(),
                item_quantity: 1,
                item_value: None,
                item_icon: None,
            })
        } else {
            None
        }

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
        if message_content.contains("has reached") && message_content.contains("level") {
            return BroadcastType::Level;
        }
        return BroadcastType::Unknown;
    }
}


#[cfg(test)]
mod tests {
    use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::DropItem;
    use super::*;

    #[test]
    fn test_get_drop_type_broadcast() {
        let possible_drop_broadcasts = get_drop_messages();
        for possible_drop_broadcast in possible_drop_broadcasts  {
            let broadcast_type = osrs_broadcast_extractor::get_broadcast_type(possible_drop_broadcast);
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
    fn test_get_level_type_broadcast() {
        let possible_level_broadcasts = get_level_messages();
        for possible_level_broadcast in possible_level_broadcasts {
            let broadcast_type = osrs_broadcast_extractor::get_broadcast_type(possible_level_broadcast);
            assert!(matches!(broadcast_type, osrs_broadcast_extractor::BroadcastType::Level));
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
        item_quantity:usize,
        //The value of the item
        item_value: Option<u64>,
    }

    struct LevelMessageTest {
        message: String,
        player_it_happened_to: String,
        skill_name: String,
        skill_level: String,
        skill_xp: String,
    }


    fn get_raid_messages() -> Vec<ItemMessageTest>{
        let mut possible_raid_broadcasts: Vec<ItemMessageTest> = Vec::new();
        possible_raid_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received special loot from a raid: Twisted buckler.".to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Twisted buckler".to_string(),
            item_quantity: 1,
            item_value: None,

        });
        possible_raid_broadcasts.push(ItemMessageTest {
            message: "Player received special loot from a raid: Twisted bow.".to_string(),
            player_it_happened_to: "Player".to_string(),
            item_name: "Twisted bow".to_string(),
            item_quantity: 1,
            item_value: None,
        });
        possible_raid_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received special loot from a raid: Tumeken's shadow (uncharged)".to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Tumeken's shadow (uncharged)".to_string(),
            item_quantity: 1,
            item_value: None,
        });
        possible_raid_broadcasts.push(ItemMessageTest {
            message: "RuneScape Player received special loot from a raid: Justiciar legguards.".to_string(),
            player_it_happened_to: "RuneScape Player".to_string(),
            item_name: "Justiciar legguards".to_string(),
            item_quantity: 1,
            item_value: None,
        });

        // possible_raid_broadcasts.push("RuneScape  received special loot from a raid: Twisted buckler.".to_string());
        // possible_raid_broadcasts.push("Player received special loot from a raid: Twisted bow.".to_string());
        // possible_raid_broadcasts.push("RuneScape Player received special loot from a raid: Tumeken's shadow (uncharged)".to_string());
        // possible_raid_broadcasts.push("RuneScape Player received special loot from a raid: Justiciar legguards.".to_string());
        return  possible_raid_broadcasts;
    }

    fn get_drop_messages() -> Vec<String>{
        let mut possible_drop_broadcasts: Vec<String> = Vec::new();
        possible_drop_broadcasts.push("RuneScape Player received a drop: Abyssal whip (1,456,814 coins).".to_string());
        return  possible_drop_broadcasts;
    }

    fn get_level_messages() -> Vec<String>{
        let mut possible_level_broadcasts: Vec<String> = Vec::new();
        possible_level_broadcasts.push("RuneScape Player has reached Ranged level 99.".to_string());
        return  possible_level_broadcasts;
    }


    fn get_pet_messages() -> Vec<String>{
        let mut possible_pet_broadcasts: Vec<String> = Vec::new();
        possible_pet_broadcasts.push("Op Rausta has a funny feeling like he's being followed: Butch at 194 kills.".to_string());
        possible_pet_broadcasts.push("Runescape Vision has a funny feeling like she would have been followed: Heron at 11,212,255 XP.".to_string());

        return  possible_pet_broadcasts;
    }

}