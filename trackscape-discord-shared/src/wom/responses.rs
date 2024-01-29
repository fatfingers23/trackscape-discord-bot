use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: i64,
    pub username: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub r#type: String,
    pub build: String,
    pub country: Option<String>,
    pub status: String,
    pub patron: bool,
    pub exp: i64,
    pub ehp: f64,
    pub ehb: f64,
    pub ttm: f64,
    pub tt200m: f64,
    #[serde(rename = "registeredAt")]
    pub registered_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "lastChangedAt")]
    pub last_changed_at: String,
    #[serde(rename = "lastImportedAt")]
    pub last_imported_at: Option<String>,
}
