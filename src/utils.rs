use reqwest::{Client, ClientBuilder};
use std::time::Duration;

pub fn to_human (secs: i64) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

fn create_insecure_client() -> Client {
    ClientBuilder::new()
        .danger_accept_invalid_certs(true)  // <-- questa è la riga chiave
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build insecure client")
}

pub async fn get_json_from_url(url: &str) -> Result<String, reqwest::Error> {
    let response = create_insecure_client().get(url).send().await?;
    let body = response.text().await?;
    Ok(body)
}

pub async fn get_song_from_textyl(query: &str) -> Result<String, reqwest::Error> {
    let url = format!("https://api.textyl.co/api/lyrics?q={}", query);
    get_json_from_url(&url).await
}