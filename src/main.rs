use crossterm::cursor::position;
use lyric_finder::LyricResult;
use ratatui::{prelude::*, widgets::*};
use ratatui::text::{Line, Span};
use reqwest::{Client, ClientBuilder};
use core::panic;
use std::{fmt::Error, io::{self, BufRead, BufReader}, process::{Command, Stdio}, sync::mpsc, thread, time::Duration};
use crossterm::{event::{self, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use std::io::stdout;
use serde_derive::Deserialize;
use serde_json;

#[derive(Deserialize)]
struct LyricLine {
    seconds: i64,
    lyrics: String,
}

fn current_lyric_index(position_secs: f64, lyrics: &[LyricLine]) -> Option<usize> {
    lyrics.iter()
        .enumerate()
        .rev()
        .find(|(_, line)| (line.seconds as f64) <= position_secs)
        .map(|(i, _)| i)
}

fn to_human (secs: i64) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

fn style_text(position_secs: f64, rows: &Vec<LyricLine>) -> Vec<ratatui::text::Line<'static>> {
    let current_index = if let Some(i) = current_lyric_index(position_secs, &rows) { i } else { 0 };
    let lines: Vec<Line> = rows.iter().enumerate().map(|(i, line)| {
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

/* let items: Vec<Spans> = lyrics.iter().enumerate().map(|(i, line)| {
    let prefix = if i == current_index { "➤ " } else { "  " };
    Spans::from(vec![
        Span::styled(prefix, Style::default().fg(Color::Gray)),
        Span::raw(&line.frase),
    ])
}).collect(); */



fn create_insecure_client() -> Client {
    ClientBuilder::new()
        .danger_accept_invalid_certs(true)  // <-- questa è la riga chiave
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build insecure client")
}

async fn get_song_from_textyl(query: &str) -> Result<String, reqwest::Error> {
    let url = format!("https://api.textyl.co/api/lyrics?q={}", query);
    get_json_from_url(&url).await
}

async fn get_json_from_url(url: &str) -> Result<String, reqwest::Error> {
    let response = create_insecure_client().get(url).send().await?;

    // let status = response.status();
    let body = response.text().await?;

    // println!("Status: {}", status);
    // println!("Body:\n{}", body);

    Ok(body)
}

fn main1() -> Result<(), Box<dyn std::error::Error>> {
    // Channel for communication between reader thread and UI
    let (tx, rx) = mpsc::channel();

    // Spawn a thread to read playerctl output
    thread::spawn(move || {
        /* let child = Command::new("playerctl")
            .arg("metadata")
            .arg("--follow")
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to run playerctl"); */

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

    let mut lines: Vec<String> = Vec::new();
    let mut running = String::new();
    let mut last_text = String::new();
    let mut last_text_with_times: Vec<LyricLine> = vec![];
    let mut rendered_text: Vec<ratatui::text::Line> = vec![];

    let mut title = String::new();
    let mut artist = String::new();
    let mut album = String::new();
    let mut length = String::new();
    let mut position = String::new();
    let mut len_secs: f64 = 0.0;
    let mut pos_secs: f64 = 0.0;
    let mut perc: f64 = 0.0;

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

        // Non-blocking receive
        while let Ok(line) = rx.try_recv() {
            let mut chars = line.chars();
            chars.next();
            chars.next_back();

            
            let values: Vec<String> = chars
                .as_str()
                .split('|')
                .map(|s| s.to_string())
                .collect();

            // println!("{:?}", &values);

            if values.len() == 4 {
                [title, artist, album, length] =
                    values.try_into().expect("exactly 4 fields expected");

                // println!("t:{} a:{} a:{} l:{} p:{}", title, artist, album, length, position);
            } else {
                // println!("Wrong split result length");
            }

            if let Ok(ilen) = length.parse::<f64>() {
                len_secs = ilen / 1000.0 / 1000.0;
            }

            let new_running = format!("{artist} {title}");
            if running != new_running {

                last_text = String::new();
                last_text_with_times = vec![];
                time_offset = 0.0; // if !pos_secs_incremented { -pos_secs * 1000.0 } else { 0.0 }; // time_offset = 0.0;

                running = new_running.clone();

                // f.set_title(&running);
                let rt = tokio::runtime::Runtime::new()?;

                /* let text = rt.block_on(download_lyrics(&new_running));
                match text {
                    Ok(lyric_finder::LyricResult::Some {
                        track,
                        artists,
                        lyric,
                    }) => {
                        last_text = lyric;
                        // println!("{} by {}'s lyric:\n{}", track, artists, lyric)
                    },
                    Ok(lyric_finder::LyricResult::None) => { last_text = "".to_string(); println!("lyric not found!") },
                    Err(e) => { last_text = "".to_string(); println!("Error: {}", e) }
                } */

                let text = rt.block_on(get_song_from_textyl(&new_running));
                match text {
                    Ok(lyric) => {
                        if let Ok(rows) = serde_json::from_str::<Vec<LyricLine>>(&lyric) {
                            last_text_with_times = rows;
                            last_text = "json ok".to_string();
                        } else {}
                        // println!("{} by {}'s lyric:\n{}", track, artists, lyric)
                    },
                    Err(e) => { last_text = format!("Error: {}", e); }
                }
            }

            /* if last_text_with_times.len() > 0 {
                rendered_text = style_text(pos_secs + (time_offset as f64 / 1000.0), &last_text_with_times);
            } */

            /* lines.push(line);
            if lines.len() > 10 {
                lines.remove(0);
            } */
        }

        let output = Command::new("playerctl")
            .arg("metadata")
            .arg("--format")
            .arg("'{{position}}'")
            .output();
            // .expect("failed to run playerctl for position");
        let position_dirt = String::from_utf8(output.unwrap().stdout).unwrap();

        let mut chars = position_dirt.chars();
        chars.next();
        chars.next_back();
        chars.next_back();

        position = chars.collect();

        // let mut pos_secs_incremented = false;
        let mut time_changed: bool = false;
        if let Ok(ipos) = position.parse::<f64>() {
            let new_pos_secs = ipos / 1000.0 / 1000.0;
            // pos_secs_incremented = pos_secs < new_pos_secs;
            time_changed = pos_secs != new_pos_secs;
            pos_secs = new_pos_secs;
            perc = pos_secs / len_secs;
        }

        if last_text_with_times.len() > 0 && time_changed {
            rendered_text = style_text(pos_secs + (time_offset as f64 / 1000.0), &last_text_with_times);
        }

        terminal.draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(8),
                    Constraint::Min(5)
                ])
                .split(f.area());

            let block_info = Block::default().title("Playerctl Output").borders(Borders::ALL);
            let block = Block::default().title("Lyrics").borders(Borders::ALL);
            // let paragraph = Paragraph::new(lines.clone().join("\n")).block(block);
            // '{{title}}|{{artist}}|{{album}}|{{mpris:length}}|{{position}}'
            if title != "" {
                // let mut chars = last.chars();
                // chars.next();
                // chars.next_back();
                // let [title, artist, album, length, position]: [&str; 5] = chars.as_str().split('|').collect::<Vec<&str>>().try_into().unwrap();

                let perc_100 = perc * 100.0;
                let offset_secs = time_offset / 1000.0;
                let simulated_pos = pos_secs + offset_secs;
                let h_simulated_pos = to_human(simulated_pos as i64);
                let to_print = format!("title: {title}\nartist {artist}\nalbum {album}\nlength {length} ({len_secs:.1} secs)\nposition {position} ({pos_secs:.1} secs) + offset {offset_secs:.1} secs = {h_simulated_pos}\npercentage {perc_100:.0}%");
                let paragraph_info = Paragraph::new(to_print).block(block_info);
                let paragraph = Paragraph::new(rendered_text.clone())
                    .block(block).scroll((vertical_scroll as u16, 0));
                f.render_widget(paragraph_info, layout[0]);
                f.render_widget(paragraph, layout[1]);
                f.render_stateful_widget(
                    Scrollbar::new(ScrollbarOrientation::VerticalRight)
                        .begin_symbol(Some("↑"))
                        .end_symbol(Some("↓")),
                    layout[1],
                    &mut vertical_scroll_state
                )
            }
        })?;

        thread::sleep(Duration::from_millis(100));
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
