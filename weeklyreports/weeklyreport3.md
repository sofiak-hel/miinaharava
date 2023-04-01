# Weekly Raport 3
This week was busy for me, so I did not have time to work on the AI as I had
intended. Wednesday was an emergency, so I couldn't work on that day. I also
realized that after this week I only have one week until peer-reviews begin, and
I wanted to have the proper testing environments done for the peer-reviewers,
which was priority for this week, because I did not know how much time it would
actually take to set them up. 



1. What have you done this week? & 2. How has the project progressed?
> I was able to hopefully finish the user-interface part of the project, meaning
> the windowed and the headless versions are now as they were meant to be, and I
> can now fully focus on making the AI better and optimizing stuff.
3. What did you learn this week / today?
> I learned what `cargo bench` does somewhat and learned how to set that up. It
> seems pretty useful, but maybe less so for a project like this where the
> individual parts are difficult to test separately.
4. What has been unclear or problematic? Please answer this question truthfully, as this is something the course assistant may be able to help with.
> The course timings after week 4 are actually not very clear. From what it
> seems, week 5 and 6 are meant solely for peer-reviewing, does this mean, that
> we're not supposed to code the project further during that time? Are we just
> supposed to do it on another branch? Or what is the deal here with that
> exactly? 

> I'm also unsure if I was supposed to copy-and-paste an image form
> codecov.io or something similar for the testing document. The requirement for
> image/table seems weird, at least for my case, when the whole thing is hosted
> on `codecov.io` interactibly.

> One thing I also noticed, none of the weeks are currently scheduling on when
> we should be writing the user guide -document and implementation document. I
> haven't done so so far simply because no week suggested I do it, but now I'm
> unsure. Also; is a good README.md (such as the one currently) sufficient for
> the User Guide -document?
5. What next?
> Now that the project aside from the AI mostly complete, I will finally have properly time to work on the AI.

## Time usage this week

### Sat 01.04.2023 (8h)

#### 16:00 - 17:00 (1h)
- Added Matrix Struct for convenience
- Prepared moving AI to separate thread

#### 17:00 - 21:15 (4h 15min)
- Moved AI to separate thread
- Made the AI ui somewhat easy to read, added controls.

#### 21:15 - 00:00 (2h 45min)
- Added headless testing and proper benchmarking
- Added documentation
- Fixed builds
- Wrote testing document