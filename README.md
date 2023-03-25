
# Mine sweeper

[![Rust](https://github.com/sofiak-hel/minesweeper/actions/workflows/rust.yml/badge.svg)](https://github.com/sofiak-hel/minesweeper/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/sofiak-hel/minesweeper/branch/main/graph/badge.svg?token=LK0NOTUKGI)](https://codecov.io/gh/sofiak-hel/minesweeper)

This repository contains two Rust projects:  
  - [miinaharava](./miinaharava/), a minesweeper implementation written in Rust, contains both a human-playable binary and a `lib.rs` in order to expose it for an AI to use.  
  - [minesweeper-ai](./minesweeper-ai/), an AI that will attempt to play minesweeper using the aforementioned minesweeper implementation. **This project is the relevant project for Tiralabra**, but the other one contains some tests and code coverage as well.

## Building and running manually
For this you need at least Cargo/Rust version `1.68.1`. Recommended way to
install and update rust is using [rustup](https://rustup.rs/).

1. To build the project run `cargo build --release`
2. To run the human-playable `miinaharava` run `cargo run --release -p miinaharava`
3. To run the AI driven minesweeper, run `cargo run --release -p minesweeper-ai`

## Testing and coverage
Testing can be run simply by running `cargo test` in the root folder of this
repository.  
To get coverage in the terminal:
1. Install tarpaulin with `cargo install cargo-tarpaulin`
2. Run `cargo tarpaulin --exclude-files='miinaharava/src/minefield_renderer.rs,miinaharava/src/game.rs,miinaharava/src/main.rs,minesweeper-ai/src/main.rs' --engine Llvm`

Both here and in codecov.io the three files in the above flag are excluded
because they are highly related to rendering and user input, difficult to test
and mostly irrelevant to this project and therefore intentionally not tested.

## Documents
- [Project specification document](./documentation/projectspecification.md)

## Weekly reports
1. [Report 1](./weeklyreports/weeklyreport1.md)
