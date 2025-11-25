use std::{io::{BufRead, BufReader}, process::{Command, Stdio}, sync::mpsc::Sender, thread};



pub struct SongState {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub length: i64,
    pub len_secs: f64,
    pub pos_secs: f64,
    pub percentage: f64
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
            percentage: 0.0
        }
    }

    pub fn update_metadata (&mut self, line: &String) {

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
            let [title, artist, album, length] =
                values.try_into().expect("exactly 4 fields expected");
            self.title = title;
            self.artist = artist;
            self.album = album;
            self.length = if let Ok(l) = length.parse::<i64>() { l } else { i64::MAX }; //length.parse().unwrap();
            self.len_secs = (self.length as f64) / 1000.0 / 1000.0;

            // println!("t:{} a:{} a:{} l:{} p:{}", title, artist, album, length, position);
        } else {
            // println!("Wrong split result length");
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

    pub fn listen_to_playerctl (&mut self, tx: Sender<String>) {
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
    }
}