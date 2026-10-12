/// Possible Responses found in [Competitions Types & Entities](https://docs.wiseoldman.net/competitions-api/competition-type-definitions)
pub mod competition;
///Error models from the api
pub mod error;
/// Possible enums found in all models [Global Types & Entities](https://docs.wiseoldman.net/global-type-definitions)
pub mod global_enums;
/// Global types for models
pub mod global_types;
/// Possible Responses found in [Group Types & Entities](https://docs.wiseoldman.net/groups-api/group-type-definitions)
pub mod group;
/// Possible Responses found in [Name Change Types & Entities](https://docs.wiseoldman.net/names-api/name-type-definitions)
pub mod name;
/// Possible Responses found in [Player Types & Entities](https://docs.wiseoldman.net/players-api/player-type-definitions)
pub mod player;
/// Possible Responses found in [Record Types & Entities](https://docs.wiseoldman.net/records-api/record-type-definitions)
pub mod record;
/// Mapping used for snapshots since they are a bit more complex. Very similar to the PlayerGained in models::player, but kept separate for clarity.
pub mod snapshot;
