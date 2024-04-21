use redis::{Commands, RedisResult};

use super::job_helpers::get_redis_client;

pub mod pb_command;

const RUNELITE_BASE_URL: &str = "https://api.runelite.net/runelite-";

pub async fn get_runelite_api_url() -> String {
    let version = get_runelite_version().await;
    format!("{}{}", RUNELITE_BASE_URL, version)
}

async fn get_runelite_version() -> String {
    let mut redis_connection = get_redis_client().unwrap();
    let version_key = "runelite_version";
    let exists: RedisResult<bool> = redis_connection.exists(version_key.clone());
    match exists {
        Ok(exist) => {
            if exist {
                redis_connection.get(version_key).unwrap()
            } else {
                let version = get_runelite_version_from_api().await;
                let _: RedisResult<String> =
                    redis_connection.set_ex(version_key, version.clone(), 3600);
                version
            }
        }
        Err(err) => {
            println!("Failed to check if key exists: {:?}", err);
            "".to_string()
        }
    }
}

async fn get_runelite_version_from_api() -> String {
    let resp = reqwest::get(
        "https://raw.githubusercontent.com/runelite/runelite/master/runelite-api/pom.xml",
    )
    .await
    .unwrap()
    .text()
    .await
    .unwrap();
    let version = resp.split("<version>").collect::<Vec<&str>>()[1]
        .split("</version>")
        .collect::<Vec<&str>>()[0];
    version.to_string()
}
