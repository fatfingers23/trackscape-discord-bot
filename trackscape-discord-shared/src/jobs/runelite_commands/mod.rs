use super::job_helpers::get_redis_client;
use redis::{Commands, RedisResult};

pub mod pb_command;

const RUNELITE_BASE_URL: &str = "https://api.runelite.net/runelite-";

pub async fn get_runelite_api_url() -> Result<String, anyhow::Error> {
    let version = get_runelite_version().await?;
    Ok(format!("{}{}", RUNELITE_BASE_URL, version))
}

async fn get_runelite_version() -> Result<String, anyhow::Error> {
    let mut redis_connection = get_redis_client().unwrap();
    let version_key = "runelite_version";
    let exists: RedisResult<bool> = redis_connection.exists(version_key);
    match exists {
        Ok(exist) => {
            if exist {
                Ok(redis_connection.get(version_key).unwrap())
            } else {
                let version = get_runelite_version_from_api().await?;
                let _: RedisResult<String> =
                    redis_connection.set_ex(version_key, version.clone(), 3600);
                Ok(version)
            }
        }
        Err(err) => Err(anyhow::Error::new(err)),
    }
}

async fn get_runelite_version_from_api() -> Result<String, anyhow::Error> {
    let resp = reqwest::get(
        "https://raw.githubusercontent.com/runelite/runelite/master/runelite-api/pom.xml",
    )
    .await?
    .text()
    .await?;

    let version = resp.split("<version>").collect::<Vec<&str>>()[1]
        .split("</version>")
        .collect::<Vec<&str>>()[0];
    Ok(version.to_string())
}
