pub mod wiki_api {
    use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::QuestDifficulty;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    const BASE_URL: &str = "https://oldschool.runescape.wiki/api.php";

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

    fn build_quest_url(difficulty: QuestDifficulty) -> String {
        format!(
            "{}?format=json&action=parse&page=Quests%2F{}&section=1",
            BASE_URL,
            difficulty.to_string()
        )
    }

    pub async fn get_quests_and_difficulties() -> Result<Vec<WikiQuest>, reqwest::Error> {
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
                            return Err(e);
                        }
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(quests)
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
