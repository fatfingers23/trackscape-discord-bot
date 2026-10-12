use crate::helpers::{handle_response, query_params_to_string};

use crate::models::global_types::{GroupId, GroupName};
use crate::models::group::{Group, GroupDetail};
use crate::models::name::NameChange;
use crate::{ApiEndpoint, Pagination, QueryParams};

enum GroupEndPoints {
    Search,
    GetGroupDetails(GroupId),
    // CreateGroup,
    GroupNameChanges(GroupId),
}

impl GroupEndPoints {
    fn url(&self) -> String {
        match self {
            GroupEndPoints::Search => ApiEndpoint::Group.as_str().to_string(),
            GroupEndPoints::GetGroupDetails(id) => {
                format!("{}/{}", ApiEndpoint::Group.as_str(), id)
            }
            // GroupEndPoints::CreateGroup => ApiEndpoint::Group.as_str().to_string(),
            GroupEndPoints::GroupNameChanges(id) => {
                format!("{}/{}/name-changes", ApiEndpoint::Group.as_str(), id)
            }
        }
    }
}

/// Handles all requests to the [Group Endpoints](https://docs.wiseoldman.net/groups-api/group-endpoints)
pub struct GroupClient {
    client: reqwest::Client,
    base_url: String,
}

impl GroupClient {
    pub fn new(client: reqwest::Client, base_url: &str) -> Self {
        Self {
            client,
            base_url: base_url.to_string(),
        }
    }

    fn get_url(&self, endpoint: GroupEndPoints, query_params: Option<QueryParams>) -> String {
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

    /// Search for groups by groupname, takes an optional pagination parameter
    /// [Search Groups](https://docs.wiseoldman.net/groups-api/group-endpoints#search-groups)
    pub async fn search(
        &self,
        name: GroupName,
        pagination: Option<Pagination>,
    ) -> anyhow::Result<Vec<Group>, anyhow::Error> {
        let mut queries = Vec::new();
        if let Some(pagination) = pagination {
            queries.extend(pagination.to_query())
        }
        queries.push(("name".to_string(), name.to_string()));
        let full_url = self.get_url(GroupEndPoints::Search, Some(queries));
        let result = self.client.get(full_url.as_str()).send().await;
        handle_response(result).await
    }

    /// Get group details by group id
    /// [Get Group Details](https://docs.wiseoldman.net/groups-api/group-endpoints#get-group-details)
    pub async fn get_group_details(
        &self,
        group_id: GroupId,
    ) -> anyhow::Result<GroupDetail, anyhow::Error> {
        let full_url = self.get_url(GroupEndPoints::GetGroupDetails(group_id), None);
        let result = self.client.get(full_url.as_str()).send().await;
        handle_response(result).await
    }

    /// Get group name changes by group id
    /// [Get Group Name Changes](https://docs.wiseoldman.net/groups-api/group-endpoints#get-group-name-changes)
    pub async fn get_group_name_changes(
        &self,
        group_id: GroupId,
        pagination: Option<Pagination>,
    ) -> anyhow::Result<Vec<NameChange>, anyhow::Error> {
        let mut queries = Vec::new();
        if let Some(pagination) = pagination {
            queries.extend(pagination.to_query())
        }
        let full_url = self.get_url(GroupEndPoints::GroupNameChanges(group_id), Some(queries));
        let result = self.client.get(full_url.as_str()).send().await;
        handle_response(result).await
    }

    // Needs further testing
    // Create a new group
    // [Create Group](https://docs.wiseoldman.net/groups-api/group-endpoints#create-group)
    // pub async fn create_group(
    //     &self,
    //     create_group: CreateGroupRequest,
    // ) -> anyhow::Result<GroupCreateResponse, anyhow::Error> {
    //     unimplemented!();
    //     let full_url = self.get_url(GroupEndPoints::CreateGroup, None);
    //     let result = self
    //         .client
    //         .post(full_url.as_str())
    //         .json(&create_group)
    //         .send()
    //         .await;
    //     handle_response(result).await
    // }
}

#[cfg(test)]
mod tests {
    use crate::{Pagination, WomClient};
    use httpmock::prelude::*;

    const BASE_URL: &str = "/groups";
    const CONTENT_TYPE: &str = "content-type";
    const APPLICATION_JSON: &str = "application/json";

    #[tokio::test]
    async fn group_search_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path(BASE_URL).query_param_exists("name");
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/group/group_search.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .group_client
            .search("A Cool Group Name".to_string(), None)
            .await;

        mock.assert();
        assert!(result.is_ok());
        let players = result.unwrap();
        assert_eq!(players.len(), 2);
    }

    #[tokio::test]
    async fn get_group_by_id_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path(format!("{}/1", BASE_URL));
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/group/group_detail.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client.group_client.get_group_details(1).await;
        println!("{:?}", result);
        mock.assert();
        assert!(result.is_ok());
    }

    // #[tokio::test]
    // async fn create_group_test() {
    //     let server = MockServer::start();
    //     let mock = server.mock(|when, then| {
    //         when.method(POST).path(BASE_URL);
    //         then.status(200)
    //             .header(CONTENT_TYPE, APPLICATION_JSON)
    //             .body_from_file("./tests/mocks/group/group_create_response.json");
    //     });
    //
    //     let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
    //     let result = wom_client
    //         .group_client
    //         .create_group(CreateGroupRequest::new("A Cool Group Name".to_string()))
    //         .await;
    //
    //     mock.assert();
    //     assert!(result.is_ok());
    // }

    #[tokio::test]
    async fn get_group_name_changes_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/1/name-changes", BASE_URL));
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/group/group_name_changes.json");
        });

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .group_client
            .get_group_name_changes(1, None)
            .await;

        mock.assert();
        assert!(result.is_ok());
        let name_changes = result.unwrap();
        assert_eq!(name_changes.len(), 2);
    }

    #[tokio::test]
    async fn get_group_name_changes_with_pagination_test() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}/1/name-changes", BASE_URL))
                .query_param("offset", "1")
                .query_param("limit", "10");
            then.status(200)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body_from_file("./tests/mocks/group/group_name_changes.json");
        });

        let pagination = Pagination {
            offset: Option::from(1),
            limit: Option::from(10),
        };

        let wom_client = WomClient::new_with_base_url(server.base_url().to_string(), None);
        let result = wom_client
            .group_client
            .get_group_name_changes(1, Some(pagination))
            .await;

        mock.assert();
        assert!(result.is_ok());
        let name_changes = result.unwrap();
        assert_eq!(name_changes.len(), 2);
    }
}
