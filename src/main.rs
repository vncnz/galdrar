// use crossterm::cursor::position;
// use lyric_finder::LyricResult;
use ratatui::{prelude::*, widgets::*};
use ratatui::text::{Line, Span};
use reqwest::{Client, ClientBuilder};
use core::panic;
use std::{fmt::Error, io::{self, BufRead, BufReader}, process::{Command, Stdio}, sync::mpsc, thread, time::Duration};
use crossterm::{event::{self, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use std::io::stdout;

use serde_json;

mod songstate;
use songstate::*;

mod lyrics;
use lyrics::*;

mod utils;
use utils::*;

/* let items: Vec<Spans> = lyrics.iter().enumerate().map(|(i, line)| {
    let prefix = if i == current_index { "➤ " } else { "  " };
    Spans::from(vec![
        Span::styled(prefix, Style::default().fg(Color::Gray)),
        Span::raw(&line.frase),
    ])
}).collect(); */

fn main1() -> Result<(), Box<dyn std::error::Error>> {
    // Channel for communication between reader thread and UI
    let (tx, rx) = mpsc::channel();
    let (tx_lyrics, rx_lyrics) = mpsc::channel();
    let mut songinfo = SongState::new();
    let mut lyrics = Lyrics::new();

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
                if tx.send(l).is_err() {
                    break;
                }
            }
        }
    });

    // Terminal setup
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let mut vertical_scroll: usize = 0;
    let mut vertical_scroll_state = ScrollbarState::new(10);

    let mut running = String::new();
    let mut log_text = "Starting...".to_string();
    // let mut rendered_text: Vec<ratatui::text::Line> = vec![];

    let mut time_offset = 0.0;

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        terminal.clear()?;
                        let mut stdout = io::stdout();
                        execute!(stdout, LeaveAlternateScreen)?;
                        disable_raw_mode()?;
                        terminal.show_cursor()?;

                        return Ok(());
                    },
                    KeyCode::Down => {
                        vertical_scroll = vertical_scroll.saturating_add(1);
                        vertical_scroll_state = vertical_scroll_state.position(vertical_scroll);
                    },
                    KeyCode::Up => {
                        vertical_scroll = vertical_scroll.saturating_sub(1);
                        vertical_scroll_state = vertical_scroll_state.position(vertical_scroll);
                    },
                    KeyCode::Left => {
                        time_offset -= 1000.0;
                    },
                    KeyCode::Right => {
                        time_offset += 1000.0;
                    },
                    _ => {}
                }
            }
        }

        let mut text_changed = false;

        // Non-blocking receive
        while let Ok(line) = rx.try_recv() {
            log_text = "Playerctl update received".to_string();

            songinfo.update_metadata(&line);

            let new_running = format!("{} {}", songinfo.artist, songinfo.title);
            if running != new_running {

                log_text = "New song".to_string();
                lyrics.lines = vec![];
                text_changed = true;
                time_offset = 0.0;

                running = new_running.clone();

                let mut stop = "";
                if running.contains("Advertisment") { stop = "Advertisement"; }
                else if running.contains("Voice message") { stop = "Voice message"; }
                else if songinfo.artist == "" { stop = "No artist"; }
                if stop == "" {
                    let rt = tokio::runtime::Runtime::new()?;
                    let text = rt.block_on(get_song_from_textyl(&new_running));

                    /* let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.spawn({
                        let tx_lyrics_cloned = tx_lyrics.clone();
                        async move {
                            if let Ok(lines) = Lyrics::fetch_lyrics(&new_running).await {
                                tx_lyrics_cloned.send(lines).expect("Error sending lyrics");
                            }
                        }
                    }); */

                    match text {
                        Ok(lyric) => {
                            if lyric == "No lyrics available" {
                                lyrics.lines = vec![];
                                log_text = "No lyrics available".to_string();
                            } else if let Ok(rows) = serde_json::from_str::<Vec<LyricLine>>(&lyric) {
                                lyrics.lines = rows;
                                text_changed = true;
                                log_text = "lyrics json ok".to_string();
                            } else {
                                log_text = format!("Json conversion NOT OK: {}", lyric);
                            }
                            // println!("{} by {}'s lyric:\n{}", track, artists, lyric)
                        },
                        Err(e) => { log_text = format!("Error: {}", e); }
                    }
                } else {
                    lyrics.lines = vec![];
                    log_text = stop.to_string();
                }
            } else {
                log_text = "No changes".to_string();
            }
        }

        let mut lyrics_updated = false;
        while let Ok(lines) = rx_lyrics.try_recv() {
            lyrics.set_text(lines);
            lyrics_updated = true;
        }

        let output = Command::new("playerctl")
            .arg("metadata")
            .arg("--format")
            .arg("'{{position}}'")
            .output();
            // .expect("failed to run playerctl for position");
        let position_dirt = String::from_utf8(output.unwrap().stdout).unwrap();

        let time_changed = songinfo.update_position(&position_dirt);

        if lyrics.lines.len() > 0 && (time_changed || lyrics_updated) {
            // rendered_text = lyrics.style_text(songinfo.pos_secs + (time_offset as f64 / 1000.0));
            lyrics.update_style_text(songinfo.pos_secs + (time_offset as f64 / 1000.0));
        } else {
            // last_text = "No need to refresh".to_string();
        }

        terminal.draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(8),
                    Constraint::Min(5),
                    Constraint::Length(3)
                ])
                .split(f.area());

            let block_info = Block::default().title("Playerctl Output").borders(Borders::ALL);
            let block = Block::default().title("Lyrics").borders(Borders::ALL);
            let block_log = Block::default().title("Log").borders(Borders::ALL);
            // let paragraph = Paragraph::new(lines.clone().join("\n")).block(block);
            // '{{title}}|{{artist}}|{{album}}|{{mpris:length}}|{{position}}'
            if songinfo.title != "" {

                let perc_100 = songinfo.percentage * 100.0;
                let offset_secs = time_offset / 1000.0;
                let simulated_pos = songinfo.pos_secs + offset_secs;
                let h_simulated_pos = to_human(simulated_pos as i64);
                let to_print = format!("title: {}\nartist {}\nalbum {}\nlength {} ({:.1} secs)\nposition {:.1} secs + offset {:.1} secs = {}\npercentage {:.0}%", songinfo.title, songinfo.artist, songinfo.album, songinfo.length, songinfo.len_secs, songinfo.pos_secs, offset_secs, h_simulated_pos, perc_100);
                let paragraph_info = Paragraph::new(to_print).block(block_info);
                let paragraph = Paragraph::new(lyrics.rendered_text.clone())
                    .block(block).scroll((vertical_scroll as u16, 0));
                let paragraph_log = Paragraph::new(log_text.clone()).block(block_log);
                f.render_widget(paragraph_info, layout[0]);
                f.render_widget(paragraph, layout[1]);
                f.render_widget(paragraph_log, layout[2]);
                f.render_stateful_widget(
                    Scrollbar::new(ScrollbarOrientation::VerticalRight)
                        .begin_symbol(Some("↑"))
                        .end_symbol(Some("↓")),
                    layout[1],
                    &mut vertical_scroll_state
                )
            }
        })?;

        thread::sleep(Duration::from_millis(16));
    }
}

async fn download_lyrics (searc_query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client =  lyric_finder::Client::new();
    let result = client.get_lyric(searc_query).await?;
    match result {
        lyric_finder::LyricResult::Some {
            track,
            artists,
            lyric,
        } => {
            // println!("{} by {}'s lyric:\n{}", track, artists, lyric);
            Ok(lyric)
        }
        lyric_finder::LyricResult::None => {
            // println!("lyric not found!");
            panic!("lyric not found!")
        }
    }
}


fn main() -> io::Result<()> {
    main1();
    Ok(())
}
