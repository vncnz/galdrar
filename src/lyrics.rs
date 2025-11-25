use ratatui::{style::{Color, Modifier, Style}, text::{Line, Span}};
use serde_derive::Deserialize;

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
            rendered_index: 0
        }
    }

    fn current_lyric_index(&mut self, position_secs: f64) -> Option<usize> {
        self.lines.iter()
            .enumerate()
            .rev()
            .find(|(_, line)| (line.seconds as f64) <= position_secs)
            .map(|(i, _)| i)
    }

    // TODO: cache current_index and don't restyle all if not changed
    fn style_text(&mut self, position_secs: f64) -> Option<(Vec<ratatui::text::Line<'static>>, usize)> {
        let current_index = if let Some(i) = self.current_lyric_index(position_secs) { i } else { 0 };
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
}