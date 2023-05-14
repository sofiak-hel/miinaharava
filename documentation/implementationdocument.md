# Implementation document

## Performance
For the following analysis, for the manual tests all three difficulties were run
for 60 seconds with 10 threads.

For the context of these results, difficulties are explicitly explained only
here for convenience:
1. Easy is a `10x10` field with `10` mines
2. Intermedaite is a `16x16` field with `40` mines
3. Expert is a `30x16` field with `99` mines.

### Performance:

These performance tests were run with an [AMD Ryzen
5900X](https://www.amd.com/en/products/cpu/amd-ryzen-9-5900x) processor, running
the same tests with a different processor might yield different results.

As it is, I think these results are already pretty good, but could possibly be
improved upon.

These numbers represent how long it took to generate a field, solve it using the
AI and process revealing and flagging of the tiles that the AI decided. So it
represents the total time it takes from generation to victory or loss.

|               | easy     | intermediate | expert    |
|---------------|:--------:|:------------:|:---------:|
| Manual        | `18.5µs` | `93.7µs`     | `424.2µs` |
| `cargo bench` | `16.8µs` | `83.6µs`     | `404.2µs` |


### Victory-rates

As we can see from thes results the victory-rates are not quite as good as what
the papers had promised, however from what I can personally tell I implemented
all of the features reported by both papers described in my project
specification, except for the explicit detection of crap-shoots, which should
not affect win-rates in any measure. I am personally confused as to how these
win-rates are achieveavable, and looking back at the papers analytically I have
to only conclude that the setups from which these are gained are rather
inconsistent and unreliable, making comparing these win-rates difficult to impossible.

Either way I would say that these win rates are fairly good in the end.

|               | easy     | intermediate | expert   |
|---------------|:--------:|:------------:|:--------:|
| Manual        | `85.70%` | `64.65%`     | `27.70%` |
| Expected      | `91.25%` | `75.94%`     | `32.90%` |


### Raw results:

#### Easy
```
  Total time spent: 60.0s (x 10 thread(s))
    AI thinking: 413.6s (13.6µs avg.)
    flagging + revealing: 147.9s (4.9µs avg.)
    board generating: 4.7s (154.0ns avg.)

  Total games played: 30395874
    Victories: 26049634, (85.70122%)
    Losses: 4346240, (14.298783%)

Total guesses:
  Amount of guesses: 43332711
  Successful: 38986470 (89.97007%)
  Average guess success: 90.35286%
  Average amount of guesses: 1.43
```

#### Intermediate
```
  Total time spent: 60.0s (x 10 thread(s))
    AI thinking: 491.7s (80.1µs avg.)
    flagging + revealing: 83.7s (13.6µs avg.)
    board generating: 3.1s (508.0ns avg.)

  Total games played: 6137506
    Victories: 3967756, (64.64769%)
    Losses: 2169750, (35.352306%)

Total guesses:
  Amount of guesses: 12940061
  Successful: 10770311 (83.23231%)
  Average guess success: 84.231255%
  Average amount of guesses: 2.11
```

#### Expert
```
  Total time spent: 60.1s (x 10 thread(s))
    AI thinking: 555.9s (399.6µs avg.)
    flagging + revealing: 34.3s (24.6µs avg.)
    board generating: 1.8s (1.3µs avg.)

  Total games played: 1391096
    Victories: 385333, (27.699957%)
    Losses: 1005763, (72.30004%)

Total guesses:
  Amount of guesses: 5009529
  Successful: 4003766 (79.923004%)
  Average guess success: 80.35041%
  Average amount of guesses: 3.60
```

## O-notation analysis
I've procrastinated making the O-notation analysis for so long, that I don't
really have the energy to make one in the end. The be clear, O-notation analysis
for an algorithm of this size and complexity might also simply be not
applicable, as there are so many if-statements and branches to the algorithm,
that there are many different approaches to how someone could craft one on the
end. If I were to make one, I would probably end up making simply a worst-case
and a best-case scenario analysis separately without an attempt to make a
"generic" analysis, and even then I would have to make a O-notation based on
performing a guess and constructing the constraint-set separately, hard work.

One simple point I can point out though, according to [this
paper](https://www.cs.toronto.edu/~cvs/minesweeper/minesweeper.pdf) the
backtracking algorithm is roughly exponential to the difference in amount of
constraints and variables, so if `c` is the amount of constraints and `v` is the
amount of variables, it would probably resemble something alike to `O(n^(c-v))`.
Already somewhat complicated.

## Project structure
The project is split into two different entire programs.
1. `miinaharava` is the actual minesweeper game with no AI, it is human playable.
2. `minesweeper-ai` is the AI part of this project, implementing two different
   ways to empirically test the functionality of the AI against the `miinaharava`-game:
    1. You can run it on windowed mode, where the games are run a lot more
       slowly, but it is an interesting way to see how the AI solves the games
    2. You  can also run it headlessly, where the games are solved as fast as
       possible and statistics about said games is shown only after they are
       done.

### Minesweeper-ai
As this is the relevant part of the project, this is where I will explain the
project structure more specifically.

- The program starts at `main.rs` where a at least one thread is launched for
  the ai to solve on. The thread is managed on `thread_controller.rs`. On
  windowed mode, only one game can be run at the same time, so only one thread
  can be used. These files mostly contain the user experience part of the program.
- `ai` folder contains all of the actual AI part, where it all starts in `ai/mod.rs`
    - `ai/mod.rs` contains high-level `ponder` function which takes reveals as
      an argument, which is something `miinaharava`'s `reveal` function API
      returns. This is simply used to update the AI on what new information has
      appeared. `ai/mod.rs` also contains some code about doing educated guesses
      and a function for doing a generally random guess. The main struct here is
      the `CSPState` which is the state of the AI and contains and manages a
      coupled set of constraints.
        - `ai/constraint_sets.rs` then contains the actual code for the coupled
            set of constraints and a lot of functions or anging it, like
            inserting. Code of individual constraints is contained in
            `ai/constraints.rs`
        - `ai/coord_set.rs` contains simply a "more optimized" and specialized
          version of a HashSet of constraints.
        - `ai/backtracking`-folder then contains all of the code for the actual
          backtracking algorithm where:
            - `ai/backtracking/mod.rs` contains only the backtracking algorithm,
              so the code for finding all viable solutions
            - `ai/backtracking/solutions.rs` contains the code for managing the
              actual solution sets after the backtracking algorithm has found
              them.

#### Flow:
1. The program starts at `ponder` where it tries to do simple trivial solving
   and reducing with the constraint sets if possible.
2. If the program is not able to find anything to do with simple trivial solving
   and reducing it uses a backtracking algorithm and guessing to find all of the
   viable solutions of each constraint set.
3. Once all viable solutions are found, they are analyzed, invalid solutions are
   discarded and propabilities, that a certain variable is 0 are calculated.
4. All the propabilities are put together and the highest propability is chosen
   and guessed.
5. Sometimes unconstrained variables have actually a better chance of not
   containing a mine than any of the propabilities previously found, in this
   case the program simply guesses from the unconstrained variables.  