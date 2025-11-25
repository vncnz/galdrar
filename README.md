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
- Consider autoscroll implementing

## Known bug (to be fixed soon)
First song loading is tried when there is a wrong length as metadata. Length is fixed within seconds, when play starts, but the network request is already running with the wrong query data.
For example, this is a log trace:

[22:55:13.023] title: Boulevard of Broken Dreams artist: Green Day album: American Idiot (20th Anniversary Deluxe Edition) length: 9223372036854.775
[22:55:17.967] syncedLyrics NOT found
[22:55:17.967] {"message":"duration: must be between 1 and 3600","name":"ValidationError","statusCode":400}
[22:55:17.967] No lines produced
[22:55:17.967] Something's wrong