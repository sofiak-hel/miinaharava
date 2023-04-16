
# User Guide
Ready built binaries for Linux and Windows can be found on Releases here
https://github.com/sofiak-hel/minesweeper/releases/tag/code-review-1

Running the ready built binaries as easy as:
- `./miinaharava` for the human-playable game
- `./minesweeper-ai` for the graphically appealing ai-version
- `./minesweeper-ai --headless` for the headless performance-specific ai-version

## Linux specific notes
1. When unzipping the zip for linux, the binary is not executable at first and
   you will need to run `sudo chmod +x minesweeper-ai` to run it.
2. If you are running on [Wayland](https://wayland.freedesktop.org/), you will
   need to specify environment variable `SDL_VIDEODRIVER=x11` in order to launch
   the program, so ie. `SDL_VIDEODRIVER=x11 ./minesweeper-ai`

# Building (and running) manually
For this you need at least Cargo/Rust version `1.68.1`. Recommended way to
install and update rust is using [rustup](https://rustup.rs/).

1. To build the project run `cargo build --release`
2. To run the human-playable `miinaharava` run `cargo run --release -p miinaharava`
3. To run the AI driven minesweeper, run `cargo run --release -p minesweeper-ai`

# AI version specific instructions:
Since the AI version is the one that is relevant, I will be giving more specific
instructions to the AI version here.

The AI version has a windowed mode and a headless mode. The windowed more is the
default one, but will be more difficult to use for actual performance analysis.
The headless mode is run on the command-line and does not have a visual
interface, although it will print progress messages and a statistics-message at
the end.

- To run it windowed, run `minesweeper-ai` or `cargo run --release -p
minesweeper-ai` depending on if you're running a ready binary or building manually.
- To run it headlessly, run `minesweeper-ai --headless` or `cargo run --release -p
minesweeper-ai -- --headless` depending on if you're running a ready binary or building manually.

Both versions accept also `--difficulty <easy/intermediate/expert>` to change
the difficulty the program is launched with.

## Windowed
Windowed mode has a small text UI build into the side-panel that is meant to
give some perspective on what is actually happening.

First on the sidebar are three values
- `X, Y` The size of the grid.
- `N mines` tells simply the amount of mines
- Real time timer that is paused and reset with hotkeys, but does not reflect
  the amount of time the AI needed.

First are the keybinds for the program:
- `1` to reset the game with the `Easy` difficulty
- `2` for `Intermediate`
- `3` for `Expert`
- `Spacebar` will pause the execution and the realtime timer
- `Arrow keys Up/Down` will increase or decrease the amount of delay between any
  actions on the screen. This will not increase the time that is spent and
  therefore does not reflect on the AI time spent or average game duration. It
  is simply to make the process go as fast or slow as you like.

Finally are some statistics:
- Games played is the total amount of games played so far
- The number below that tells how many of those games were victories.
- Third row tells how much processing time the AI has needed so far (not
  accounting the real time delays or time spent generating minefields or
  revealing areas)
- Final row tells how much of that above time was spent on one game on average.

## Headless

Headless version also has optional arguments for
- `--games <number of games>` 
- `--seconds <the number of seconds to run games>`.

Use `--help` for more detail.