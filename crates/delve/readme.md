# Delve
---
A real-time ASCII art 3D rendering engine, using raycasting.

The goal is to layer a simple game atop this engine, so it may later involve physics and all that stuff. This is a simple project that's just for fun. No AI allowed to write code, although Claude was pretty helpful in explaining a lot of the vector math, and for establishing a preliminary roadmap in [[roadmap.md]].

The engine is designed to have little dependencies; currently, only `crossterm`, and maybe `rayon` in the future if I want to squeeze out some extra performance.


## Limitations
Relies on the kitty extended keyboard protocol for proper movement, so will only be compatible with terminals that support that protocol. If you use Alacritty, iTerm2, kitty (of course), or perhaps even Windows Terminal (ew) > v1.25 (according to the AI overview, which is never wrong), you should be good. Otherwise, switch to a better terminal.


## Running
No packaged binaries just yet. Just download the source and run:
```sh
# curl www.virus.com | sh # just kidding
git clone https://github.com/arekouzounian/delve/delve.git
cd delve
cargo run --release # don't forget release mode or else performance is brutal
```

## Controls

WASD for movement, arrow keys for looking up/down/left/right.

Also, you can ascend by holding left shift, and descend by holding left control.

