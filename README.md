# Galdrar - ᚷᚨᛚᛞᚱᚨᚱ

Galdrar is the plural of galdr: in norse mythology, it is a song, a spell, an incantation.

Please note that this is a personal project, for personal use, developed in my (not so much) free time. You'll not find clean code or a flexible, modular system here. You'll find lots of experiments, abandoned ideas, dead code, temporary hacks and workarounds. Oh, and last but not least, I'm just learning both Rust and RataTUI. You've been warned.

## Notes
This program logs to file, because the terminal output is used by TUI. The file path is /tmp/galdrar.log
Lyrics are downloaded from [lrclib](https://lrclib.net) APIs.

## Usage
- Arrows up and down: scroll text up and down
- Arrows left and right: subtract/add 500ms as time offset for text highlightning
- q or Esc: exit

## TODO
- ~~Consider autoscroll implementing~~ Done!
- Consider multi-player management

## Known bug (to be fixed soon)
- Manage the case syncedLyrics is null and plainLyrics is not null (now panics!)
- ~~The loaded lyrics is shown when time reaches the first line~~ Fixed!