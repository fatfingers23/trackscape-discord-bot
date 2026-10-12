use crate::helpers::{handle_response, query_params_to_string};
use crate::models::competition::{
    CompetitionStatus, PlayerCompetitionStanding, PlayerParticipation,
};
use crate::models::global_enums::{Metric, Period};
use crate::models::global_types::{PlayerId, Username};
use crate::models::group::PlayerMembership;
use crate::models::name::NameChange;
use crate::models::player::{
    Achievement, AchievementProgress, AssertPlayerType, Player, PlayerArchive, PlayerDetails,
    PlayerGain, SnapShot, TimelineDatapoint,
};
use crate::models::record::Record;
use crate::{ApiEndpoint, Pagination, QueryParam, QueryParams};
use anyhow::Result;
use chrono::{DateTime, Utc};

enum PlayerEndPoints {
    Search,
    Update(Username),
    AssertType(Username),
    Details(Username),
    DetailsById(PlayerId),
    Achievements(Username),
    AchievementsProgress(Username),
    Competitions(Username),
    CompetitionsStandings(Username),
    GroupMembership(Username),
    Gains(Username),
    Records(Username),
    Snapshots(Username),
    SnapshotsTimeline(Username),
    NameChange(Username),
    Archives(Username),
}

impl PlayerEndPoints {
    fn url(&self) -> String {
        match self {
            PlayerEndPoints::Search => {
                format!("{}/search", ApiEndpoint::Player.as_str())
            }
            PlayerEndPoints::Update(username) => {
                format!("{}/{}", ApiEndpoint::Player.as_str(), username)
            }
            PlayerEndPoints::AssertType(username) => {
                format!("{}/{}/assert-type", ApiEndpoint::Player.as_str(), username)
            }
            PlayerEndPoints::Details(username) => {
                format!("{}/{}", ApiEndpoint::Player.as_str(), username)
            }
            PlayerEndPoints::DetailsById(player_id) => {
                format!("{}/id/{}", ApiEndpoint::Player.as_str(), player_id)
            }
            PlayerEndPoints::Achievements(username) => {
                format!("{}/{}/achievements", ApiEndpoint::Player.as_str(), username)
            }
            PlayerEndPoints::AchievementsProgress(username) => {
                format!(
                    "{}/{}/achievements/progress",
                    ApiEndpoint::Player.as_str(),
                    username
                )
            }
            PlayerEndPoints::Competitions(username) => {
                format!("{}/{}/competitions", ApiEndpoint::Player.as_str(), username)
            }
            PlayerEndPoints::CompetitionsStandings(username) => {
                format!(
                    "{}/{}/competitions/standings",
                    ApiEndpoint::Player.as_str(),
                    username
                )
            }
            PlayerEndPoints::GroupMembership(username) => {
                format!("{}/{}/groups", ApiEndpoint::Player.as_str(), username)
            }
            PlayerEndPoints::Gains(username) => {
                format!("{}/{}/gained", ApiEndpoint::Player.as_str(), username)
            }
            PlayerEndPoints::Records(username) => {
                format!("{}/{}/records", ApiEndpoint::Player.as_str(), username)
            }
            PlayerEndPoints::Snapshots(username) => {
                format!("{}/{}/snapshots", ApiEndpoint::Player.as_str(), username)
            }
            PlayerEndPoints::SnapshotsTimeline(username) => {
                format!(
                    "{}/{}/snapshots/timeline",
                    ApiEndpoint::Player.as_str(),
                    username
                )
            }
            PlayerEndPoints::NameChange(username) => {
                format!("{}/{}/names", ApiEndpoint::Player.as_str(), username)
            }
            PlayerEndPoints::Archives(username) => {
                format!("{}/{}/archives", ApiEndpoint::Player.as_str(), username)
            }
        }
    }
}

/// Handles all requests to the [Player Endpoints](https://docs.wiseoldman.net/players-api/player-endpoints)
pub struct PlayerClient {
    client: reqwest::Client,
    base_url: String,
}

impl PlayerClient {
    pub fn new(client: reqwest::Client, base_url: &str) -> Self {
        Self {
            client,
            base_url: base_url.to_string(),
        }
    }

    fn get_url(&self, endpoint: PlayerEndPoints, query_params: Option<QueryParams>) -> String {
        let base_url_with_endpoint = format!("{}{}", self.base_url, endpoint.url());
        match query_params {
            Some(params) => format!(
                "{}{}",
                base_url_with_endpoint,
                query_params_to_string(&params)
            ),
            None => base_url_with_endpoint,
        }
    }

    /// Search for players by username, takes an optional pagination parameter
    /// [Player Search](https://docs.wiseoldman.net/players-api/player-endpoints#search)
    pub async fn search(
        &self,
        username: Username,
        pagination: Option<Pagination>,
    ) -> Result<Vec<Player>, anyhow::Error> {
        let mut queries = Vec::new();
        if let Some(pagination) = pagination {
            queries.extend(pagination.to_query())
        }
        let username_query: QueryParam = ("username".to_string(), username.to_string());
        queries.push(username_query);

        let full_url = self.get_url(PlayerEndPoints::Search, Some(queries));
        let result = self.client.get(full_url.as_str()).send().await;
        handle_response(result).await
    }

    /// Sends a request to update the players hiscore data from the offical hiscores
    /// [Player Update](https://docs.wiseoldman.net/players-api/player-endpoints#update-a-player)
    pub async fn update(&self, username: Username) -> Result<PlayerDetails, anyhow::Error> {
        let full_url = self.get_url(PlayerEndPoints::Update(username), None);
        let result = self.client.post(full_url.as_str()).send().await;
        handle_response(result).await
    }

    /// Asserts (and attempts to fix, if necessary) a player's game-mode type.
    /// [Assert Player Type](https://docs.wiseoldman.net/players-api/player-endpoints#assert-player-type)
    pub async fn assert_type(&self, username: Username) -> Result<AssertPlayerType, anyhow::Error> {
        let result = self
            .client
            .post(
                self.get_url(PlayerEndPoints::AssertType(username), None)
                    .as_str(),
            )
            .send()
            .await;
        handle_response(result).await
    }

    /// Get a player's details by username
    /// [Player Details](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-details)
    pub async fn get_details(&self, username: Username) -> Result<PlayerDetails, anyhow::Error> {
        let result = self
            .client
            .get(
                self.get_url(PlayerEndPoints::Details(username), None)
                    .as_str(),
            )
            .send()
            .await;
        handle_response(result).await
    }

    /// Get a player's details by player id
    /// [Player Details](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-details-by-id)
    pub async fn get_details_by_id(
        &self,
        player_id: PlayerId,
    ) -> Result<PlayerDetails, anyhow::Error> {
        let result = self
            .client
            .get(
                self.get_url(PlayerEndPoints::DetailsById(player_id), None)
                    .as_str(),
            )
            .send()
            .await;
        handle_response(result).await
    }

    /// Get a player's achievements by username
    /// [Player Achievements](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-achievements)
    pub async fn get_achievements(
        &self,
        username: Username,
    ) -> Result<Vec<Achievement>, anyhow::Error> {
        let result = self
            .client
            .get(
                self.get_url(PlayerEndPoints::Achievements(username), None)
                    .as_str(),
            )
            .send()
            .await;
        handle_response(result).await
    }

    /// Get a player's achievements progress by username
    /// [Player Achievements Progress](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-achievement-progress)
    pub async fn get_achievement_progress(
        &self,
        username: Username,
    ) -> Result<Vec<AchievementProgress>, anyhow::Error> {
        let result = self
            .client
            .get(
                self.get_url(PlayerEndPoints::AchievementsProgress(username), None)
                    .as_str(),
            )
            .send()
            .await;
        handle_response(result).await
    }

    /// Get a player's competitions they have participated in by username
    ///  [Get Player Competition Participations](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-competition-participations)
    pub async fn get_competitions(
        &self,
        username: Username,
        competition_status: Option<CompetitionStatus>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<PlayerParticipation>, anyhow::Error> {
        let mut queries = Vec::new();
        if let Some(pagination) = pagination {
            queries.extend(pagination.to_query())
        }
        if let Some(status) = competition_status {
            queries.push(("status".to_string(), status.as_str().to_string()));
        }

        let full_url = format!(
            "{}{}",
            self.get_url(PlayerEndPoints::Competitions(username), None),
            query_params_to_string(&queries)
        );

        let result = self.client.get(full_url).send().await;
        handle_response(result).await
    }

    /// Get a player's competition standings by username
    /// [Get Player Competition Standings](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-competition-standings)
    pub async fn get_competition_standings(
        &self,
        username: Username,
        competition_status: CompetitionStatus,
    ) -> Result<Vec<PlayerCompetitionStanding>, anyhow::Error> {
        let full_url = format!(
            "{}{}",
            self.get_url(PlayerEndPoints::CompetitionsStandings(username), None),
            query_params_to_string(&vec![(
                "status".to_string(),
                competition_status.as_str().to_string()
            )])
        );
        let result = self.client.get(full_url).send().await;
        handle_response(result).await
    }

    /// Get a player's group memberships by username
    /// [Get Player Group Memberships](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-group-memberships)
    pub async fn get_groups(
        &self,
        username: Username,
        pagination: Option<Pagination>,
    ) -> Result<Vec<PlayerMembership>, anyhow::Error> {
        let mut queries = Vec::new();
        if let Some(pagination) = pagination {
            queries.extend(pagination.to_query())
        }

        let full_url = format!(
            "{}{}",
            self.get_url(PlayerEndPoints::GroupMembership(username), None),
            query_params_to_string(&queries)
        );

        let result = self.client.get(full_url).send().await;
        handle_response(result).await
    }

    /// Get a player's gains by username and period
    /// [Get Player Gains](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-gains)
    pub async fn get_gains_by_period(
        &self,
        username: Username,
        period: Period,
    ) -> Result<PlayerGain, anyhow::Error> {
        let full_url = self.get_url(
            PlayerEndPoints::Gains(username),
            Some(vec![("period".to_string(), period.as_str().to_string())]),
        );
        let result = self.client.get(full_url).send().await;
        handle_response(result).await
    }

    /// Get a player's gains by username plus start and end date
    /// [Get Player Gains](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-gains)
    pub async fn get_gains_by_date(
        &self,
        username: Username,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<PlayerGain, anyhow::Error> {
        let full_url = self.get_url(
            PlayerEndPoints::Gains(username),
            Some(vec![
                (
                    "startDate".to_string(),
                    start_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                ),
                (
                    "endDate".to_string(),
                    end_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                ),
            ]),
        );

        let result = self.client.get(full_url).send().await;
        handle_response(result).await
    }

    /// Get a player's records by username
    /// [Get Player Records](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-records)
    pub async fn get_records(
        &self,
        username: Username,
        period: Option<Period>,
        metric: Option<Metric>,
    ) -> Result<Vec<Record>, anyhow::Error> {
        let mut queries = Vec::new();
        if let Some(period) = period.clone() {
            queries.push(("period".to_string(), period.as_str().to_string()));
        }
        if let Some(metric) = metric.clone() {
            queries.push(("metric".to_string(), metric.to_string()));
        }

        let result = self
            .client
            .get(self.get_url(PlayerEndPoints::Records(username), Some(queries)))
            .send()
            .await;
        handle_response(result).await
    }

    /// Get a player's snapshots by username and within a period
    /// [Get Player Snapshots](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-snapshots)
    pub async fn get_snapshots_by_period(
        &self,
        username: Username,
        period: Period,
    ) -> Result<Vec<SnapShot>, anyhow::Error> {
        let result = self
            .client
            .get(
                self.get_url(
                    PlayerEndPoints::Snapshots(username),
                    Some(vec![("period".to_string(), period.as_str().to_string())]),
                )
                .as_str(),
            )
            .send()
            .await;
        handle_response(result).await
    }

    /// Get a player's snapshot by username with a start and end date
    /// [Get Player Snapshots](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-snapshots)
    pub async fn get_snapshots_by_date(
        &self,
        username: Username,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<SnapShot>, anyhow::Error> {
        let full_url = self.get_url(
            PlayerEndPoints::Snapshots(username),
            Some(vec![
                (
                    "startDate".to_string(),
                    start_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                ),
                (
                    "endDate".to_string(),
                    end_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                ),
            ]),
        );

        let result = self.client.get(full_url).send().await;
        handle_response(result).await
    }

    /// Get a player's snapshots timeline by username and within a period
    /// Get Player Snapshots Timeline
    /// [Get Player Snapshots Timeline](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-snapshots-timeline)
    pub async fn get_snapshots_timeline(
        &self,
        username: Username,
        metric: Metric,
        period: Period,
    ) -> Result<Vec<TimelineDatapoint>, anyhow::Error> {
        let result = self
            .client
            .get(
                self.get_url(
                    PlayerEndPoints::SnapshotsTimeline(username),
                    Some(vec![
                        ("period".to_string(), period.as_str().to_string()),
                        ("metric".to_string(), metric.to_string()),
                    ]),
                )
                .as_str(),
            )
            .send()
            .await;
        handle_response(result).await
    }

    ///Gets a player's snapshots timeline by username with a start and end date
    /// [Get Player Snapshots Timeline](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-snapshots-timeline)
    pub async fn get_snapshots_timeline_by_date(
        &self,
        username: Username,
        metric: Metric,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<TimelineDatapoint>, anyhow::Error> {
        let full_url = self.get_url(
            PlayerEndPoints::SnapshotsTimeline(username),
            Some(vec![
                (
                    "startDate".to_string(),
                    start_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                ),
                (
                    "endDate".to_string(),
                    end_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                ),
                ("metric".to_string(), metric.to_string()),
            ]),
        );

        let result = self.client.get(full_url).send().await;
        handle_response(result).await
    }

    /// Get a player's name changes by username
    /// [Get Player Name Changes](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-name-changes)
    pub async fn get_name_changes(
        &self,
        username: Username,
    ) -> Result<Vec<NameChange>, anyhow::Error> {
        let result = self
            .client
            .get(
                self.get_url(PlayerEndPoints::NameChange(username), None)
                    .as_str(),
            )
            .send()
            .await;
        handle_response::<Vec<NameChange>>(result).await
    }

    /// Get a player's archives by username
    /// [Get Player Archives](https://docs.wiseoldman.net/players-api/player-endpoints#get-player-archives)
    pub async fn get_archives(
        &self,
        username: Username,
    ) -> Result<Vec<PlayerArchive>, anyhow::Error> {
        let result = self
            .client
            .get(
                self.get_url(PlayerEndPoints::Archives(username), None)
                    .as_str(),
            )
            .send()
            .await;
        handle_response(result).await
    }
}

#[cfg(test)]
mod tests {
    use crate::models::competition::CompetitionStatus;
    use crate::models::global_enums::Skill::Overall;
    use crate::models::global_enums::{Metric, Period};
    use crate::{Pagination, WomClient};
    use chrono::TimeZone;
    use httpmock::prelude::*;

    const BASE_URL: &str = "/players";
    const CONTENT_TYPE: &str = "content-type";
    const APPLICATION_JSON: &str = "application/json";

    #[tokio::test]
    async fn player_search_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/search", BASE_URL))
                .query_param_exists("username");
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_search.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .search("Zezima".to_string(), None)
            .await;

        mock.assert();
        assert!(result.is_ok());
        let players = result.unwrap();
        assert_eq!(players.len(), 2);
    }

    #[tokio::test]
    async fn player_search_check_pagination_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/search", BASE_URL))
                .query_param("limit", "10")
                .query_param("offset", "10")
                .query_param_exists("username");
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_search.json");
        });

        let pagination = Some(Pagination {
            limit: Some(10),
            offset: Some(10),
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .search("Zezima".to_string(), pagination)
            .await;

        mock.assert();
        assert!(result.is_ok());
        let players = result.unwrap();
        assert_eq!(players.len(), 2);
    }

    #[tokio::test]
    async fn player_update_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path(format!("{}/Zezima", BASE_URL));
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_details.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client.player_client.update("Zezima".to_string()).await;
        mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn player_assert_type_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("{}/Zezima/assert-type", BASE_URL));
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_assert_type.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .assert_type("Zezima".to_string())
            .await;
        mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn player_details_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers", BASE_URL));
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_details.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .get_details("IFat Fingers".to_string())
            .await;

        mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn player_details_by_id_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path(format!("{}/id/1", BASE_URL));
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_details.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client.player_client.get_details_by_id(1).await;

        mock.assert();
        assert!(result.is_ok());
        let player_details = result.unwrap();
        assert_eq!(player_details.id, 1);
    }

    #[tokio::test]
    async fn player_achievements_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/achievements", BASE_URL));
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_achievements.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .get_achievements("IFat Fingers".to_string())
            .await;

        mock.assert();
        assert!(result.is_ok());
        let achievements = result.unwrap();
        assert_eq!(achievements.len(), 2);
    }

    #[tokio::test]
    async fn player_achievements_progress_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/achievements/progress", BASE_URL));
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_achievement_progress.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .get_achievement_progress("IFat Fingers".to_string())
            .await;

        mock.assert();
        assert!(result.is_ok());
        let achievements_progress = result.unwrap();
        assert_eq!(achievements_progress.len(), 4);
    }

    #[tokio::test]
    async fn player_competitions_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/competitions", BASE_URL));
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_competition_participation.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .get_competitions("IFat Fingers".to_string(), None, None)
            .await;
        mock.assert();
        assert!(result.is_ok());
        let competitions = result.unwrap();
        assert_eq!(competitions.len(), 2);
    }

    #[tokio::test]
    async fn player_competitions_with_params_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/competitions", BASE_URL))
                .query_param("limit", "10")
                .query_param("offset", "10")
                .query_param("status", "finished");
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_competition_participation.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .get_competitions(
                "IFat Fingers".to_string(),
                Some(CompetitionStatus::Finished),
                Some(Pagination {
                    limit: Some(10),
                    offset: Some(10),
                }),
            )
            .await;
        mock.assert();
        assert!(result.is_ok());
        let competitions = result.unwrap();
        assert_eq!(competitions.len(), 2);
    }

    #[tokio::test]
    async fn player_competitions_with_only_status_params_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/competitions", BASE_URL))
                .query_param("status", "finished");
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_competition_participation.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .get_competitions(
                "IFat Fingers".to_string(),
                Some(CompetitionStatus::Finished),
                None,
            )
            .await;
        mock.assert();
        assert!(result.is_ok());
        let competitions = result.unwrap();
        assert_eq!(competitions.len(), 2);
    }

    #[tokio::test]
    async fn player_competitions_with_only_pagination_params_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/competitions", BASE_URL))
                .query_param("limit", "10")
                .query_param("offset", "10");

            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_competition_participation.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .get_competitions(
                "IFat Fingers".to_string(),
                None,
                Some(Pagination {
                    limit: Some(10),
                    offset: Some(10),
                }),
            )
            .await;
        mock.assert();
        assert!(result.is_ok());
        let competitions = result.unwrap();
        assert_eq!(competitions.len(), 2);
    }

    #[tokio::test]
    async fn player_competitions_standings_with_params_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!(
                    "{}/IFat%20Fingers/competitions/standings",
                    BASE_URL
                ))
                .query_param("status", "finished");
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_competition_standings.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .get_competition_standings("IFat Fingers".to_string(), CompetitionStatus::Finished)
            .await;
        mock.assert();
        assert!(result.is_ok());
        let competitions = result.unwrap();
        assert_eq!(competitions.len(), 1);
    }

    #[tokio::test]
    async fn player_group_membership_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/groups", BASE_URL));
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_group_membership.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);

        let result = wom_client
            .player_client
            .get_groups("IFat Fingers".to_string(), None)
            .await;

        mock.assert();
        assert!(result.is_ok());
        let groups = result.unwrap();
        assert_eq!(groups.len(), 2);
    }

    #[tokio::test]
    async fn player_gains_by_period_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/gained", BASE_URL))
                .query_param("period", "week");
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_gains.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);

        let result = wom_client
            .player_client
            .get_gains_by_period("IFat Fingers".to_string(), Period::Week)
            .await;
        mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_player_records_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/records", BASE_URL));
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_records.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);

        let result = wom_client
            .player_client
            .get_records("IFat Fingers".to_string(), None, None)
            .await;
        mock.assert();
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 4);
    }

    #[tokio::test]
    async fn get_player_records_with_params_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/records", BASE_URL))
                .query_param("period", "week")
                .query_param("metric", "overall");
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_records.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);

        let result = wom_client
            .player_client
            .get_records(
                "IFat Fingers".to_string(),
                Some(Period::Week),
                Some(Metric::Skill(Overall)),
            )
            .await;
        mock.assert();
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 4);
    }

    #[tokio::test]
    async fn player_get_snapshots_by_period_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/snapshots", BASE_URL))
                .query_param("period", "week");
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_snapshots.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .get_snapshots_by_period("IFat Fingers".to_string(), Period::Week)
            .await;

        mock.assert();

        assert!(result.is_ok());
        let snapshots = result.unwrap();
        assert_eq!(snapshots.len(), 1);
    }

    #[tokio::test]
    async fn player_get_snapshots_by_date_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/snapshots", BASE_URL))
                .query_param("startDate", "2021-01-01 00:00:00")
                .query_param("endDate", "2021-01-07 00:00:00");
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_snapshots.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .get_snapshots_by_date(
                "IFat Fingers".to_string(),
                chrono::Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
                chrono::Utc.with_ymd_and_hms(2021, 1, 7, 0, 0, 0).unwrap(),
            )
            .await;

        mock.assert();

        assert!(result.is_ok());
        let snapshots = result.unwrap();
        assert_eq!(snapshots.len(), 1);
    }

    #[tokio::test]
    async fn player_get_snapshots_timeline_by_period_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/snapshots/timeline", BASE_URL))
                .query_param("metric", "overall")
                .query_param("period", "week");
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_snapshots_timeline.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .get_snapshots_timeline(
                "IFat Fingers".to_string(),
                Metric::Skill(Overall),
                Period::Week,
            )
            .await;

        mock.assert();
        assert!(result.is_ok());
        let snapshots = result.unwrap();
        assert_eq!(snapshots.len(), 3);
    }

    #[tokio::test]
    async fn player_get_snapshots_timeline_by_date_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/snapshots/timeline", BASE_URL))
                .query_param("startDate", "2021-01-01 00:00:00")
                .query_param("endDate", "2021-01-07 00:00:00")
                .query_param("metric", "overall");
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_snapshots_timeline.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .get_snapshots_timeline_by_date(
                "IFat Fingers".to_string(),
                Metric::Skill(Overall),
                chrono::Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
                chrono::Utc.with_ymd_and_hms(2021, 1, 7, 0, 0, 0).unwrap(),
            )
            .await;

        mock.assert();
        assert!(result.is_ok());
        let snapshots = result.unwrap();
        assert_eq!(snapshots.len(), 3);
    }

    #[tokio::test]
    async fn player_name_changes_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/names", BASE_URL));
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_name_changes.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .get_name_changes("IFat Fingers".to_string())
            .await;

        mock.assert();
        assert!(result.is_ok());
        result.unwrap();
    }

    #[tokio::test]
    async fn player_archives_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/IFat%20Fingers/archives", BASE_URL));
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/player/player_archives.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .player_client
            .get_archives("IFat Fingers".to_string())
            .await;

        println!("{:?}", result);
        mock.assert();
        assert!(result.is_ok());
        result.unwrap();
    }
}
