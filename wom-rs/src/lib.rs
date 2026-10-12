use crate::clients::group_client::GroupClient;
use crate::clients::player_client::PlayerClient;
use reqwest::header::{HeaderMap, HeaderValue};

/// Individual clients for each endpoint
pub mod clients;

/// Responses for each endpoint
pub mod models;

const BASE_URL: &str = "https://api.wiseoldman.net/v2";

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    "/",
    "GitHub:https://github.com/fatfingers23/wom_rs"
);

///Wom endpoints
pub(crate) enum ApiEndpoint {
    Player,
    Group,
}

impl ApiEndpoint {
    pub fn as_str(&self) -> &str {
        match self {
            ApiEndpoint::Player => "/players",
            ApiEndpoint::Group => "/groups",
        }
    }
}

/// Wise Old Man Client
pub struct WomClient {
    pub player_client: PlayerClient,
    pub group_client: GroupClient,
}

/// Used for endpoints that take pagination
pub struct Pagination {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Query parameters for a api call
pub(crate) type QueryParams = Vec<QueryParam>;
/// Query parameter tuple
pub(crate) type QueryParam = (String, String);

impl Pagination {
    pub fn to_query(&self) -> QueryParams {
        let limit = ("limit".to_string(), self.limit.unwrap_or(20).to_string());
        let offset = ("offset".to_string(), self.offset.unwrap_or(0).to_string());
        vec![limit, offset]
    }
}

impl WomClient {
    fn new_reqwest_client(api_key: Option<String>) -> reqwest::Client {
        let client = reqwest::Client::builder().user_agent(APP_USER_AGENT);
        match api_key {
            Some(key) => {
                let mut headers = HeaderMap::new();
                headers.insert("x-api-key", HeaderValue::from_str(&*key).unwrap());
                client.default_headers(headers)
            }
            None => client,
        }
        .build()
        .unwrap()
    }

    fn new_wom_client(client: reqwest::Client, base_url: String) -> Self {
        let _ = env_logger::try_init();

        let sub_client_base_url = base_url.clone();
        Self {
            player_client: PlayerClient::new(client.clone(), &*sub_client_base_url),
            group_client: GroupClient::new(client, &*sub_client_base_url),
        }
    }

    ///Creates a new `WOMClient`
    pub fn new() -> Self {
        let client = WomClient::new_reqwest_client(None);
        Self::new_wom_client(client, BASE_URL.to_string())
    }

    ///Creates a new `WOMClient` with an API key
    pub fn new_with_key(api_key: String) -> Self {
        let client = WomClient::new_reqwest_client(Some(api_key));
        Self::new_wom_client(client, BASE_URL.to_string())
    }

    ///Creates a new `WOMClient` with a custom base URL, and optionally an API key.
    /// Used for testing and self hosted/League WOM instances
    pub fn new_with_base_url(base_url: String, api_key: Option<String>) -> Self {
        let client = WomClient::new_reqwest_client(api_key);
        Self::new_wom_client(client, base_url)
    }
}

pub(crate) mod helpers {
    use crate::models::error::ErrorResponse;
    use anyhow::anyhow;
    use log::debug;
    use reqwest::{Error, Response, StatusCode};
    use serde::de::DeserializeOwned;

    pub async fn handle_response<ResponseType: DeserializeOwned>(
        response: Result<Response, Error>,
    ) -> Result<ResponseType, anyhow::Error> {
        match response {
            Ok(result) => match result.status() {
                StatusCode::OK => {
                    let body = result.json::<ResponseType>().await;
                    match body {
                        Ok(body) => Ok(body),
                        Err(err) => Err(anyhow!(err)),
                    }
                }
                StatusCode::NOT_FOUND => {
                    let error_body = result.json::<ErrorResponse>().await;
                    match error_body {
                        Ok(body) => Err(anyhow!(body.message)),
                        Err(err) => Err(anyhow!(err)),
                    }
                }
                StatusCode::BAD_REQUEST => {
                    debug!("{:?}", result);

                    let error_body = result.json::<ErrorResponse>().await;
                    match error_body {
                        Ok(body) => Err(anyhow!(body.message)),
                        Err(err) => Err(anyhow!(err)),
                    }
                }
                _ => {
                    debug!("{:?}", result);
                    let error_body = result.json::<ErrorResponse>().await;
                    match error_body {
                        Ok(body) => Err(anyhow!(body.message)),
                        Err(err) => Err(anyhow!(err)),
                    }
                }
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn _handle_empty_response(response: Result<Response, Error>) -> Result<(), Error> {
        match response {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub fn query_params_to_string(query_params: &Vec<(String, String)>) -> String {
        let mut query_string = String::new();
        for (index, (key, value)) in query_params.iter().enumerate() {
            if index == 0 {
                query_string.push_str(&format!("?{}={}", key, value));
            } else {
                query_string.push_str(&format!("&{}={}", key, value));
            }
        }
        query_string
    }
}

#[cfg(test)]
mod tests {}
