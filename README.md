
# Mine sweeper

[![Rust](https://github.com/sofiak-hel/minesweeper/actions/workflows/rust.yml/badge.svg)](https://github.com/sofiak-hel/minesweeper/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/sofiak-hel/minesweeper/branch/main/graph/badge.svg?token=LK0NOTUKGI)](https://codecov.io/gh/sofiak-hel/minesweeper)

This repository contains two Rust projects:  
  - [miinaharava](./miinaharava/), a minesweeper implementation written in Rust, contains both a human-playable binary and a `lib.rs` in order to expose it for an AI to use.  
  - [minesweeper-ai](./minesweeper-ai/), an AI that will attempt to play minesweeper using the aforementioned minesweeper implementation. **This project is the relevant project for Tiralabra**, but the other one contains some tests and code coverage as well.

## Documents
- [Project specification document](./documentation/projectspecification.md)
- [Testing document](./documentation/testingdocument.md)
- [User Guide](./documentation/userguide.md)

## Weekly reports
1. [Report 1](./weeklyreports/weeklyreport1.md)
2. [Report 2](./weeklyreports/weeklyreport2.md)
3. [Report 3](./weeklyreports/weeklyreport3.md)
4. [Report 4](./weeklyreports/weeklyreport4.md)

## Performance and testing

### Performance and benchmarking
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

### Testing and coverage
Testing can be run simply by running `cargo test` in the root folder of this
repository.  
To get coverage in the terminal:
1. Install tarpaulin with `cargo install cargo-tarpaulin`
2. Run `cargo tarpaulin --exclude-files='miinaharava/src/minefield_renderer.rs,miinaharava/src/game.rs,miinaharava/src/main.rs,minesweeper-ai/src/main.rs,minesweeper-ai/src/thread_controller.rs,minesweeper-ai/benches/ai.rs' --engine Llvm`

Both here and in codecov.io the files in the above flag are excluded because
they are highly related to rendering and user input, difficult to test and
mostly irrelevant to this project and therefore intentionally not tested.