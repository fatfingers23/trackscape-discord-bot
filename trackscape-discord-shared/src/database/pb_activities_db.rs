use bson::serde_helpers::serialize_object_id_as_hex_string;
use bson::DateTime;
use futures::TryStreamExt;
use mockall::predicate::*;
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use mongodb::{bson, Database};
use serde::{Deserialize, Serialize};

use super::PersonalBestActivitiesDb;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonalBestActivitiesModel {
    #[serde(rename = "_id")]
    #[serde(serialize_with = "serialize_object_id_as_hex_string")]
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
        let trimmed_activity_name = activity_name.trim();
        let collection = self.db.collection::<PersonalBestActivitiesModel>(
            PersonalBestActivitiesModel::COLLECTION_NAME,
        );

        //Matches case insensitive and exact name from start and end
        let name_filter = format!("^{}$", trimmed_activity_name);
        let re = mongodb::bson::Regex {
            pattern: name_filter,
            options: "i".to_string(),
        };

        let filter = doc! {
            "activity_name": re
        };

        match collection.find_one(filter.clone(), None).await? {
            Some(activity) => {
                return Ok(activity);
            }
            None => {
                let new_activity = PersonalBestActivitiesModel {
                    id: bson::oid::ObjectId::new(),
                    activity_name: trimmed_activity_name.to_string(),
                    created_at: bson::DateTime::now(),
                };
                collection.insert_one(new_activity.clone(), None).await?;
                return Ok(new_activity);
            }
        }
    }

    pub async fn get_activities(
        &self,
    ) -> Result<Vec<PersonalBestActivitiesModel>, mongodb::error::Error> {
        let collection = self.db.collection::<PersonalBestActivitiesModel>(
            PersonalBestActivitiesModel::COLLECTION_NAME,
        );

        let opts = FindOptions::builder()
            .sort(doc! { "activity_name": 1 })
            .build();
        let filter = doc! {"activity_name": {"$ne": ""}};
        let cursor = collection.find(filter, opts).await?;
        let result = cursor.try_collect().await;
        Ok(result.unwrap())
    }
}
