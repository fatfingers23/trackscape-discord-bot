pub mod wiki_api {
    use crate::{
        osrs_broadcast_extractor::osrs_broadcast_extractor::QuestDifficulty,
        redis_helpers::{fetch_redis_json_object, write_to_cache_with_seconds},
    };
    use redis::Connection;
    use scraper::Html;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    const BASE_URL: &str = "https://oldschool.runescape.wiki/api.php";

    const QUEST_CACHE_KEY: &str = "quests";

    const CLOGS_CACHE_KEY: &str = "clogs";

    static APP_USER_AGENT: &str = concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
        "/",
        "GitHub:https://github.com/fatfingers23/trackscape-discord-bot"
    );

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WikiQuest {
        pub name: String,
        pub difficulty: QuestDifficulty,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WikiClogs {
        pub name: String,
        pub percentage: f64,
    }

    fn build_quest_url(difficulty: QuestDifficulty) -> String {
        format!(
            "{}?format=json&action=parse&page=Quests%2F{}&section=1",
            BASE_URL,
            difficulty.to_string()
        )
    }

    fn build_clogs_url() -> String {
        format!(
            "{}?format=json&action=parse&page=Collection_log/Table&section=2",
            BASE_URL
        )
    }

    pub async fn get_quests_and_difficulties(
        redis_connection: &mut Connection,
    ) -> Result<Vec<WikiQuest>, anyhow::Error> {
        let cached_result =
            fetch_redis_json_object::<Vec<WikiQuest>>(redis_connection, QUEST_CACHE_KEY).await;
        match cached_result {
            Ok(ge_mapping) => {
                return Ok(ge_mapping);
            }
            Err(_) => {
                let mut quests: Vec<WikiQuest> = Vec::new();
                let client = reqwest::Client::builder()
                    .user_agent(APP_USER_AGENT)
                    .build()?;
                for difficulty in QuestDifficulty::iter() {
                    let url = build_quest_url(difficulty.clone());
                    println!("Getting quests from {}", url.as_str());
                    let resp = client.get(url.as_str()).send().await;
                    match resp {
                        Ok(ok_resp) => {
                            let possible_json_body = ok_resp.json::<Root>().await;
                            match possible_json_body {
                                Ok(wiki_result) => {
                                    wiki_result.parse.links.iter().for_each(|link| {
                                        if link.ns == 0 {
                                            quests.push(WikiQuest {
                                                name: link.field.clone(),
                                                difficulty: difficulty.clone(),
                                            });
                                        }
                                    });
                                }
                                Err(e) => {
                                    println!("Failed to parse quests from wiki: {}", e);
                                    return Err(e.into());
                                }
                            }
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                write_to_cache_with_seconds(
                    redis_connection,
                    QUEST_CACHE_KEY,
                    quests.clone(),
                    604800,
                )
                .await;

                Ok(quests)
            }
        }
    }

    pub async fn get_clogs_and_percentages(
        redis_connection: &mut Connection,
    ) -> Result<Vec<WikiClogs>, anyhow::Error> {
        let cached_result =
            fetch_redis_json_object::<Vec<WikiClogs>>(redis_connection, CLOGS_CACHE_KEY).await;
        match cached_result {
            Ok(clogs_mapping) => {
                return Ok(clogs_mapping);
            }
            Err(_) => {
                let mut clogs: Vec<WikiClogs> = Vec::new();
                let client = reqwest::Client::builder()
                    .user_agent(APP_USER_AGENT)
                    .build()?;
                let url = build_clogs_url();
                println!("Getting clogs from {}", url.as_str());
                let resp = client.get(url.as_str()).send().await;
                match resp {
                    Ok(ok_resp) => {
                        let possible_json_body = ok_resp.json::<Root>().await;
                        match possible_json_body {
                            Ok(wiki_result) => {
                                let html_content = wiki_result.parse.text.field.clone();
                                let document = Html::parse_document(&html_content);
                                let selector = scraper::Selector::parse("tr").unwrap();
                                let rows = document.select(&selector);
                                for row in rows {
                                    let td_selector = scraper::Selector::parse("td").unwrap();
                                    let mut cells = row.select(&td_selector);
                                    if let Some(first_cell) = cells.next() {
                                        cells.next();
                                        if let Some(third_cell) = cells.next() {
                                            if third_cell.text().collect::<String>().contains("%") {
                                                let name = first_cell.text().collect::<String>().trim().to_string();
                                                let percentage = third_cell
                                                    .text()
                                                    .collect::<String>()
                                                    .replace("%", "")
                                                    .parse::<f64>()
                                                    .unwrap_or(100.0);
                                                clogs.push(WikiClogs {
                                                    name: name.clone(),
                                                    percentage: percentage.clone(),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Failed to parse clogs from wiki: {}", e);
                                return Err(e.into());
                            }
                        }
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
                write_to_cache_with_seconds(
                    redis_connection,
                    CLOGS_CACHE_KEY,
                    clogs.clone(),
                    604800,
                )
                .await;

                Ok(clogs)
            }
        }
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Root {
        pub parse: Parse,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Parse {
        pub title: String,
        pub pageid: i64,
        pub revid: i64,
        pub text: Text,
        pub langlinks: Vec<Value>,
        pub categories: Vec<Category>,
        pub links: Vec<Link>,
        pub templates: Vec<Template>,
        pub images: Vec<Value>,
        pub externallinks: Vec<Value>,
        pub sections: Vec<Section>,
        pub parsewarnings: Vec<Value>,
        pub displaytitle: String,
        pub iwlinks: Vec<Value>,
        pub properties: Vec<Property>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Text {
        #[serde(rename = "*")]
        pub field: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Category {
        pub sortkey: String,
        pub hidden: String,
        #[serde(rename = "*")]
        pub field: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Link {
        pub ns: i64,
        pub exists: String,
        #[serde(rename = "*")]
        pub field: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Template {
        pub ns: i64,
        pub exists: String,
        #[serde(rename = "*")]
        pub field: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Section {
        pub toclevel: i64,
        pub level: String,
        pub line: String,
        pub number: String,
        pub index: String,
        pub fromtitle: String,
        pub byteoffset: i64,
        pub anchor: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Property {
        pub name: String,
        #[serde(rename = "*")]
        pub field: String,
    }
}
