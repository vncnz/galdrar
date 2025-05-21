use ratatui::{prelude::*, widgets::*};
use std::{io::{self, BufRead, BufReader}, process::{Command, Stdio}, sync::mpsc, thread, time::Duration};
use crossterm::{event::{self, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use std::io::stdout;

fn main() -> io::Result<()> {
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

    let mut lines: Vec<String> = Vec::new();

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
            }
        }

        // Non-blocking receive
        while let Ok(line) = rx.try_recv() {
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
                let to_print = format!("title: {title}\nartist {artist}\nalbum {album}\nlength {length}\nposition {position}\npercentage {perc}");
                let paragraph = Paragraph::new(to_print).block(block);
                f.render_widget(paragraph, size);
            }
        })?;

        thread::sleep(Duration::from_millis(100));
    }
}
