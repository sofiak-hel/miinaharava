//! General module for ai related functions, just mostly a home for the
//! [ponder]-function.

use miinaharava::minefield::{Cell, Coord, Minefield};
use rand::seq::SliceRandom;

use crate::csp::ConstaintSatisficationState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Represents a single decision
pub enum Decision<const W: usize, const H: usize> {
    /// Flag this coordinate as a mine
    Flag(Coord<W, H>),
    /// Reveal this coordinate
    Reveal(Coord<W, H>),
}

/// Ponder the orb (orb being the [Minefield])
///
/// Looks at the minefield at the current state of things and returns a list of
/// decisions based on it.
pub fn ponder<const W: usize, const H: usize>(minefield: &Minefield<W, H>) -> Vec<Decision<W, H>> {
    let mut first_ponder = true;
    'outer: for row in minefield.field {
        for cell in row {
            if cell != Cell::Hidden {
                first_ponder = false;
                break 'outer;
            }
        }
    }

    if first_ponder {
        // On the first turn, pick one of the corners randomly.
        let mut rng = rand::thread_rng();

        vec![Decision::Reveal(
            *vec![
                Coord(0, 0),
                Coord(W - 1, 0),
                Coord(0, H - 1),
                Coord(W - 1, H - 1),
            ]
            .choose(&mut rng)
            .unwrap(),
        )]
    } else {
        let state = ConstaintSatisficationState::from(minefield);
        state.solve_trivial_cases().unwrap()
    }
}
