use super::get_runelite_api_url;

pub async fn get_pb(message: String, player: String, guild_id: u64) {
    let runelite_version = get_runelite_api_url().await;
    println!("Runelite version: {}", runelite_version);
}
