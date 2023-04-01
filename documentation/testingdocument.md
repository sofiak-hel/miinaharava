# Testing Document

[![Rust](https://github.com/sofiak-hel/minesweeper/actions/workflows/rust.yml/badge.svg)](https://github.com/sofiak-hel/minesweeper/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/sofiak-hel/minesweeper/branch/main/graph/badge.svg?token=LK0NOTUKGI)](https://codecov.io/gh/sofiak-hel/minesweeper)

Tests are currently being run automatically in CI, code coverage is being
gathered via `tarpaulin` and said coverage is uploaded to `codecov.io` for
hosting of the actual files that can be viewed. Above badges show easily whether
tests pass and how much of the tested filebase is covered.

A more detailed viewing of the coverage can be found here:
https://codecov.io/gh/sofiak-hel/minesweeper

## Files that are currently tested:
The following files are being intentionally tested. Below them is also a link to
the tests most specific to that file, where the tests have been named and
documented in a way that should be somewhat explanatory on what is being tested.
Below the link to the tests is a more brief explanation to what is tested in the
file.

- [miinaharava/src/minefield.rs](../miinaharava/src/minefield.rs)
    - [tests](../miinaharava/src/tests/minefield.rs)
    - Tests that minefield generation works correct and that reveal / flag
      functions work as intended, and that cheating is not possible.
- [minesweeper-ai/src/ai.rs](../minesweeper-ai/src/ai.rs)
    - [tests](../minesweeper-ai/src/tests/ai.rs)
    - Tests mostly the `ponder`-method, will probably contain more tests in the future
- [minesweeper-ai/src/csp.rs](../minesweeper-ai/src/csp.rs)
    - [tests](../minesweeper-ai/src/tests/csp.rs)
    - Also tests using the `ponder`-method, but also tests that the constraint
      generation works correctly and that trivial cases according to said
      constraints works as intended.



## Files that are excluded from code-coverage (at least for now)
- [miinaharava/src/game.rs](../miinaharava/src/game.rs)
- [miinaharava/src/minefield_renderer.rs](../miinaharava/src/minefield_renderer.rs)
- [miinaharava/src/main.rs](../miinaharava/src/main.rs)
- [minesweeper-ai/src/main.rs](../minesweeper-ai/src/main.rs)
- [minesweeper-ai/src/thread_controller.rs](../minesweeper-ai/src/thread_controller.rs.rs)
    - This one could be separated into at least two files, it has some elements
      that could in theory be tested, but it remains to see if I have the time
      to implement said tests, since they are not entirely relevant to the AI
      itself.

All other files excluded for testing, other than `thread_controller.rs` have
been excluded from code-coverage because they are mostly scaffolding-code and
code related directly to rendering thus difficult to test and somewhat
irrelevant to the project as a whole.


## Running tests and coverage manually
Testing can be run simply by running `cargo test` in the root folder of this
repository.  
To get coverage in the terminal:
1. Install tarpaulin with `cargo install cargo-tarpaulin`
2. Run `cargo tarpaulin --exclude-files='miinaharava/src/minefield_renderer.rs,miinaharava/src/game.rs,miinaharava/src/main.rs,minesweeper-ai/src/main.rs,minesweeper-ai/src/thread_controller.rs' --engine Llvm`

Both here and in codecov.io the files in the above flag are excluded because
they are highly related to rendering and user input, difficult to test and
mostly irrelevant to this project and therefore intentionally not tested.