pub mod ge_api {
    use redis::Connection;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    use crate::redis_helpers::{fetch_redis_json_object, write_to_cache_with_seconds};

    const CACHE_KEY: &str = "ge_mapping";

    static APP_USER_AGENT: &str = concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
        "/",
        "GitHub:https://github.com/fatfingers23/trackscape-discord-bot"
    );
    const BASE_URL: &str = "https://prices.runescape.wiki/api/v1/";

    pub async fn get_item_mapping(
        redis_connection: &mut Connection,
    ) -> Result<GeItemMapping, anyhow::Error> {
        let cached_result =
            fetch_redis_json_object::<GeItemMapping>(redis_connection, CACHE_KEY).await;

        match cached_result {
            Ok(ge_mapping) => {
                return Ok(ge_mapping);
            }
            Err(_) => {
                let client = reqwest::Client::builder()
                    .user_agent(APP_USER_AGENT)
                    .build()?;
                let ge_mapping = client
                    .get(format!("{}{}", BASE_URL, "osrs/mapping").as_str())
                    .send()
                    .await?
                    .json::<GeItemMapping>()
                    .await?;
                write_to_cache_with_seconds(
                    redis_connection,
                    CACHE_KEY,
                    ge_mapping.clone(),
                    604800,
                )
                .await;
                Ok(ge_mapping)
            }
        }
    }

    pub async fn get_item_value_by_id(id: i64) -> Result<GeItemPrice, reqwest::Error> {
        let client = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()?;
        let resp: Value = client
            .get(format!("{}{}?id={}", BASE_URL, "osrs/latest", id).as_str())
            .send()
            .await?
            .json()
            .await?;
        if resp["data"][id.to_string()].is_null() {
            return Ok(GeItemPrice {
                high: 0,
                high_time: 0,
                low: 0,
                low_time: 0,
            });
        }
        let price: GeItemPrice =
            serde_json::from_value(resp["data"][id.to_string()].clone()).unwrap();

        Ok(price)
    }

    pub type GeItemMapping = Vec<GetItem>;

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetItem {
        pub highalch: Option<i64>,
        pub members: bool,
        pub name: String,
        pub examine: String,
        pub id: i64,
        // The wiki omits `value` entirely for placeholder/unreleased items, so
        // default it rather than failing the whole mapping (see ge_api tests).
        #[serde(default)]
        pub value: i64,
        pub icon: String,
        pub lowalch: Option<i64>,
        pub limit: Option<i64>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GeItemPrice {
        pub high: i64,
        pub high_time: i64,
        pub low: i64,
        pub low_time: i64,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // The wiki's osrs/mapping response omits `value` for placeholder/unreleased
        // items. That used to fail deserialization of the entire mapping, so
        // `ge_mapping` was never written to redis and every request re-fetched.
        #[test]
        fn test_mapping_deserializes_when_item_value_is_missing() {
            let payload = r#"[
                {
                    "name": "Dummy stymphike feather",
                    "examine": "placeholder",
                    "id": 33644,
                    "members": false,
                    "icon": "Dummy stymphike feather.png"
                },
                {
                    "examine": "Fabulously ancient mage protection enchanted in the 3rd Age.",
                    "id": 10344,
                    "members": true,
                    "lowalch": 20200,
                    "limit": 8,
                    "value": 50500,
                    "highalch": 30300,
                    "icon": "3rd Age amulet.png",
                    "name": "3rd Age amulet"
                }
            ]"#;

            let mapping: GeItemMapping = serde_json::from_str(payload).unwrap();

            assert_eq!(mapping.len(), 2);
            assert_eq!(mapping[0].id, 33644);
            assert_eq!(mapping[0].value, 0);
            assert_eq!(mapping[0].highalch, None);
            assert_eq!(mapping[1].id, 10344);
            assert_eq!(mapping[1].value, 50500);
        }
    }
}
