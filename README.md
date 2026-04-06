# absorb

A terminal speed reader using [RSVP](https://en.wikipedia.org/wiki/Rapid_serial_visual_presentation) (Rapid Serial Visual Presentation). Feed it any text file and absorb the content word-by-word at your chosen pace.

![Rust](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## How it works

RSVP displays one word at a time at a fixed point on screen. Each word is aligned on its **Optimal Recognition Point** (ORP) — the character your eye naturally fixates on — highlighted in red. This eliminates saccadic eye movements and lets you read significantly faster than traditional left-to-right scanning.

The reading speed eases in gradually over the first 10 words, starting at one third of your target WPM and ramping up smoothly. This gives your brain time to settle into the flow.

## Install

```bash
cargo install absorb
```

Or build from source:

```bash
git clone https://github.com/kloki/absorb.git
cd absorb
cargo build --release
```

## Usage

```bash
# Read a file
absorb document.txt

# Pipe text from stdin
cat article.md | absorb

# Set speed to 400 words per minute
absorb -w 400 document.txt
```

## Controls

| Key          | Action                   |
| ------------ | ------------------------ |
| `Space`      | Play / Pause             |
| `←` or `h`   | Step back one word       |
| `→` or `l`   | Step forward one word    |
| `↑` or `k`   | Increase speed (+25 WPM) |
| `↓` or `j`   | Decrease speed (-25 WPM) |
| `v`          | Toggle split view        |
| `r`          | Restart                  |
| `q` or `Esc` | Quit                     |

## Split view

Press `v` to toggle split view. The top half shows the RSVP word display, while the bottom half shows the full text with the current word highlighted in red. This is useful for maintaining context while speed reading.

## Configuration

| Flag          | Default | Description             |
| ------------- | ------- | ----------------------- |
| `-w`, `--wpm` | 600     | Target words per minute |

Speed can also be adjusted on the fly with `↑`/`↓` in increments of 25 WPM (range: 50–1000).

## License

MIT
