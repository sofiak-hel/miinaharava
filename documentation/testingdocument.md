# Testing Document

[![Rust](https://github.com/sofiak-hel/minesweeper/actions/workflows/rust.yml/badge.svg)](https://github.com/sofiak-hel/minesweeper/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/sofiak-hel/minesweeper/branch/main/graph/badge.svg?token=LK0NOTUKGI)](https://codecov.io/gh/sofiak-hel/minesweeper)

Tests are currently being run automatically in CI, code coverage is being
gathered via [`tarpaulin`](https://github.com/xd009642/tarpaulin) and said
coverage is uploaded to
[`codecov.io`](https://codecov.io/gh/sofiak-hel/minesweeper) for hosting of the
actual files that can be viewed. Above badges show easily whether tests pass and
how much of the tested filebase is covered.

A more detailed viewing of the coverage can be found here:
https://codecov.io/gh/sofiak-hel/minesweeper

## What is currently tested
As the program is made of many separate testable bits, all the bits are
obviously tested as thoroughly as possible separately, or as much as I've had
the energy to write tests for them:

### Miinaharava
Miinaharava is the game itself at the base, which of course needs to be tested
as well.

1. Minefield generation is tested, so that the correct amount of mines is always
   present.
2. Generation is also tested if too many mines are given.
3. Game state's checking is tested so that it always returns the correct state
   of the game.
4. Revealing is tested, so that it always reveals hidden tiles recursively
   correctly.
5. Revealing and flagging is tested, so that it is impossible to cheat through
   the API.

### Minesweeper-ai
Here are the tests for the actual AI part, which I will again divide into a few
subsections because it is made of quite a bit of different modules.

#### CoordSet
Coord set is not very thoroughly tested, because it's implementation is fairly
trivial. However it's transpose function is thoroughly tested, and it does have
a few tests in the form of documentation examples, which should account for
something.

#### Backtracking
The backtracking algorithm is much more thoroughly tested:

1. It is tested that `find_ordered` always returns the correct list of variables
   with the correct indices for constraints. This is important for the other
   tests.
2. Backtracking only returns valid solutions
3. Backtracking always returns the correct solution
4. When solutions are put together, invalid solutions (too many mines) are
   thrown away

##### Solutions
Solutions in this context represents the code that manages entire solution sets,
once backtracking has found them.

1. Transpose is tested, meaning if solution list has N solutions that are M long, transpose will always return a transposed M-solutions that are N-long
2. Solution list's trivial finder is able to spot corrently trivial answers, so
   all of the coords that are either 1's or 0's in all solutions.
3. Solution list's min and max mine counts are always correct
3. Solution list and a list of solution lists is always able to find the best
   guess, so the guess that has the highest propability of being a 0

#### Constraints
Individual constraints like the CoordSet are not really tested, for the same
reason. I haven't had the energy to write thorough tests for them and the
implementation is fairly trivial and unlikely to not work.

#### Constraint Sets
Constraint Sets on the other hand are tested very thoroughly.

1. It's tested that combining sets via drain and combine works correctly.
2. Reduce is tested thoroughly so that after reduce there is no possible subsets
   remaining.
3. Trivial solving is tested so that it is always able to find the trivial solutios; constraints that have the same number of variables as it's label, or if it's label is 0, meaning all of the variables have to be 0.
4. Reduce and trivial solving are tested to be idempotent, meaning they have no
   effect if they are executed twice in a row.
5. Splitting constraint sets is tested so that after the split there should
   never be an intersection between the new sets.

#### The AI itself
The AI is tested somewhat thoroughly, but one of the main functions of the
AI,`perform_educated_guess` is not tested because it is exeedingly difficult to
test and I haven't had the time to think of how to test it. However:

1. `ponder` is tested so that trivial fields are able to be solved.
2. Inserting constraints into the sets are tested, so that trivial constraints
   are solved as they are inserted.
3. `ponder` with reveals is tested, so that it will handle reveals correctly,
   solve and new trivial constraints and reduce the sets as much as possible
   with the new information at hand.
4. Guessing is tested so that guessing first always tries a corner, then an edge
   and only after neither work, in the middle. If the guess has to be in the
   middle, guess currently doesn't do any further heuristics for it.

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


## Running tests and coverage manually
Testing can be run simply by running `cargo test` in the root folder of this
repository.  
To get coverage in the terminal:
1. Install tarpaulin with `cargo install cargo-tarpaulin`
2. Run `cargo tarpaulin --exclude-files='miinaharava/src/minefield_renderer.rs,miinaharava/src/game.rs,miinaharava/src/main.rs,minesweeper-ai/src/main.rs,minesweeper-ai/src/thread_controller.rs,minesweeper-ai/benches/ai.rs' --engine Llvm`

Both here and in codecov.io the files in the above flag are excluded because
they are highly related to rendering and user input, difficult to test and
mostly irrelevant to this project and therefore intentionally not tested.
