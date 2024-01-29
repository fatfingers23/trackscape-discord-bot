mod responses;

use crate::wom::responses::Player;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Error, Response};
use serde::de::DeserializeOwned;

const BASE_URL: &str = "https://api.wiseoldman.net/v2";

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    "/",
    "GitHub:https://github.com/fatfingers23/trackscape-discord-bot"
);

enum ApiEndpoints {
    Player,
}

impl ApiEndpoints {
    fn to_string(&self) -> String {
        match self {
            ApiEndpoints::Player => format!("{}/players", BASE_URL),
        }
    }
}

pub struct Pagination {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}
pub struct PlayerSearch {
    pub username: String,
    pub pagination: Option<Pagination>,
}

type Username = String;
enum PlayerEndPoints {
    Search(Username),
    Update,
    AssertType,
    Details,
    DetailsById,
    Achievements,
    AchievementsProgress,
    Competitions,
    CompetitionsStandings,
    GroupMembership,
    Gains,
    Records,
    Snapshots,
    SnapshotsTimeline,
    NameChange,
    Archives,
}

impl PlayerEndPoints {
    fn url(&self) -> String {
        match self {
            PlayerEndPoints::Search(username) => {
                format!(
                    "{}/search?username={}",
                    ApiEndpoints::Player.to_string(),
                    username
                )
            }
            _ => format!("{}", ApiEndpoints::Player.to_string()),
        }
    }
}

pub struct Client {
    pub player_client: PlayerClient,
    client: reqwest::Client,
}

impl Client {
    fn new_client(api_key: Option<String>) -> reqwest::Client {
        let client = reqwest::Client::builder().user_agent(APP_USER_AGENT);
        match api_key {
            Some(key) => {
                let mut headers = HeaderMap::new();
                headers.insert("api-key", HeaderValue::from_str(&*key).unwrap());
                client.default_headers(headers)
            }
            None => client,
        }
        .build()
        .unwrap()
    }
    pub fn new(api_key: Option<String>) -> Self {
        let client = self::Client::new_client(api_key);
        Self {
            player_client: PlayerClient::new(client.clone()),
            client,
        }
    }

    pub fn pagination_to_query(pagination: Option<Pagination>) -> String {
        match pagination {
            Some(p) => format!(
                "limit={}&offset={}",
                p.limit.unwrap_or(20),
                p.offset.unwrap_or(0)
            ),
            None => "".to_string(),
        }
    }

    pub async fn handle_response<ResponseType: DeserializeOwned>(
        response: Result<Response, Error>,
    ) -> Result<ResponseType, Error> {
        match response {
            Ok(result) => {
                let body = result.json::<ResponseType>().await;
                match body {
                    Ok(body) => Ok(body),
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(err),
        }
    }
}

pub struct PlayerClient {
    client: reqwest::Client,
}

type ResponseType = Vec<Player>;
impl PlayerClient {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn search(
        &self,
        username: Username,
        pagination: Option<Pagination>,
    ) -> Result<Vec<Player>, Error> {
        let pagination_query = Client::pagination_to_query(pagination);
        let full_url = format!(
            "{}{}",
            PlayerEndPoints::Search(username).url(),
            pagination_query
        );
        let result = self.client.get(full_url.as_str()).send().await;
        Client::handle_response::<ResponseType>(result).await
    }
}
