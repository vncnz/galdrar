use regex::Regex;
use reqwest::{Client, ClientBuilder};
use serde_json::Value;
use std::{thread, time::Duration};

use crate::lyrics::{LyricLine, Lyrics};

use std::fs::OpenOptions;
use std::io::Write;

pub fn log_to_file(msg: String) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/galdrar.log")
        .expect("impossibile aprire log file");
    writeln!(file, "[{}] {}", chrono::Local::now().format("%H:%M:%S%.3f"), msg).unwrap();
}

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

pub async fn get_text_from_url(url: &str) -> Result<String, reqwest::Error> {
    let response = create_insecure_client().get(url).send().await?;
    let body = response.text().await?;
    Ok(body)
}

pub async fn get_song_from_textyl(query: &str) -> Result<String, reqwest::Error> {
    let url = format!("https://api.textyl.co/api/lyrics?q={}", query);
    get_text_from_url(&url).await
}

pub async fn get_song_from_rlclib(title: &str, artist: &str, album: &str, duration: f64) -> Result<String, reqwest::Error> {
    let url = format!("https://lrclib.net/api/get?artist_name={artist}&track_name={title}&album_name={album}&duration={duration}");
    get_text_from_url(&url).await
}

pub fn convert_text (lyrics: &str) -> Option<Vec<LyricLine>> {

    let re = Regex::new(r"^\[(\d+):(\d+\.\d+)\]\s*(.*)$").unwrap();
    let mut lines = Vec::new();
    let v: Value = serde_json::from_str(&lyrics).unwrap();

    // Accedere direttamente alla chiave
    if let Some(synced) = v.get("syncedLyrics") {
        log_to_file("syncedLyrics found".into());
        // println!("Synced lyrics:\n{}", synced);
        for line in synced.as_str()?.lines() {
            if let Some(caps) = re.captures(line) {
                let minutes: i64 = caps[1].parse().unwrap_or(0);
                let seconds: f64 = caps[2].parse().unwrap_or(0.0);
                let total_seconds = (minutes * 60) as f64 + seconds;
                lines.push(LyricLine {
                    seconds: total_seconds.round() as i64,
                    lyrics: caps[3].to_string(),
                });
            }
        }
    } else {
        log_to_file("syncedLyrics NOT found".into());
        log_to_file(format!("{v}"));
    }

    if lines.len() == 0 {
        log_to_file("No lines produced".into());
        None
    } else {
        Some(lines)
    }
}
