use miinaharava::minefield::{Coord, Minefield};

use crate::ai::{ponder, Decision};

#[test]
fn first_ponder_is_a_corner() {
    for _ in 0..100 {
        let minefield = Minefield::<10, 10>::generate(10).unwrap();
        let decisions = ponder(&minefield);
        let decision = decisions.get(0).unwrap();
        assert!(matches!(decision, Decision::Reveal(Coord(0 | 9, 0 | 9))));
    }
}
