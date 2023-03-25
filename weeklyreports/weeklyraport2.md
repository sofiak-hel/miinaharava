# Weekly Raport 2
2. What have you done this week?
> I wrote up the base minesweeper that the AI will use in order to solve the
> game. I also made a small `main.rs` file for it, so that it is actually human
> playable as well. I also began work with the CSP algorithm itself, wrote up
> the structures I will use, and made a solver for the trivial cases. I also
> wrote tests and set up codecov.io for the project.

2. How has the project progressed?
> Well I think, in retrospective when reading what I was supposed to achieve
> this week I think I may have even made the AI a little bit too far, but I am
> unsure since the weekly goals are a bit unclear and seem very small compared
> to what I can do in a week.

3. What did you learn this week / today?
> I learned how to use SDL2 and how to generate code coverage for Rust. I
> actually forgot to write in my last weekly raport, but I already did kind of
> leard the basic gist of the algorithm last week, enough at least that I was
> able to get to this point.

4. What has been unclear or problematic? Please answer this question truthfully, as this is something the course assistant may be able to help with.
> Not much so far, code coverage was a little bit confusing to get going, as
> I've never done rust code coverage before.

5. What next?
> Next I will read more on the paper about CSP solving and the specific CSP
> subset algorithm and will start implementing that I suppose so that the
> algorithm AI will get more performant and so that it will be able to solve
> less trivial cases

## Time usage this week:

### wed 22.03.23 (8h 30 min)
#### 17:00 - 19:15 (2:15)
- Minefield
- SDL window creation
#### 19:15 - 21:50 (2:35)
- Implement minesweeper as a standalone game
#### 21:50 - 01:30 (3:40)
- Improve UI, and usability
- Rearchitecture the whole thing for future-proofing
- Add more difficulty levels to the testing

### sat 25.03.23 (6h 10 min)
#### 16:00 - 16:40 (0:40)
- Code coverage setting up

#### 16:40 - 18:45 (2:05)
- Write tests and documentation for miinaharava

### 18:45 - 22:10 (3:25)
- Write CSP trivial solver and tests and documentation for it.