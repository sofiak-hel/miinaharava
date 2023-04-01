
# Mine sweeper

[![Rust](https://github.com/sofiak-hel/minesweeper/actions/workflows/rust.yml/badge.svg)](https://github.com/sofiak-hel/minesweeper/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/sofiak-hel/minesweeper/branch/main/graph/badge.svg?token=LK0NOTUKGI)](https://codecov.io/gh/sofiak-hel/minesweeper)

This repository contains two Rust projects:  
  - [miinaharava](./miinaharava/), a minesweeper implementation written in Rust, contains both a human-playable binary and a `lib.rs` in order to expose it for an AI to use.  
  - [minesweeper-ai](./minesweeper-ai/), an AI that will attempt to play minesweeper using the aforementioned minesweeper implementation. **This project is the relevant project for Tiralabra**, but the other one contains some tests and code coverage as well.

## Documents
- [Project specification document](./documentation/projectspecification.md)

## Weekly reports
1. [Report 1](./weeklyreports/weeklyreport1.md)
2. [Report 2](./weeklyreports/weeklyreport2.md)

## Performance and benchmarking
To do performance and benchmark-testing, you can run `minesweeper-ai` in
headless-mode, which will give accurate statistics for specifically how much
time was spent ie. running AI.

Alternatively you can run `cargo bench` which will run a few different
benchmarks for each difficulty:
- How long does it take to generate a minefield, and solve it (including time
  spent revealing)
- How long does it take to simply generate the minefield
- How long does it to reveal a mine field and then reveal a random coordinate

`cargo bench` is technically most likely more accurate, but since there is no
specific benchmark for how long only the revealing takes or how long is only
spent running AI, it is not optimal for all use cases.

## Building and running manually
For this you need at least Cargo/Rust version `1.68.1`. Recommended way to
install and update rust is using [rustup](https://rustup.rs/).

1. To build the project run `cargo build --release`
2. To run the human-playable `miinaharava` run `cargo run --release -p miinaharava`
3. To run the AI driven minesweeper, run `cargo run --release -p minesweeper-ai`

### AI version specific instructions:
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

#### Windowed
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

#### Headless

Headless version also has optional arguments for
- `--games <number of games>` 
- `--seconds <the number of seconds to run games>`.

Use `--help` for more detail.

## Testing and coverage
Testing can be run simply by running `cargo test` in the root folder of this
repository.  
To get coverage in the terminal:
1. Install tarpaulin with `cargo install cargo-tarpaulin`
2. Run `cargo tarpaulin --exclude-files='miinaharava/src/minefield_renderer.rs,miinaharava/src/game.rs,miinaharava/src/main.rs,minesweeper-ai/src/main.rs,minesweeper-ai/src/thread_controller.rs' --engine Llvm`

Both here and in codecov.io the three files in the above flag are excluded
because they are highly related to rendering and user input, difficult to test
and mostly irrelevant to this project and therefore intentionally not tested.