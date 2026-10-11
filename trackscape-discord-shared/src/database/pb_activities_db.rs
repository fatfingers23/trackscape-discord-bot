use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::PersonalBestActivitiesDb;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonalBestActivitiesModel {
    // The dashboard reads activities as `_id`, so the JSON key stays `_id`.
    #[serde(rename = "_id")]
    pub id: Uuid,
    pub activity_name: String,
    pub created_at: DateTime<Utc>,
}

impl PersonalBestActivitiesModel {
    pub const COLLECTION_NAME: &'static str = "personal_best_activities";
}

struct PersonalBestActivitiesRow {
    id: Uuid,
    activity_name: String,
    created_at: DateTime<Utc>,
}

impl From<PersonalBestActivitiesRow> for PersonalBestActivitiesModel {
    fn from(row: PersonalBestActivitiesRow) -> Self {
        Self {
            id: row.id,
            activity_name: row.activity_name,
            created_at: row.created_at,
        }
    }
}

impl PersonalBestActivitiesDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_or_get_activity(
        &self,
        activity_name: String,
    ) -> Result<PersonalBestActivitiesModel, sqlx::Error> {
        let trimmed_activity_name = activity_name.trim();

        // Matches case insensitively on the exact name (the unique index on
        // lower(activity_name) backs this up).
        let existing = sqlx::query_as!(
            PersonalBestActivitiesRow,
            r#"
            SELECT id, activity_name, created_at
            FROM personal_best_activities
            WHERE lower(activity_name) = lower($1)
            "#,
            trimmed_activity_name
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(activity) = existing {
            return Ok(activity.into());
        }

        let new_activity = PersonalBestActivitiesModel {
            id: Uuid::new_v4(),
            activity_name: trimmed_activity_name.to_string(),
            created_at: Utc::now(),
        };
        // ON CONFLICT DO NOTHING guards against a concurrent create racing
        // us to the same name; the row already there is returned either way.
        sqlx::query!(
            r#"
            INSERT INTO personal_best_activities (id, activity_name, created_at)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
            new_activity.id,
            new_activity.activity_name,
            new_activity.created_at,
        )
        .execute(&self.pool)
        .await?;

        let saved = sqlx::query_as!(
            PersonalBestActivitiesRow,
            r#"
            SELECT id, activity_name, created_at
            FROM personal_best_activities
            WHERE lower(activity_name) = lower($1)
            "#,
            new_activity.activity_name
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(match saved {
            Some(activity) => activity.into(),
            // The insert raced and lost, but the follow up read found
            // nothing: fall back to the model we tried to insert.
            None => new_activity,
        })
    }

    pub async fn get_activities(&self) -> Result<Vec<PersonalBestActivitiesModel>, sqlx::Error> {
        let rows = sqlx::query_as!(
            PersonalBestActivitiesRow,
            r#"
            SELECT id, activity_name, created_at
            FROM personal_best_activities
            WHERE activity_name <> ''
            ORDER BY activity_name ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|row| row.into()).collect())
    }
}
