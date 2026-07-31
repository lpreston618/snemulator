# Snemulator: Super Nintendo emulator written in Rust

## Features

- Cycle-accurate emulation
- Debug viewer including:
    - CPU trace (breakpoints!)
    - Graphics data visualizations (tilemaps, character data)
    - Complete memory dumps
    - Sound data visualizations
- Save states
- Controller remapping


## Running the code

To run the app, the ```cargo``` tool is required:

```cargo run --bin snemulator```

To compile with debug features enabled (super cool for looking at game internals), run

```cargo run --bin snemulator --features=debug```

Easy installation is a WIP.

ROM files not included. You'll need to source those on your own.

