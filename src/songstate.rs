use std::{io::{BufRead, BufReader}, process::{Command, Stdio}, sync::mpsc::Sender, thread};

use std::sync::Arc;
use std::sync::Mutex;

use serde_json::Value;

use crate::{lyrics::{LyricLine, Lyrics}, utils::{get_song_blocking, log_to_file}};

pub enum LyricsState {
    Loaded,
    // Loading,
    // Missing,
    // Error,
    Invalidated
}

pub struct SongState {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub length: i64,
    pub len_secs: f64,
    pub pos_secs: f64,
    pub percentage: f64,
    pub lyrics_state: LyricsState,
    pub lyrics: Lyrics
}

impl SongState {
    pub fn new() -> Self {
        Self { 
            title: "".to_string(),
            artist: "".to_string(),
            album: "".to_string(),
            length: 0,
            len_secs: 0.0,
            pos_secs: 0.0,
            percentage: 0.0,
            lyrics_state: LyricsState::Invalidated,
            lyrics: Lyrics::new()
        }
    }

    pub fn update_metadata (&mut self, line: &String) -> bool {

        // let mut chars = line.chars();
        // chars.next();
        // chars.next_back();

        let values: Vec<String> = line// chars.as_str()
            .split('|')
            .map(|s| s.to_string())
            .collect();

        // println!("{:?}", &values);

        if values.len() == 4 {
            let [title, artist, album, length] =
                values.try_into().expect("exactly 4 fields expected");
            self.title = title;
            self.artist = artist;
            self.album = album;
            self.length = if let Ok(l) = length.parse::<i64>() { l } else { i64::MAX }; //length.parse().unwrap();
            self.len_secs = (self.length as f64) / 1000.0 / 1000.0;

            true

            // println!("t:{} a:{} a:{} l:{} p:{}", title, artist, album, length, position);
        } else {
            // println!("Wrong split result length");
            log_to_file("Wrong split result length".into());
            false
        }
    }

    pub fn update_position (&mut self, position_dirt: &String) -> bool {
        let mut chars = position_dirt.chars();
        chars.next();
        chars.next_back();
        chars.next_back();

        let position: String = chars.collect();
        let p: f64 = position.parse().unwrap();
        let new_pos_secs = p / 1000.0 / 1000.0;
        // pos_secs_incremented = pos_secs < new_pos_secs;
        let time_changed = self.pos_secs != new_pos_secs;
        self.pos_secs = new_pos_secs;
        self.percentage = self.pos_secs / self.len_secs;
        time_changed
    }

    pub fn check_and_update_position(&mut self) -> bool {
        let output = Command::new("playerctl")
            .arg("metadata")
            .arg("--format")
            .arg("'{{position}}'")
            .output();
            // .expect("failed to run playerctl for position");
        let position_dirt = String::from_utf8(output.unwrap().stdout).unwrap();
        if position_dirt != "" { self.update_position(&position_dirt) }
        else { false }
    }

    /* pub fn check_length(&mut self) -> Option<f64> {
        let output = Command::new("playerctl")
            .arg("metadata")
            .arg("--format")
            .arg("'{{mpris:length}}'")
            .output();
            // .expect("failed to run playerctl for position");
        let position_dirt = String::from_utf8(output.unwrap().stdout).unwrap();
        log_to_file(format!("position_dirt: |||{position_dirt}|||"));
        if position_dirt == "" {
            None
        } else {
            let mut chars = position_dirt.chars();
            chars.next();
            chars.next_back();
            chars.next_back();

            let position: String = chars.collect();
            let p: f64 = position.parse().unwrap();
            let new_pos_secs = p / 1000.0 / 1000.0;
            Some(new_pos_secs)
        }
    } */

    pub fn apply_song_text (&mut self, maybe_server_response: Result<String, reqwest::Error>) -> String {
        // TODO Manage the case syncedLyrics is null and plainLyrics is not null
        let mut status: String = String::new();

        match maybe_server_response {
            Ok(server_response) => {
                log_to_file(server_response.clone());
                let parsed: Value = serde_json::from_str(&server_response).unwrap();
                if let Some(status_code) = parsed.get("statusCode") { // API error
                    // Example: {"message":"Failed to find specified track","name":"TrackNotFound","statusCode":404}
                    status = parsed["message"].as_str().unwrap().to_string();
                    log_to_file(format!("status: {status_code} {status}"));
                    self.lyrics_state = LyricsState::Invalidated;
                } else if let Some(synced) = parsed.get("syncedLyrics") { // We have the lyrics!
                    if self.lyrics.convert_text(synced.as_str().unwrap()) {
                        // text_changed = true;
                        status = "Lyrics loaded and parsed successfully".into();
                        log_to_file(status.clone());
                        self.lyrics_state = LyricsState::Loaded;
                    } else {
                        status = "Something's wrong (1)".into();
                        log_to_file(status.clone());
                        self.lyrics_state = LyricsState::Invalidated;
                    }
                } else {
                    status = "Something's wrong (2)".into();
                    log_to_file(status.clone());
                    self.lyrics_state = LyricsState::Invalidated;
                }
            },
            Err(e) => {
                self.lyrics_state = LyricsState::Invalidated;
                status = "Error".into();
                log_to_file(format!("Error: {}", e));
            }
        }
        log_to_file(format!("NEW: {status}"));
        status
    }

    /* pub fn listen_to_playerctl (&mut self, tx: Sender<String>) {
        thread::spawn(move || {

            let child = Command::new("playerctl")
                .arg("metadata")
                .arg("--follow")
                .arg("--format")
                .arg("'{{title}}|{{artist}}|{{album}}|{{mpris:length}}'")
                .stdout(Stdio::piped())
                .spawn()
                .expect("failed to run playerctl");

            let stdout = child.stdout.expect("no stdout");
            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                if let Ok(l) = line {
                    log_to_file(format!("RECEIVED: {}", l.clone()));
                    if tx.send(l).is_err() {
                        break;
                    }
                }
            }
        });
    } */
}

pub fn listen_to_playerctl(
    state: Arc<Mutex<SongState>>,
    tx_notify: Sender<String>
) {
    thread::spawn(move || {
        let child = Command::new("playerctl")
            .arg("metadata")
            .arg("--follow")
            .arg("--format")
            .arg("{{title}}|{{artist}}|{{album}}|{{mpris:length}}")
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to run playerctl");

        let stdout = child.stdout.expect("no stdout");
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            if let Ok(l) = line {
                log_to_file(format!("RECEIVED: {}", l));

                let mut updated: bool = false;

                let mut title: String = "".into();
                let mut artist: String = "".into();
                let mut album: String = "".into();
                let mut duration: f64 = 0.0;
                // Lock dello stato
                if let Ok(mut s) = state.lock() {
                    updated = s.update_metadata(&l);

                    title = s.title.clone();
                    artist = s.artist.clone();
                    album = s.album.clone();
                    duration = s.len_secs;
                }
                if updated {
                    log_to_file("Metadata updated".to_string());
                    if let Ok(mut s) = state.lock() {
                        // let fake = LyricLine { seconds: 0, lyrics: "Fetching text".to_string() };
                        s.lyrics.reset();
                    }
                    if duration < 1.0 || duration > 3600.0 {
                        let _ = tx_notify.send(format!("Wrong length {duration}"));
                    } else if artist == "" {
                        let _ = tx_notify.send("No artist".into());
                    } else {
                        let _ = tx_notify.send("Fetching".to_string());

                        /* if let Ok(mut s) = state.lock() {
                            let fake = LyricLine { seconds: 0, lyrics: "Fetching text".to_string() };
                            s.lyrics.lines = vec![fake];
                        } */
                        let maybe_server_response = get_song_blocking(&title, &artist, &album, duration);
                        if updated {
                            if let Ok(mut s) = state.lock() {
                                let status = s.apply_song_text(maybe_server_response);
                                // Notifica che lo stato è cambiato
                                let _ = tx_notify.send(status);
                            }
                        }
                    }
                } else {
                    log_to_file("Metadata NOT updated".to_string());
                }
            }
        }
    });
}
