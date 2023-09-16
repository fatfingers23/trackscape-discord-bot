use crate::dto::bot_info_dto::DiscordServerCount;
use tracing::error;

pub struct ApiWebClient {
    base_url: String,
    auth_token: String,
    web_client: reqwest::Client,
}

const HEADER_AUTH_KEY: &str = "api-key";

impl ApiWebClient {
    pub fn new(base_url: String, auth_token: String) -> Self {
        let client = reqwest::Client::new();
        ApiWebClient {
            base_url,
            auth_token,
            web_client: reqwest::Client::new(),
        }
    }

    pub async fn send_server_count(&self, server_count: i64) {
        let discord_server_count = DiscordServerCount { server_count };

        let resp = self
            .web_client
            .post(format!("{}{}", self.base_url, "/api/info/set-server-count").as_str())
            .header(HEADER_AUTH_KEY, self.auth_token.clone())
            .json(&discord_server_count)
            .send()
            .await;

        if resp.is_err() {
            error!(
                "Error sending message to api: {}",
                resp.err().expect(
                    "Error getting a error from the error for an api call for new discord chat"
                )
            );
        }
    }
}
