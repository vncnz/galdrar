use ratatui::{style::{Color, Modifier, Style}, text::{Line, Span}};
use serde_derive::Deserialize;

pub fn to_human (secs: i64) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

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
}