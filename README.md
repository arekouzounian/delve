# Delve
---
An ASCII rendering engine & video game for the terminal. Hand-coded, for enjoyment purposes only.

The goal is to layer a simple game atop this engine, so it may later involve physics and all that stuff. This is a simple project that's just for fun. No AI allowed to write code, although Claude was pretty helpful in explaining a lot of the vector math, and for establishing a preliminary roadmap in [[roadmap.md]].

The goal is to adopt few dependencies; `crossterm` is the major way the engine performs actual terminal rendering, and some number crates (`num-trait`, `rand`) will also likely enter the picture. But other than that I want to try to make it as light on deps as possible.

## Limitations
Relies on the kitty extended keyboard protocol for proper movement, so will only be compatible with terminals that support that protocol. If you use Alacritty, iTerm2, kitty (of course), or perhaps even Windows Terminal (ew) > v1.25 (according to the AI overview, which is never wrong), you should be good. Otherwise, switch to a better terminal.


## Running
No packaged binaries just yet. Just download the source and run:
```sh
# curl www.virus.com | sh # just kidding
git clone https://github.com/arekouzounian/delve/delve.git
cd crates/delve
cargo run --release # don't forget release mode or else performance is brutal

# Alternatively, if you're on kitty with remote-control enabled, you can use the run.sh script.
# This script takes in a command line argument (font size) and resizes your terminal to that font size
# for higher resolution.
cd crates/delve && ./run.sh 2
```

## Controls

WASD for movement, arrow keys for looking up/down/left/right.

Also, you can ascend by holding left shift, and descend by holding left control.

