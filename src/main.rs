use ratatui::{prelude::*, widgets::*};
use core::panic;
use std::{fmt::Error, io::{self, BufRead, BufReader, ErrorKind}, process::{Command, Stdio}, sync::mpsc, thread, time::Duration};
use crossterm::{event::{self, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use std::io::stdout;





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
            .arg("'{{title}}|{{artist}}|{{album}}|{{mpris:length}}|{{position}}'")
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
    let mut scroll: u16 = 0;

    let mut lines: Vec<String> = Vec::new();
    let mut running = String::new();
    let mut last_text = String::new();

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    terminal.clear()?;
                    let mut stdout = io::stdout();
                    execute!(stdout, LeaveAlternateScreen)?;
                    disable_raw_mode()?;
                    terminal.show_cursor()?;

                    return Ok(());
                }
                if key.code == KeyCode::Down {
                    scroll = scroll.saturating_add(1);
                }
                if key.code == KeyCode::Up {
                    scroll = scroll.saturating_sub(1);
                }
            }
        }

        // Non-blocking receive
        while let Ok(line) = rx.try_recv() {

            let mut chars = line.chars();
            chars.next();
            chars.next_back();
            let [title, artist, album, length, position]: [&str; 5] = chars.as_str()
                .split('|').collect::<Vec<&str>>().try_into().unwrap();
            
            let mut perc:f64 = 0.0;
            if let (Ok(ipos), Ok(ilen)) = (position.parse::<f64>(), length.parse::<f64>()) {
                perc = ipos / ilen;
            }

            let new_running = format!("{title} {artist}");
            if running != new_running {
                running = new_running;
                // f.set_title(&running);
                let rt = tokio::runtime::Runtime::new()?;
                let text = rt.block_on(downloadLyrics());

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
                }
            }

            lines.push(line);
            if lines.len() > 10 {
                lines.remove(0);
            }
        }

        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().title("Playerctl Output").borders(Borders::ALL);
            // let paragraph = Paragraph::new(lines.clone().join("\n")).block(block);
            // '{{title}}|{{artist}}|{{album}}|{{mpris:length}}|{{position}}'
            if let Some(last) = lines.last() {
                let mut chars = last.chars();
                chars.next();
                chars.next_back();
                let [title, artist, album, length, position]: [&str; 5] = chars.as_str()
                    .split('|').collect::<Vec<&str>>().try_into().unwrap();
                
                let mut perc:f64 = 0.0;
                if let (Ok(ipos), Ok(ilen)) = (position.parse::<f64>(), length.parse::<f64>()) {
                    perc = ipos / ilen;
                }
                let to_print = format!("title: {title}\nartist {artist}\nalbum {album}\nlength {length}\nposition {position}\npercentage {perc:.2}%\nlyric {last_text}");
                let paragraph = Paragraph::new(to_print).block(block).scroll((scroll, 0));
                f.render_widget(paragraph, size);
            }
        })?;

        thread::sleep(Duration::from_millis(100));
    }
}

async fn downloadLyrics () -> Result<lyric_finder::LyricResult, Box<dyn std::error::Error>> {
    let client =  lyric_finder::Client::new();
    let result = client.get_lyric("Epica - Storm the sorrow").await?;
    match result {
        lyric_finder::LyricResult::Some {
            track,
            artists,
            lyric,
        } => {
            // println!("{} by {}'s lyric:\n{}", track, artists, lyric);
            Ok(lyric_finder::LyricResult::Some {
                track,
                artists,
                lyric,
            })
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
