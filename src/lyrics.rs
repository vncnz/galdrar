use ratatui::{style::{Color, Modifier, Style}, text::{Line, Span}};
use serde_derive::Deserialize;
use regex::Regex;

use super::utils::*;

#[derive(Deserialize)]
pub struct LyricLine {
    pub seconds: i64,
    pub lyrics: String,
}

pub struct Lyrics {
    pub lines: Vec<LyricLine>,
    pub rendered_text: Vec<ratatui::text::Line<'static>>,
    pub rendered_index: usize
}

impl Lyrics {
    pub fn new() -> Self {
        Self { 
            lines: vec![],
            rendered_text: vec![],
            rendered_index: 1000000
        }
    }

    fn current_lyric_index(&mut self, position_secs: f64) -> usize {
        self.lines.iter()
            .enumerate()
            .rev()
            .find(|(_, line)| (line.seconds as f64) <= position_secs)
            .map(|(i, _)| i)
            .unwrap_or(1000000)
    }

    fn style_text(&mut self, position_secs: f64) -> Option<(Vec<ratatui::text::Line<'static>>, usize)> {
        let current_index = self.current_lyric_index(position_secs);
        if current_index != self.rendered_index {
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
            Some((lines, current_index))
        } else {
            None
        }
    }

    pub fn update_style_text(&mut self, position_secs: f64) {
        if let Some((t, idx)) = self.style_text(position_secs) {
            self.rendered_text = t;
            self.rendered_index = idx;
        }
    }

    pub fn set_text(&mut self, lines: Vec<LyricLine>) {
        self.lines = lines;
        self.update_style_text(0.0);
    }

    pub fn convert_text (&mut self, synced: &str) -> bool {

        let re = Regex::new(r"^\[(\d+):(\d+\.\d+)\]\s*(.*)$").unwrap();
        let mut lines = Vec::new();
        // let v: Value = serde_json::from_str(&lyrics).unwrap();

        // Accedere direttamente alla chiave
        // if let Some(synced) = v.get("syncedLyrics") {
            log_to_file("syncedLyrics found".into());
            // println!("Synced lyrics:\n{}", synced);
            for line in synced.lines() {
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
        /* } else {
            log_to_file("syncedLyrics NOT found".into());
            log_to_file(format!("{v}"));
        } */

        if lines.len() == 0 {
            log_to_file("No lines produced".into());
            false
        } else {
            self.set_text(lines);
            true
        }
    }
}