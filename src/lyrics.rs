use std::sync::Arc;

use ratatui::{style::{Color, Modifier, Style}, text::{Line, Span}};
use serde_derive::Deserialize;
use reqwest::Error;

use super::utils::*;

#[derive(Deserialize)]
pub struct LyricLine {
    pub seconds: i64,
    pub lyrics: String,
}

pub struct Lyrics {
    pub lines: Vec<LyricLine>,
    pub rendered_text: Vec<ratatui::text::Line<'static>>
}

impl Lyrics {
    pub fn new() -> Self {
        Self { 
            lines: vec![],
            rendered_text: vec![]
        }
    }

    fn current_lyric_index(&mut self, position_secs: f64) -> Option<usize> {
        self.lines.iter()
            .enumerate()
            .rev()
            .find(|(_, line)| (line.seconds as f64) <= position_secs)
            .map(|(i, _)| i)
    }

    fn style_text(&mut self, position_secs: f64) -> Vec<ratatui::text::Line<'static>> {
        let current_index = if let Some(i) = self.current_lyric_index(position_secs) { i } else { 0 };
        let lines: Vec<Line> = self.lines.iter().enumerate().map(|(i, line)| {
            let style = if i == current_index {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Line::from(vec![
                Span::raw(format!("{} ", to_human(line.seconds))),
                Span::styled(line.lyrics.clone(), style),
            ])
        }).collect();
        lines
    }

    pub fn update_style_text(&mut self, position_secs: f64) {
        self.rendered_text = self.style_text(position_secs);
    }

    pub fn set_text(&mut self, lines: Vec<LyricLine>) {
        self.lines = lines;
        self.update_style_text(0.0);
    }

    /* pub fn update_lyrics(&mut self, query: &String) {
        let url = format!("https://api.textyl.co/api/lyrics?q={}", query);
        // get_json_from_url(&url).await

        tokio::spawn(async move {
            //let rt = tokio::runtime::Runtime::new();
            //let text = rt.block_on(get_song_from_textyl(&query));
            let text = get_json_from_url(&url).await;
            
            match text {
                Ok(lyric) => {
                    if let Ok(rows) = serde_json::from_str::<Vec<LyricLine>>(&lyric) {
                        self.lines = rows;
                        // text_changed = true;
                        // log_text = "lyrics json ok".to_string();
                    } else {}
                    // println!("{} by {}'s lyric:\n{}", track, artists, lyric)
                },
                Err(e) => { /* log_text = format!("Error: {}", e); */ }
            }
        });
    } */

    /* async fn get_song_from_textyl(query: &str) -> Result<String, reqwest::Error> {
        let url = format!("https://api.textyl.co/api/lyrics?q={}", query);
        get_json_from_url(&url).await
    } */

    pub async fn fetch_lyrics(query: &String) -> Result<Vec<LyricLine>, Box<dyn std::error::Error>> {
        let url = format!("https://api.textyl.co/api/lyrics?q={}", query);
        let json = get_json_from_url(&url).await?;
        let lines = serde_json::from_str::<Vec<LyricLine>>(&json)?;
        Ok(lines)
    }
}