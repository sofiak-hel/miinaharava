use miinaharava::minefield::{Coord, Minefield};

use crate::ai::{guess, Decision};

#[test]
fn guess_is_a_corner() {
    for _ in 0..100 {
        let minefield = Minefield::<10, 10>::generate(10).unwrap();
        let decision = guess(&minefield);
        assert!(matches!(decision, Decision::Reveal(Coord(0 | 9, 0 | 9))));
    }
}
