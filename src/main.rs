use ratatui::{prelude::*, widgets::*};
// use ratatui::text::{Line, Span};
// use reqwest::{Client, ClientBuilder};
use std::{io, sync::mpsc, thread, time::Duration};
use crossterm::{event::{self, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use std::io::stdout;

mod songstate;
use songstate::*;

mod lyrics;
use lyrics::*;

mod utils;
use utils::*;

use std::sync::Arc;
use std::sync::Mutex;

fn main1() -> Result<(), Box<dyn std::error::Error>> {
    // Channel for communication between reader thread and UI
    let (tx, rx) = mpsc::channel();
    // let (tx_lyrics, rx_lyrics) = mpsc::channel();
    // let mut songinfo = SongState::new();
    // let mut lyrics = Lyrics::new();

    let songinfo_mux = Arc::new(Mutex::new(SongState::new()));
    // let (tx_notify, rx_notify) = std::sync::mpsc::channel();
    listen_to_playerctl(songinfo_mux.clone(), tx);

    // Terminal setup
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let mut vertical_scroll: usize = 0;
    let mut vertical_scroll_state = ScrollbarState::new(10);

    let mut running = String::new();
    // let mut log_text = "Starting...".to_string();
    
    // let mut rendered_text: Vec<ratatui::text::Line> = vec![];

    let mut time_offset = 0.0;

    let songinfo_mux_clone = songinfo_mux.clone();
    loop {
        let mut status: String = "".to_string();
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
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
                        time_offset -= 500.0;
                    },
                    KeyCode::Right => {
                        time_offset += 500.0;
                    },
                    _ => {}
                }
            }
        }

        let mut text_changed = false;
        let mut lyrics_updated = false;

        let mut songinfo = songinfo_mux_clone.lock().unwrap();

        // Non-blocking receive
        while let Ok(_line) = rx.try_recv() { // TODO: line is intended to be used in near future
            text_changed = true;
            lyrics_updated = true;
            // log_text = "Playerctl update received".to_string();

            let new_running = format!("{} {}", songinfo.artist, songinfo.title);
            if running != new_running {
                log_to_file("New song".into());

                vertical_scroll_state = vertical_scroll_state.content_length(songinfo.lyrics.lines.len());
                // text_changed = true;
                time_offset = 0.0;

                running = new_running.clone();

                
                if running.contains("Advertisment") {
                    let fake = LyricLine { seconds: 0, lyrics: "Oh, another advertisment!".to_string() };
                    songinfo.lyrics.lines = vec![fake];
                    status = "Advertisement".into();
                    log_to_file("Advertisement recognized".into());
                }
                else if running.contains("Voice message") {
                    let fake = LyricLine { seconds: 0, lyrics: "Not a song!".to_string() };
                    songinfo.lyrics.lines = vec![fake];
                    status = "Voice message".into();
                    log_to_file("Voice message recognized".into());
                }
                else if songinfo.artist == "" {
                    let fake = LyricLine { seconds: 0, lyrics: "No artist: not a song, maybe?".to_string() };
                    songinfo.lyrics.lines = vec![fake];
                    status = "No artist".into();
                    log_to_file("No artist".into());
                }
                else {
                    // This is a song
                    /* let rt = tokio::runtime::Runtime::new()?;
                    log_to_file(format!("title: {} artist: {} album: {} length: {}", &songinfo.title, &songinfo.artist, &songinfo.album, songinfo.len_secs));
                    songinfo.lyrics_state = LyricsState::Loading;
                    let maybe_server_response = rt.block_on(get_song_from_rlclib(&songinfo.title, &songinfo.artist, &songinfo.album, songinfo.len_secs));

                    match maybe_server_response {
                        Ok(server_response) => {
                            log_to_file(server_response.clone());
                            let parsed: Value = serde_json::from_str(&server_response).unwrap();
                            if let Some(status_code) = parsed.get("statusCode") { // API error
                                // Example: {"message":"Failed to find specified track","name":"TrackNotFound","statusCode":404}
                                status = parsed["message"].as_str().unwrap().to_string();
                                log_to_file(format!("status: {status_code} {status}"));
                                songinfo.lyrics_state = LyricsState::Invalidated;
                            } else if let Some(synced) = parsed.get("syncedLyrics") { // We have the lyrics!
                                if songinfo.lyrics.convert_text(synced.as_str().unwrap()) {
                                    text_changed = true;
                                    status = "Lyrics loaded and parsed successfully".into();
                                    log_to_file(status.clone());
                                    songinfo.lyrics_state = LyricsState::Loaded;
                                } else {
                                    status = "Something's wrong (1)".into();
                                    log_to_file(status.clone());
                                    songinfo.lyrics_state = LyricsState::Invalidated;
                                }
                            } else {
                                status = "Something's wrong (2)".into();
                                log_to_file(status.clone());
                                songinfo.lyrics_state = LyricsState::Invalidated;
                            }
                        },
                        Err(e) => {
                            songinfo.lyrics_state = LyricsState::Invalidated;
                            status = "Error".into();
                            log_to_file(format!("Error: {}", e));
                        }
                    } */
                }
            } else {
                status = "No changes".into();
            }

            vertical_scroll_state = vertical_scroll_state.content_length(songinfo.lyrics.lines.len());
        }

        
        /* while let Ok(lines) = rx_lyrics.try_recv() {
            songinfo.lyrics.set_text(lines);
            lyrics_updated = true;
            // status = format!("Lyrics update received ({} lines)", lyrics.lines.len());
        } */
        let time_changed = songinfo.check_and_update_position();
        // let len = songinfo.check_length();
        // log_to_file(format!("len {len:?}"));

        if songinfo.lyrics.lines.len() > 0 && (time_changed || text_changed || lyrics_updated) {
            // rendered_text = lyrics.style_text(songinfo.pos_secs + (time_offset as f64 / 1000.0));
            let new_pos = songinfo.pos_secs + (time_offset as f64 / 1000.0);
            songinfo.lyrics.update_style_text(new_pos);
        } else {
            // last_text = "No need to refresh".to_string();
        }

        let status_clone = status.clone();
        terminal.draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(6),
                    Constraint::Min(5),
                    Constraint::Length(3)
                ])
                .split(f.area());

            let block_info = Block::default().title("Playerctl Output").borders(Borders::ALL);
            let block = Block::default().title("Lyrics").borders(Borders::ALL);
            let block_log = Block::default().title("Status").borders(Borders::ALL);
            // let paragraph = Paragraph::new(lines.clone().join("\n")).block(block);
            // '{{title}}|{{artist}}|{{album}}|{{mpris:length}}|{{position}}'
            if songinfo.title != "" {

                let perc_100 = songinfo.percentage * 100.0;
                let offset_secs = time_offset / 1000.0;
                let simulated_pos = songinfo.pos_secs + offset_secs;
                let h_simulated_pos = to_human(simulated_pos as i64);
                let h_length = to_human(songinfo.len_secs as i64);
                // let to_print = format!("title: {}\nartist {}\nalbum {}\nlength {:.1} secs = {}\nposition {:.1} secs + offset {:.1} secs = {}\npercentage {:.0}%", songinfo.title, songinfo.artist, songinfo.album, songinfo.len_secs, h_length, songinfo.pos_secs, offset_secs, h_simulated_pos, perc_100);
                let to_print = format!("title: {}\nartist {}\nalbum {}\n{} / {} ({:.0}%, offset {}s)", songinfo.title, songinfo.artist, songinfo.album, h_simulated_pos, h_length, perc_100, offset_secs);
                let paragraph_info = Paragraph::new(to_print).block(block_info);
                let paragraph = Paragraph::new(songinfo.lyrics.rendered_text.clone())
                    .block(block).scroll((vertical_scroll as u16, 0));
                let paragraph_log = Paragraph::new(status_clone).block(block_log);
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


fn main() -> io::Result<()> {
    let _ = main1();
    Ok(())
}
