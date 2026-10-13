//! Typed slash command choices, converted into the shared broadcast types.
//!
//! Variant order is the order Discord shows the choices in.

use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::{
    BroadcastType, DiaryTier, QuestDifficulty,
};

/// Broadcast types `/threshold` can set a GP value for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, poise::ChoiceParameter)]
pub enum ThresholdBroadcast {
    #[name = "Item Drops"]
    ItemDrop,
    #[name = "PK Loot"]
    Pk,
}

impl From<ThresholdBroadcast> for BroadcastType {
    fn from(choice: ThresholdBroadcast) -> Self {
        match choice {
            ThresholdBroadcast::ItemDrop => BroadcastType::ItemDrop,
            ThresholdBroadcast::Pk => BroadcastType::Pk,
        }
    }
}

/// Broadcast types `/reset` can put back to their default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, poise::ChoiceParameter)]
pub enum ResetBroadcast {
    #[name = "Item Drop"]
    ItemDrop,
    #[name = "Pk"]
    Pk,
    #[name = "Quest"]
    Quest,
    #[name = "Diary"]
    Diary,
    #[name = "Collection Log"]
    CollectionLog,
}

impl From<ResetBroadcast> for BroadcastType {
    fn from(choice: ResetBroadcast) -> Self {
        match choice {
            ResetBroadcast::ItemDrop => BroadcastType::ItemDrop,
            ResetBroadcast::Pk => BroadcastType::Pk,
            ResetBroadcast::Quest => BroadcastType::Quest,
            ResetBroadcast::Diary => BroadcastType::Diary,
            ResetBroadcast::CollectionLog => BroadcastType::CollectionLog,
        }
    }
}

/// Broadcast types that can have a custom filter list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, poise::ChoiceParameter)]
pub enum FilterBroadcast {
    #[name = "Item Drop"]
    ItemDrop,
    #[name = "Pet Drop"]
    PetDrop,
    #[name = "Quest"]
    Quest,
    #[name = "Diary"]
    Diary,
    #[name = "Personal Best"]
    PersonalBest,
}

impl From<FilterBroadcast> for BroadcastType {
    fn from(choice: FilterBroadcast) -> Self {
        match choice {
            FilterBroadcast::ItemDrop => BroadcastType::ItemDrop,
            FilterBroadcast::PetDrop => BroadcastType::PetDrop,
            FilterBroadcast::Quest => BroadcastType::Quest,
            FilterBroadcast::Diary => BroadcastType::Diary,
            FilterBroadcast::PersonalBest => BroadcastType::PersonalBest,
        }
    }
}

/// Broadcast types `/toggle` can turn on or off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, poise::ChoiceParameter)]
pub enum ToggleBroadcast {
    #[name = "Item Drop"]
    ItemDrop,
    #[name = "Pk"]
    Pk,
    #[name = "Quest"]
    Quest,
    #[name = "Diary"]
    Diary,
    #[name = "Raid Drop"]
    RaidDrop,
    #[name = "Pet Drop"]
    PetDrop,
    #[name = "Clue Item"]
    ClueItem,
    #[name = "Invite"]
    Invite,
    #[name = "Left The Clan"]
    LeftTheClan,
    #[name = "Expelled From Clan"]
    ExpelledFromClan,
    #[name = "Level Milestone"]
    LevelMilestone,
    #[name = "XP Milestone"]
    XPMilestone,
    #[name = "Collection Log"]
    CollectionLog,
    #[name = "Coffer Withdrawal"]
    CofferWithdrawal,
    #[name = "Coffer Donation"]
    CofferDonation,
    #[name = "Personal Best"]
    PersonalBest,
    #[name = "Combat Achievement"]
    CombatAchievement,
    #[name = "Area Unlock"]
    AreaUnlock,
    #[name = "Leagues Rank"]
    LeaguesRank,
    #[name = "Combat Masteries"]
    CombatMasteries,
    #[name = "Relic Tier"]
    RelicTier,
}

impl From<ToggleBroadcast> for BroadcastType {
    fn from(choice: ToggleBroadcast) -> Self {
        match choice {
            ToggleBroadcast::ItemDrop => BroadcastType::ItemDrop,
            ToggleBroadcast::Pk => BroadcastType::Pk,
            ToggleBroadcast::Quest => BroadcastType::Quest,
            ToggleBroadcast::Diary => BroadcastType::Diary,
            ToggleBroadcast::RaidDrop => BroadcastType::RaidDrop,
            ToggleBroadcast::PetDrop => BroadcastType::PetDrop,
            ToggleBroadcast::ClueItem => BroadcastType::ClueItem,
            ToggleBroadcast::Invite => BroadcastType::Invite,
            ToggleBroadcast::LeftTheClan => BroadcastType::LeftTheClan,
            ToggleBroadcast::ExpelledFromClan => BroadcastType::ExpelledFromClan,
            ToggleBroadcast::LevelMilestone => BroadcastType::LevelMilestone,
            ToggleBroadcast::XPMilestone => BroadcastType::XPMilestone,
            ToggleBroadcast::CollectionLog => BroadcastType::CollectionLog,
            ToggleBroadcast::CofferWithdrawal => BroadcastType::CofferWithdrawal,
            ToggleBroadcast::CofferDonation => BroadcastType::CofferDonation,
            ToggleBroadcast::PersonalBest => BroadcastType::PersonalBest,
            ToggleBroadcast::CombatAchievement => BroadcastType::CombatAchievement,
            ToggleBroadcast::AreaUnlock => BroadcastType::AreaUnlock,
            ToggleBroadcast::LeaguesRank => BroadcastType::LeaguesRank,
            ToggleBroadcast::CombatMasteries => BroadcastType::CombatMasteries,
            ToggleBroadcast::RelicTier => BroadcastType::RelicTier,
        }
    }
}

/// Quest difficulties `/quests` can set as the minimum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, poise::ChoiceParameter)]
pub enum QuestDifficultyChoice {
    #[name = "Novice"]
    Novice,
    #[name = "Intermediate"]
    Intermediate,
    #[name = "Experienced"]
    Experienced,
    #[name = "Master"]
    Master,
    #[name = "Grandmaster"]
    Grandmaster,
}

impl From<QuestDifficultyChoice> for QuestDifficulty {
    fn from(choice: QuestDifficultyChoice) -> Self {
        match choice {
            QuestDifficultyChoice::Novice => QuestDifficulty::Novice,
            QuestDifficultyChoice::Intermediate => QuestDifficulty::Intermediate,
            QuestDifficultyChoice::Experienced => QuestDifficulty::Experienced,
            QuestDifficultyChoice::Master => QuestDifficulty::Master,
            QuestDifficultyChoice::Grandmaster => QuestDifficulty::Grandmaster,
        }
    }
}

/// Diary tiers `/diaries` can set as the minimum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, poise::ChoiceParameter)]
pub enum DiaryTierChoice {
    #[name = "Easy"]
    Easy,
    #[name = "Medium"]
    Medium,
    #[name = "Hard"]
    Hard,
    #[name = "Elite"]
    Elite,
}

impl From<DiaryTierChoice> for DiaryTier {
    fn from(choice: DiaryTierChoice) -> Self {
        match choice {
            DiaryTierChoice::Easy => DiaryTier::Easy,
            DiaryTierChoice::Medium => DiaryTier::Medium,
            DiaryTierChoice::Hard => DiaryTier::Hard,
            DiaryTierChoice::Elite => DiaryTier::Elite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use poise::ChoiceParameter;

    fn assert_broadcast_choice<C>(choice: C, expected: BroadcastType)
    where
        C: ChoiceParameter + Into<BroadcastType>,
    {
        assert_eq!(choice.name(), expected.to_string());
        assert_eq!(choice.into(), expected);
    }

    #[test]
    fn threshold_broadcast_labels_and_conversions() {
        assert_eq!(ThresholdBroadcast::list().len(), 2);
        assert_eq!(ThresholdBroadcast::ItemDrop.name(), "Item Drops");
        assert_eq!(ThresholdBroadcast::Pk.name(), "PK Loot");
        assert_eq!(
            BroadcastType::from(ThresholdBroadcast::ItemDrop),
            BroadcastType::ItemDrop
        );
        assert_eq!(
            BroadcastType::from(ThresholdBroadcast::Pk),
            BroadcastType::Pk
        );
    }

    #[test]
    fn reset_broadcast_labels_and_conversions() {
        assert_eq!(ResetBroadcast::list().len(), 5);
        assert_broadcast_choice(ResetBroadcast::ItemDrop, BroadcastType::ItemDrop);
        assert_broadcast_choice(ResetBroadcast::Pk, BroadcastType::Pk);
        assert_broadcast_choice(ResetBroadcast::Quest, BroadcastType::Quest);
        assert_broadcast_choice(ResetBroadcast::Diary, BroadcastType::Diary);
        assert_broadcast_choice(ResetBroadcast::CollectionLog, BroadcastType::CollectionLog);
    }

    #[test]
    fn filter_broadcast_labels_and_conversions() {
        assert_eq!(FilterBroadcast::list().len(), 5);
        assert_broadcast_choice(FilterBroadcast::ItemDrop, BroadcastType::ItemDrop);
        assert_broadcast_choice(FilterBroadcast::PetDrop, BroadcastType::PetDrop);
        assert_broadcast_choice(FilterBroadcast::Quest, BroadcastType::Quest);
        assert_broadcast_choice(FilterBroadcast::Diary, BroadcastType::Diary);
        assert_broadcast_choice(FilterBroadcast::PersonalBest, BroadcastType::PersonalBest);
    }

    #[test]
    fn toggle_broadcast_labels_and_conversions() {
        assert_eq!(ToggleBroadcast::list().len(), 21);
        assert_broadcast_choice(ToggleBroadcast::ItemDrop, BroadcastType::ItemDrop);
        assert_broadcast_choice(ToggleBroadcast::Pk, BroadcastType::Pk);
        assert_broadcast_choice(ToggleBroadcast::Quest, BroadcastType::Quest);
        assert_broadcast_choice(ToggleBroadcast::Diary, BroadcastType::Diary);
        assert_broadcast_choice(ToggleBroadcast::RaidDrop, BroadcastType::RaidDrop);
        assert_broadcast_choice(ToggleBroadcast::PetDrop, BroadcastType::PetDrop);
        assert_broadcast_choice(ToggleBroadcast::ClueItem, BroadcastType::ClueItem);
        assert_broadcast_choice(ToggleBroadcast::Invite, BroadcastType::Invite);
        assert_broadcast_choice(ToggleBroadcast::LeftTheClan, BroadcastType::LeftTheClan);
        assert_broadcast_choice(
            ToggleBroadcast::ExpelledFromClan,
            BroadcastType::ExpelledFromClan,
        );
        assert_broadcast_choice(
            ToggleBroadcast::LevelMilestone,
            BroadcastType::LevelMilestone,
        );
        assert_broadcast_choice(ToggleBroadcast::XPMilestone, BroadcastType::XPMilestone);
        assert_broadcast_choice(ToggleBroadcast::CollectionLog, BroadcastType::CollectionLog);
        assert_broadcast_choice(
            ToggleBroadcast::CofferWithdrawal,
            BroadcastType::CofferWithdrawal,
        );
        assert_broadcast_choice(
            ToggleBroadcast::CofferDonation,
            BroadcastType::CofferDonation,
        );
        assert_broadcast_choice(ToggleBroadcast::PersonalBest, BroadcastType::PersonalBest);
        assert_broadcast_choice(
            ToggleBroadcast::CombatAchievement,
            BroadcastType::CombatAchievement,
        );
        assert_broadcast_choice(ToggleBroadcast::AreaUnlock, BroadcastType::AreaUnlock);
        assert_broadcast_choice(ToggleBroadcast::LeaguesRank, BroadcastType::LeaguesRank);
        assert_broadcast_choice(
            ToggleBroadcast::CombatMasteries,
            BroadcastType::CombatMasteries,
        );
        assert_broadcast_choice(ToggleBroadcast::RelicTier, BroadcastType::RelicTier);
    }

    #[test]
    fn quest_difficulty_labels_and_conversions() {
        assert_eq!(QuestDifficultyChoice::list().len(), 5);
        let cases = [
            (QuestDifficultyChoice::Novice, QuestDifficulty::Novice),
            (
                QuestDifficultyChoice::Intermediate,
                QuestDifficulty::Intermediate,
            ),
            (
                QuestDifficultyChoice::Experienced,
                QuestDifficulty::Experienced,
            ),
            (QuestDifficultyChoice::Master, QuestDifficulty::Master),
            (
                QuestDifficultyChoice::Grandmaster,
                QuestDifficulty::Grandmaster,
            ),
        ];
        for (choice, expected) in cases {
            assert_eq!(choice.name(), expected.to_string());
            let converted = QuestDifficulty::from(choice);
            // QuestDifficulty has no PartialEq, ranking is unique per variant
            assert_eq!(converted.ranking(), expected.ranking());
            assert_eq!(converted.to_string(), expected.to_string());
        }
    }

    #[test]
    fn diary_tier_labels_and_conversions() {
        assert_eq!(DiaryTierChoice::list().len(), 4);
        let cases = [
            (DiaryTierChoice::Easy, DiaryTier::Easy),
            (DiaryTierChoice::Medium, DiaryTier::Medium),
            (DiaryTierChoice::Hard, DiaryTier::Hard),
            (DiaryTierChoice::Elite, DiaryTier::Elite),
        ];
        for (choice, expected) in cases {
            assert_eq!(choice.name(), expected.to_string());
            let converted = DiaryTier::from(choice);
            // DiaryTier has no PartialEq, ranking is unique per variant
            assert_eq!(converted.ranking(), expected.ranking());
            assert_eq!(converted.to_string(), expected.to_string());
        }
    }
}
