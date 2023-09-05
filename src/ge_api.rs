
pub mod ge_api {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    "/",
    "DiscordContact:fatfingers23"
    );
    const BASE_URL: &str = "https://prices.runescape.wiki/api/v1/";


    pub async fn get_item_mapping() -> Result<GeItemMapping, reqwest::Error>{
        let client = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()?;
        let resp = client.get(format!("{}{}", BASE_URL, "osrs/mapping").as_str())
            .send()
            .await?
            .json::<GeItemMapping>()
            .await?;

        Ok(resp)
    }


    pub async fn get_item_value_by_id(id: i64) -> Result<GeItemPrice, reqwest::Error>{
        let client = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()?;
        let resp: Value = client.get(format!("{}{}?id={}", BASE_URL, "osrs/latest", id).as_str())
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
            })
        }
        let price: GeItemPrice = serde_json::from_value(resp["data"][id.to_string()].clone()).unwrap();

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
}