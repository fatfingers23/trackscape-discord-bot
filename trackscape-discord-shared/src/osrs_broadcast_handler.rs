use crate::database::clan_mate_collection_log_totals::ClanMateCollectionLogTotals;
use crate::database::clan_mates::ClanMates;
use crate::database::drop_logs_db::DropLogs;
use crate::database::guilds_db::RegisteredGuildModel;
use crate::ge_api::ge_api::{get_item_value_by_id, GeItemMapping};
use crate::jobs::new_pb_job::record_new_pb;
use crate::jobs::{remove_clanmate_job, JobQueue};
use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::{
    coffer_donation_broadcast_extractor, coffer_withdrawal_broadcast_extractor,
    collection_log_broadcast_extractor, diary_completed_broadcast_extractor,
    drop_broadcast_extractor, expelled_from_clan_broadcast_extractor, get_broadcast_type,
    invite_broadcast_extractor, left_the_clan_broadcast_extractor,
    levelmilestone_broadcast_extractor, personal_best_broadcast_extractor, pet_broadcast_extractor,
    pk_broadcast_extractor, quest_completed_broadcast_extractor, raid_broadcast_extractor,
    xpmilestone_broadcast_extractor, BroadcastType, ClanMessage,
};
use crate::wiki_api::wiki_api::WikiQuest;
use log::error;
use num_format::{Locale, ToFormattedString};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Clone)]
pub struct OSRSBroadcastHandler<
    T: DropLogs,
    CL: ClanMateCollectionLogTotals,
    CM: ClanMates,
    J: JobQueue,
> {
    clan_message: ClanMessage,
    item_mapping: Option<GeItemMapping>,
    quests: Option<Vec<WikiQuest>>,
    registered_guild: RegisteredGuildModel,
    leagues_message: bool,
    drop_log_db: T,
    collection_log_db: CL,
    clan_mates_db: CM,
    job_queue: Arc<J>,
}

impl<T: DropLogs, CL: ClanMateCollectionLogTotals, CM: ClanMates, J: JobQueue>
    OSRSBroadcastHandler<T, CL, CM, J>
{
    pub fn new(
        clan_message: ClanMessage,
        item_mapping_from_state: Result<GeItemMapping, ()>,
        quests_from_state: Result<Vec<WikiQuest>, ()>,
        register_guild: RegisteredGuildModel,
        leagues_message: bool,
        drop_log_db: T,
        collection_log_db: CL,
        clan_mates_db: CM,
        job_queue: Arc<J>,
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
            leagues_message,
            drop_log_db,
            collection_log_db,
            clan_mates_db,
            job_queue: job_queue,
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

                        if !self.leagues_message {
                            self.drop_log_db
                                .new_drop_log(drop_item.clone(), self.registered_guild.guild_id)
                                .await;
                        }

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

                        let title = match self.leagues_message {
                            true => ":bar_chart: New Leagues raid drop!".to_string(),
                            false => ":tada: New raid drop!".to_string(),
                        };

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
                            title,
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

                        let title = match self.leagues_message {
                            true => ":bar_chart: New Leagues Pet drop!".to_string(),
                            false => ":tada: New Pet drop!".to_string(),
                        };

                        Some(BroadcastMessageToDiscord {
                            type_of_broadcast: BroadcastType::PetDrop,
                            player_it_happened_to: pet_drop.player_it_happened_to,
                            message: self.clan_message.message.clone(),
                            icon_url: pet_drop.pet_icon,
                            title,
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
            BroadcastType::ExpelledFromClan => {
                let possible_expelled_broadcast =
                    expelled_from_clan_broadcast_extractor(self.clan_message.message.clone());

                match possible_expelled_broadcast {
                    None => {
                        error!(
                            "Failed to extract left clan info from message: {}",
                            self.clan_message.message.clone()
                        );
                        None
                    }
                    Some(clan_mate_who_got_kicked) => {
                        let job = remove_clanmate_job::remove_clanmate::new(
                            clan_mate_who_got_kicked.clone(),
                            self.registered_guild.guild_id,
                        );
                        let _ = self.job_queue.send_task(job).await;

                        let is_disallowed = self
                            .registered_guild
                            .disallowed_broadcast_types
                            .iter()
                            .find(|&x| {
                                if let BroadcastType::LeftTheClan = x {
                                    return true;
                                }
                                false
                            });
                        if is_disallowed.is_some() {
                            return None;
                        }
                        Some(BroadcastMessageToDiscord {
                            type_of_broadcast: BroadcastType::LeftTheClan,
                            player_it_happened_to: clan_mate_who_got_kicked,
                            message: self.clan_message.message.clone(),
                            icon_url: Some(
                                "https://oldschool.runescape.wiki/images/Your_Clan_icon.png"
                                    .to_string(),
                            ),
                            title: ":boot: Someone has been expelled!".to_string(),
                            item_quantity: None,
                        })
                    }
                }
            }
            BroadcastType::LeftTheClan => {
                let possible_left_broadcast =
                    left_the_clan_broadcast_extractor(self.clan_message.message.clone());

                match possible_left_broadcast {
                    None => {
                        error!(
                            "Failed to extract left clan info from message: {}",
                            self.clan_message.message.clone()
                        );
                        None
                    }
                    Some(clan_mate_who_left) => {
                        let job = remove_clanmate_job::remove_clanmate::new(
                            clan_mate_who_left.clone(),
                            self.registered_guild.guild_id,
                        );
                        let _ = self.job_queue.send_task(job).await;

                        let is_disallowed = self
                            .registered_guild
                            .disallowed_broadcast_types
                            .iter()
                            .find(|&x| {
                                if let BroadcastType::LeftTheClan = x {
                                    return true;
                                }
                                false
                            });
                        if is_disallowed.is_some() {
                            return None;
                        }
                        Some(BroadcastMessageToDiscord {
                            type_of_broadcast: BroadcastType::LeftTheClan,
                            player_it_happened_to: clan_mate_who_left,
                            message: self.clan_message.message.clone(),
                            icon_url: Some(
                                "https://oldschool.runescape.wiki/images/Your_Clan_icon.png"
                                    .to_string(),
                            ),
                            title: ":people_hugging: Someone has left the clan!".to_string(),
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
                        let title = match self.leagues_message {
                            true => ":bar_chart: New Leagues Level Milestone reached!".to_string(),
                            false => ":tada: New Level Milestone reached!".to_string(),
                        };
                        Some(BroadcastMessageToDiscord {
                            type_of_broadcast: BroadcastType::LevelMilestone,
                            player_it_happened_to: levelmilestone_broadcast.clan_mate,
                            message: self.clan_message.message.clone(),
                            icon_url: levelmilestone_broadcast.skill_icon,
                            title,
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
                        let title = match self.leagues_message {
                            true => ":bar_chart: New Leagues XP Milestone reached!".to_string(),
                            false => ":tada: New XP Milestone reached!".to_string(),
                        };
                        Some(BroadcastMessageToDiscord {
                            type_of_broadcast: BroadcastType::XPMilestone,
                            player_it_happened_to: xpmilestone_broadcast.clan_mate,
                            message: self.clan_message.message.clone(),
                            icon_url: xpmilestone_broadcast.skill_icon,
                            title,
                            item_quantity: None,
                        })
                    }
                }
            }
            BroadcastType::CollectionLog => self.collection_log_handler().await,
            BroadcastType::CofferDonation => {
                let possible_coffer_donation =
                    coffer_donation_broadcast_extractor(self.clan_message.message.clone());

                match possible_coffer_donation {
                    None => {
                        error!(
                            "Failed to extract coffer donation info from message: {}",
                            self.clan_message.message.clone()
                        );
                        None
                    }
                    Some(coffer_donation) => {
                        let is_disallowed = self
                            .registered_guild
                            .disallowed_broadcast_types
                            .iter()
                            .find(|&x| {
                                if let BroadcastType::CofferDonation = x {
                                    return true;
                                }
                                false
                            });
                        if is_disallowed.is_some() {
                            return None;
                        }

                        Some(BroadcastMessageToDiscord {
                            type_of_broadcast: BroadcastType::CofferDonation,
                            player_it_happened_to: coffer_donation.player,
                            message: self.clan_message.message.clone(),
                            icon_url: Some(
                                "https://oldschool.runescape.wiki/images/thumb/Clan_Coffer.png/943px-Clan_Coffer.png"
                                    .to_string(),
                            ),
                            title: ":coin: New Donation!".to_string(),
                            item_quantity: None,
                        })
                    }
                }
            }
            BroadcastType::CofferWithdrawal => {
                let possible_coffer_withdrawal =
                    coffer_withdrawal_broadcast_extractor(self.clan_message.message.clone());

                match possible_coffer_withdrawal {
                    None => {
                        error!(
                            "Failed to extract coffer withdrawal info from message: {}",
                            self.clan_message.message.clone()
                        );
                        None
                    }
                    Some(coffer_withdrawal) => {
                        let is_disallowed = self
                            .registered_guild
                            .disallowed_broadcast_types
                            .iter()
                            .find(|&x| {
                                if let BroadcastType::CofferWithdrawal = x {
                                    return true;
                                }
                                false
                            });
                        if is_disallowed.is_some() {
                            return None;
                        }

                        Some(BroadcastMessageToDiscord {
                            type_of_broadcast: BroadcastType::CofferWithdrawal,
                            player_it_happened_to: coffer_withdrawal.player,
                            message: self.clan_message.message.clone(),
                            icon_url: Some(
                                "https://oldschool.runescape.wiki/images/thumb/Clan_Coffer.png/943px-Clan_Coffer.png"
                                    .to_string(),
                            ),
                            title: ":person_running: New Clan Coffer Withdrawal!".to_string(),
                            item_quantity: None,
                        })
                    }
                }
            }
            BroadcastType::PersonalBest => self.personal_best_handler().await,
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
                if !self.leagues_message {
                    self.drop_log_db
                        .new_drop_log(drop_item.clone(), self.registered_guild.guild_id)
                        .await;
                }
                let is_disallowed = self.check_if_allowed_broad_cast(BroadcastType::ItemDrop);
                if is_disallowed {
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

                let title = match self.leagues_message {
                    true => ":bar_chart: New Leagues High Value drop!".to_string(),
                    false => ":tada: New High Value drop!".to_string(),
                };

                Some(BroadcastMessageToDiscord {
                    player_it_happened_to: drop_item.player_it_happened_to.clone(),
                    type_of_broadcast: BroadcastType::ItemDrop,
                    title,
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
                let title = match self.leagues_message {
                    true => ":bar_chart: New Leagues PK!".to_string(),
                    false => ":crossed_swords: New PK!".to_string(),
                };
                Some(BroadcastMessageToDiscord {
                    type_of_broadcast: BroadcastType::Pk,
                    player_it_happened_to: pk_broadcast.winner,
                    message: self.clan_message.message.clone(),
                    icon_url: Some("https://oldschool.runescape.wiki/images/Skull.png".to_string()),
                    title,
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
                let title = match self.leagues_message {
                    true => ":bar_chart: New Leagues quest completed!".to_string(),
                    false => ":tada: New quest completed!".to_string(),
                };
                Some(BroadcastMessageToDiscord {
                    type_of_broadcast: BroadcastType::Quest,
                    player_it_happened_to: exported_data.player_it_happened_to,
                    message: self.clan_message.message.clone(),
                    icon_url: exported_data.quest_reward_scroll_icon,
                    title,
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

                let title = match self.leagues_message {
                    true => ":bar_chart: New Leagues diary completed!".to_string(),
                    false => ":tada: New diary completed!".to_string(),
                };
                Some(BroadcastMessageToDiscord {
                    type_of_broadcast: BroadcastType::Diary,
                    player_it_happened_to: exported_data.player_it_happened_to,
                    message: self.clan_message.message.clone(),
                    icon_url: Some(
                        "https://oldschool.runescape.wiki/images/Achievement_Diaries.png"
                            .to_string(),
                    ),
                    title,
                    item_quantity: None,
                })
            }
        }
    }

    async fn collection_log_handler(&self) -> Option<BroadcastMessageToDiscord> {
        let possible_collection_log =
            collection_log_broadcast_extractor(self.clan_message.message.clone());
        match possible_collection_log {
            None => {
                error!(
                    "Failed to extract collection log info from message: {}",
                    self.clan_message.message.clone()
                );
                None
            }
            Some(collection_log_broadcast) => {
                if !self.leagues_message {
                    let possible_clan_mate = self
                        .clan_mates_db
                        .find_or_create_clan_mate(
                            self.registered_guild.guild_id,
                            collection_log_broadcast.player_it_happened_to.clone(),
                        )
                        .await;
                    let _ = match possible_clan_mate {
                        Ok(clan_mate) => {
                            self.collection_log_db
                                .update_or_create(
                                    clan_mate.guild_id,
                                    clan_mate.id,
                                    collection_log_broadcast.log_slots,
                                )
                                .await
                        }
                        Err(error) => {
                            error!("{:?}", error);
                            Err(error)
                        }
                    };
                }
                let is_disallowed = self.check_if_allowed_broad_cast(BroadcastType::CollectionLog);
                if is_disallowed {
                    return None;
                }
                let title = match self.leagues_message {
                    true => ":bar_chart: New Leagues collection log item!".to_string(),
                    false => ":tada: New collection log item!".to_string(),
                };
                Some(BroadcastMessageToDiscord {
                    type_of_broadcast: BroadcastType::CollectionLog,
                    player_it_happened_to: collection_log_broadcast.player_it_happened_to,
                    message: self.clan_message.message.clone(),
                    icon_url: collection_log_broadcast.item_icon,
                    title,
                    item_quantity: None,
                })
            }
        }
    }

    async fn personal_best_handler(&self) -> Option<BroadcastMessageToDiscord> {
        let personal_best = personal_best_broadcast_extractor(self.clan_message.message.clone());
        match personal_best {
            None => {
                error!(
                    "Failed to extract Personal Best info from message: {}",
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
                        if let BroadcastType::PersonalBest = x {
                            return true;
                        }
                        false
                    });
                if is_disallowed.is_some() {
                    return None;
                }

                let job = record_new_pb::new(exported_data.clone(), self.registered_guild.guild_id);
                let _ = self.job_queue.send_task(job).await;

                Some(BroadcastMessageToDiscord {
                    type_of_broadcast: BroadcastType::PersonalBest,
                    player_it_happened_to: exported_data.player,
                    message: self.clan_message.message.clone(),
                    icon_url: Some(self.best_guest_pb_icon(exported_data.activity).to_string()),
                    title: ":stopwatch: New Personal Best!".to_string(),
                    item_quantity: None,
                })
            }
        }
    }

    fn best_guest_pb_icon(&self, activity: String) -> String {
        match activity.as_str().to_lowercase() {
            x if x.contains("theatre of blood") => {
                "https://oldschool.runescape.wiki/images/Theatre_of_Blood_logo.png".to_string()
            }
            x if x.contains("chambers of xeric") => {
                "https://oldschool.runescape.wiki/images/Chambers_of_Xeric_logo.png".to_string()
            }
            x if x.contains("tombs of amascut") => {
                "https://oldschool.runescape.wiki/images/Tombs_of_Amascut.png".to_string()
            }
            _ => format!("https://oldschool.runescape.wiki/images/{}.png", activity).to_string(),
        }
    }

    fn check_if_allowed_broad_cast(&self, broadcast_type: BroadcastType) -> bool {
        let is_disallowed = self
            .registered_guild
            .disallowed_broadcast_types
            .iter()
            .find(|&x| {
                if broadcast_type.to_string() == x.to_string() {
                    return true;
                }
                false
            });
        if is_disallowed.is_some() {
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::clan_mate_collection_log_totals::MockClanMateCollectionLogTotals;
    use crate::database::clan_mates::MockClanMates;
    use crate::database::drop_logs_db::MockDropLogs;
    use crate::ge_api::ge_api::GetItem;
    use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::{DiaryTier, QuestDifficulty};
    use async_trait::async_trait;
    use celery::error::CeleryError;
    use celery::prelude::Task;
    use celery::task::{AsyncResult, Signature};
    use log::info;
    use mockall::mock;

    mock! {
        pub JobQueue {
            async fn send_task<T: Task + 'static>(&self, task_sig: Signature<T>) -> Result<AsyncResult, CeleryError>;
        }
    }

    #[async_trait]
    impl JobQueue for MockJobQueue {
        async fn send_task<T: Task>(
            &self,
            _task_sig: Signature<T>,
        ) -> Result<AsyncResult, CeleryError> {
            Ok(AsyncResult {
                task_id: "".to_string(),
            })
        }
    }
    #[tokio::test]
    async fn test_drop_item_handler_no_message_sent() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player: bob received a drop: Abyssal whip (1,456,814 coins)."
                .to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
            is_league_world: None,
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

        let mut drop_log_db_mock = MockDropLogs::new();
        drop_log_db_mock.expect_new_drop_log().returning(|_, _| {
            info!("Should not be calling this function");
        });

        let mock_job_queue = MockJobQueue::new();

        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            false,
            drop_log_db_mock,
            MockClanMateCollectionLogTotals::new(),
            MockClanMates::new(),
            Arc::from(mock_job_queue),
        );

        let extracted_message = handler.drop_item_handler().await;
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

    #[tokio::test]
    async fn test_drop_item_handler_message_sent() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player: bob received a drop: Cool Item (20,456,814 coins)."
                .to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
            is_league_world: None,
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

        let mut drop_log_db_mock = MockDropLogs::new();
        drop_log_db_mock.expect_new_drop_log().returning(|_, _| {
            info!("Should not be calling this function");
        });

        let mock_job_queue = MockJobQueue::new();

        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            false,
            drop_log_db_mock,
            MockClanMateCollectionLogTotals::new(),
            MockClanMates::new(),
            Arc::from(mock_job_queue),
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

    #[tokio::test]
    async fn check_disallowed_do_not_send() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player: bob received a drop: Cool Item (20,456,814 coins)."
                .to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
            is_league_world: None,
        };

        let mut registered_guild = RegisteredGuildModel::new(123);
        registered_guild.disallowed_broadcast_types = vec![BroadcastType::ItemDrop];
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

        let mut drop_log_db_mock = MockDropLogs::new();
        drop_log_db_mock.expect_new_drop_log().returning(|_, _| {
            info!("Should not be calling this function");
        });

        let mock_job_queue = MockJobQueue::new();

        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            false,
            drop_log_db_mock,
            MockClanMateCollectionLogTotals::new(),
            MockClanMates::new(),
            Arc::from(mock_job_queue),
        );

        let extracted_message = handler.drop_item_handler().await;
        info!("Extracted message: {:?}", extracted_message);
        match extracted_message {
            None => {
                //not sent
                assert_eq!(true, true);
            }
            Some(_) => {
                //sent
                info!("The broadcast type should be disallowed and not sending");
                assert_eq!(true, false);
            }
        }
    }

    #[tokio::test]
    async fn check_disallowed_do_send() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player: bob received a drop: Cool Item (20,456,814 coins)."
                .to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
            is_league_world: None,
        };

        let registered_guild = RegisteredGuildModel::new(123);
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

        let mut drop_log_db_mock = MockDropLogs::new();
        drop_log_db_mock.expect_new_drop_log().returning(|_, _| {
            info!("Should not be calling this function");
        });

        let mock_job_queue = MockJobQueue::new();

        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            false,
            drop_log_db_mock,
            MockClanMateCollectionLogTotals::new(),
            MockClanMates::new(),
            Arc::from(mock_job_queue),
        );

        let extracted_message = handler.drop_item_handler().await;
        info!("Extracted message: {:?}", extracted_message);
        match extracted_message {
            None => {
                //not sent
                info!("The broadcast type should not be disallowed and sending");
                assert_eq!(true, false);
            }
            Some(_) => {
                //sent
                assert_eq!(true, true);
            }
        }
    }

    #[tokio::test]
    async fn test_quest_handler_message_sent() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player has completed a quest: The Fremennik Isles".to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
            is_league_world: None,
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

        let mock_job_queue = MockJobQueue::new();

        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            false,
            MockDropLogs::new(),
            MockClanMateCollectionLogTotals::new(),
            MockClanMates::new(),
            Arc::from(mock_job_queue),
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

    #[tokio::test]
    async fn test_quest_handler_message_not_sent() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player has completed a quest: Cook's Assistant".to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
            is_league_world: None,
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

        let mock_job_queue = MockJobQueue::new();

        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            false,
            MockDropLogs::new(),
            MockClanMateCollectionLogTotals::new(),
            MockClanMates::new(),
            Arc::from(mock_job_queue),
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

    #[tokio::test]
    async fn test_pk_value_handler_low() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "HolidayPanda has been defeated by next trial in The Wilderness and lost (33,601 coins) worth of loot."
                .to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
            is_league_world: None,
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
        let mock_job_queue = MockJobQueue::new();

        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            false,
            MockDropLogs::new(),
            MockClanMateCollectionLogTotals::new(),
            MockClanMates::new(),
            Arc::from(mock_job_queue),
        );

        let extracted_message = handler.pk_handler();
        println!("Extracted message: {:?}", extracted_message);

        match extracted_message {
            None => {
                println!("Successfully stopped message from sending");
            }
            Some(_) => {
                println!("Should not be sending a message.");
                assert_eq!(true, false);
            }
        }
    }

    #[tokio::test]
    async fn test_diary_handler_message_sent() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player has completed the Hard Ardougne diary.".to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
            is_league_world: None,
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

        let mock_job_queue = MockJobQueue::new();

        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            false,
            MockDropLogs::new(),
            MockClanMateCollectionLogTotals::new(),
            MockClanMates::new(),
            Arc::from(mock_job_queue),
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

    #[tokio::test]
    async fn test_diary_handler_message_not_sent() {
        let clan_message = ClanMessage {
            sender: "Insomniacs".to_string(),
            message: "RuneScape Player has completed the Easy Ardougne diary.".to_string(),
            clan_name: "Insomniacs".to_string(),
            rank: "Recruit".to_string(),
            icon_id: None,
            is_league_world: None,
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

        let mock_job_queue = MockJobQueue::new();

        let handler = OSRSBroadcastHandler::new(
            clan_message,
            get_item_mapping,
            quests,
            registered_guild,
            false,
            MockDropLogs::new(),
            MockClanMateCollectionLogTotals::new(),
            MockClanMates::new(),
            Arc::from(mock_job_queue),
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
