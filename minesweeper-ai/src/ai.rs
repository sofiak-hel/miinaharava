//! General module for ai related functions, just mostly a home for the
//! [ponder]-function.

use miinaharava::minefield::{Cell, Coord, GameState, Minefield};
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
pub fn ponder<const W: usize, const H: usize>(
    minefield: &Minefield<W, H>,
) -> Option<Vec<Decision<W, H>>> {
    if minefield.game_state() != GameState::Pending {
        return None; // Early returns are evil :(
    }

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
        Some(vec![Decision::Reveal(
            *vec![
                Coord(0, 0),
                Coord(W - 1, 0),
                Coord(0, H - 1),
                Coord(W - 1, H - 1),
            ]
            .choose(&mut rng)
            .unwrap(),
        )])
    } else {
        let state = ConstaintSatisficationState::from(minefield);
        let mut decisions = state.solve_trivial_cases().unwrap();
        // if decisions.is_empty() {
        //     decisions.push(guess(minefield))
        // }
        Some(decisions)
    }
}

/// Make a purely random guess. At least for now, this function is meant for use
/// simply so that the game will never stagnate entirely.
pub fn guess<const W: usize, const H: usize>(minefield: &Minefield<W, H>) -> Decision<W, H> {
    // let corners = vec![
    //     Coord(0, 0),
    //     Coord(W - 1, 0),
    //     Coord(0, H - 1),
    //     Coord(W - 1, H - 1),
    // ]
    // .iter();
    // .filter(|coord| minefield);

    Decision::Reveal(Coord::random())
}
