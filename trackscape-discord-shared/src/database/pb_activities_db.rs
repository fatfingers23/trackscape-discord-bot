use bson::DateTime;
use mockall::predicate::*;
use mongodb::bson::doc;
use mongodb::{bson, Database};
use serde::{Deserialize, Serialize};

use super::PersonalBestActivitiesDb;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonalBestActivitiesModel {
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,
    pub activity_name: String,
    pub created_at: DateTime,
}

impl PersonalBestActivitiesModel {
    pub const COLLECTION_NAME: &'static str = "personal_best_activities";
}

impl PersonalBestActivitiesDb {
    pub fn new_instance(mongodb: Database) -> Self {
        Self { db: mongodb }
    }

    pub async fn create_or_get_activity(
        &self,
        activity_name: String,
    ) -> Result<PersonalBestActivitiesModel, mongodb::error::Error> {
        let collection = self.db.collection::<PersonalBestActivitiesModel>(
            PersonalBestActivitiesModel::COLLECTION_NAME,
        );

        let filter = doc! {
            "activity_name": activity_name.clone()
        };
        match collection.find_one(filter.clone(), None).await? {
            Some(activity) => {
                return Ok(activity);
            }
            None => {
                let new_activity = PersonalBestActivitiesModel {
                    id: bson::oid::ObjectId::new(),
                    activity_name: activity_name.clone(),
                    created_at: bson::DateTime::now(),
                };
                collection.insert_one(new_activity.clone(), None).await?;
                return Ok(new_activity);
            }
        }
    }
}
