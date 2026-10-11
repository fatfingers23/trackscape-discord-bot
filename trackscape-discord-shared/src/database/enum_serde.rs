//! Glue helpers for storing Rust enums as Postgres TEXT/JSONB without
//! changing the public model types.
//!
//! The enums (`BroadcastType`, `QuestDifficulty`, `DiaryTier`) already derive
//! `Serialize`/`Deserialize` with their variant names as plain strings, so we
//! round trip them through `serde_json` and keep Postgres columns as TEXT.
//! Invalid values in the database are skipped (or `None`) instead of failing
//! the whole read.

use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;

/// Serializes a plain unit-variant enum to its string form, e.g.
/// `BroadcastType::ItemDrop` -> `"ItemDrop"`.
pub fn enum_to_string<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default()
}

/// Parses a string back into a plain unit-variant enum, `None` if invalid.
pub fn enum_from_string<T: DeserializeOwned>(value: &str) -> Option<T> {
    serde_json::from_value(serde_json::Value::String(value.to_string())).ok()
}

/// `Vec<Enum>` -> `Vec<String>` for TEXT[] columns.
pub fn enum_vec_to_strings<T: Serialize>(values: &[T]) -> Vec<String> {
    values.iter().map(enum_to_string).collect()
}

/// `Vec<String>` -> `Vec<Enum>` for TEXT[] columns, skipping invalid entries.
pub fn enum_vec_from_strings<T: DeserializeOwned>(values: Vec<String>) -> Vec<T> {
    values.iter().filter_map(|v| enum_from_string(v)).collect()
}

/// Serializes the custom drop broadcast filter (`HashMap<BroadcastType, ...>`)
/// into a JSONB value. Map keys serialize as the variant name strings.
pub fn custom_filter_to_json(filter: &HashMap<BroadcastType, Vec<String>>) -> serde_json::Value {
    serde_json::to_value(filter).unwrap_or(serde_json::Value::Null)
}

/// Parses a JSONB value back into the custom drop broadcast filter. Returns
/// `None` for NULL/invalid values.
pub fn custom_filter_from_json(
    value: Option<serde_json::Value>,
) -> Option<HashMap<BroadcastType, Vec<String>>> {
    value.and_then(|value| serde_json::from_value(value).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;

    #[test]
    fn test_enum_round_trip() {
        assert_eq!(enum_to_string(&BroadcastType::ItemDrop), "ItemDrop");
        let parsed: Option<BroadcastType> = enum_from_string("PetDrop");
        assert_eq!(parsed, Some(BroadcastType::PetDrop));
        let invalid: Option<BroadcastType> = enum_from_string("NotARealType");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_custom_filter_round_trip() {
        let mut filter = HashMap::new();
        filter.insert(
            BroadcastType::ItemDrop,
            vec!["Dragon Warhammer".to_string()],
        );
        let json = custom_filter_to_json(&filter);
        assert_eq!(json["ItemDrop"][0], "Dragon Warhammer");
        let parsed = custom_filter_from_json(Some(json));
        assert_eq!(parsed, Some(filter));
        assert_eq!(custom_filter_from_json(None), None);
    }
}
