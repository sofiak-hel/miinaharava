
# Weekly report 4

1. What have you done this week?
> Aside from trivial solving I implemented pretty much the entire AI during
> these two weeks.
2. How has the project progressed?
> I think this project aside from some optimizations, possible improvements to
> gain a higher win-rate, is pretty much done now I think.
3. What did you learn this week / today?
> I've learned so much I'm not sure I can remember all to be honest. I've also
> scrapped a lot of the stuff I've learned in favour of other methods. The
> biggest thing I can come up with though, is that apparently cloning stuff is
> very heavy, and also apparently pointers are big and sometimes it is more
> efficient to just copy small things than to pass pointers.
4. What has been unclear or problematic? Please answer this question truthfully, as this is something the course assistant may be able to help with.
> I have absolutely no idea why my win-rates are as low as they are. The only
> thing I don't do according to the two papers is the explicit detection of
> crap-shoots, which should not affect win-rates at all.

> Second thing that actually came up is that apparently I'm not allowed to use
> standard library functions for sorting. I do however at least for now, and I
> do rely on it quite a bit too, what's the recommended way of proceeding with
> this? What part of the sorting do I need to implement? Vec::sort, `Ord`
> implementation or both? I'm guessing only Vec::sort, but I'm really not sure.
> I do implement a custom `Ord` implementation in some places already though.

> Also, if `Vec::sort` is forbidden, what else is? Is `Vec::dedup` for example?
5. What next?
> Well, next is at least the review process. Probably some optimization as well,
> and likely the swapping of the standard libary `Vec::sort`.


## Time spent (46h 50 min)

### Wed 05.04.2023 (11h 20 min)

#### 17:30 - 04:50 (11h 20 min)
- Added reduce function that should be able to solve all cases that don't
  require guessing.
- Attempted to implement optimization so that constraints don't need to be
  regenerated every time, ended up scrapping it

### Thu 06.04.2023 (5h)

#### 18:00 - 23:00 (5h)
- Made tests for reduce-function
- Optimized a bunch, did the thing that I tried to do on wednesday but failed.

### Sat 08.04.2023 (10h 10 min)

#### 14:50 - 01:00 (10h 10 min)
- Optimized a lot, wrote a bunch of tests, fixed a few bugs
- Started to write the actual algorithm part that demands finding all viable
  solutions :( my gametime is dying someone please help me balance my syscalls

### Wed 12.04.2023 (7h 30 min)

#### 15:50 - 21:10 (5h 20 min)
- Made tests for backtracking
- Smashed a few annoying bugs
- Optimized!

#### 22:20 - 00:30 (2h 10 min)
- Add find_trivial_solutions for the solution lists
- Add a bunch of tests, fix a bunch of bugs..

### Sat 15.04.2023 (9h)

#### 17:00 - 02:00 (9h)
- Added educated guessing in all it's glory

### Sun 16.04.2023 ( 3h 50 min )

#### 14:40 - 18:30 (3h 50 min)