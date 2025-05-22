# galdrar

Galdrar is the plural of galdr: in norse mythology, it is a song, a spell, an incantation.

Please note that this is a personal project, for personal use, developed in my (not so much) free time. You'll not find clean code or a flexible, modular system here. You'll find lots of experiments, abandoned ideas, dead code, temporary hacks and workarounds. Oh, and last but not least, I'm just learning both Rust and RataTUI. You've been warned.

## Notes

cargo run > log.txt 2>&1 is useful to execute and see logs tailing the file

## Usage

- Arrows up and down: scroll text up and down
- Arrows left and right: subtract/add 1s as time_offset for text highlightning
- q: exit

## TODO
- Remove loading try for "Voice message" and other titles that are not music
- Consider autoscroll implementing
- Consider a shift system on click (for position errors or wrong song version)
- Evaluate https://www.lyricsify.com as alternative lyrics source
