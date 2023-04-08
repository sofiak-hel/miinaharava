//!: TODO: Docs

use arrayvec::ArrayVec;
use miinaharava::minefield::{Cell, Coord, Matrix, Minefield, Reveal};
use rand::seq::SliceRandom;

use self::{constraint_sets::CoupledSets, constraints::Constraint};

pub mod constraint_sets;
pub mod constraints;
pub mod coord_set;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CellContent {
    Known(bool),
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Represents a single decision
pub enum Decision<const W: usize, const H: usize> {
    /// Flag this coordinate as a mine
    Flag(Coord<W, H>),
    /// Reveal this coordinate
    Reveal(Coord<W, H>),
}

pub type KnownMinefield<const W: usize, const H: usize> = Matrix<CellContent, W, H>;

/// General state used for solving Constraint Satisfication Problem
#[derive(Debug, Clone, Default)]
pub struct ConstraintSatisficationState<const W: usize, const H: usize> {
    /// List of label-mine-location-constraints for a given state
    pub constraint_sets: CoupledSets<W, H>,
    pub known_fields: KnownMinefield<W, H>,
}

impl<const W: usize, const H: usize> ConstraintSatisficationState<W, H> {
    /// TODO: Docs
    pub fn ponder(
        &mut self,
        reveals: Vec<Reveal<W, H>>,
        minefield: &Minefield<W, H>,
    ) -> Vec<Decision<W, H>> {
        if reveals.is_empty() {
            // Guess here maybe someday
            vec![guess(minefield)]
        } else {
            self.handle_reveals(reveals, minefield)
        }
    }

    /// TODO: Docs
    pub fn handle_reveals(
        &mut self,
        reveals: Vec<Reveal<W, H>>,
        minefield: &Minefield<W, H>,
    ) -> Vec<Decision<W, H>> {
        let mut decisions = Vec::new();
        for (coord, cell) in &reveals {
            self.known_fields
                .set(*coord, CellContent::Known(*cell == Cell::Mine))
        }
        for (coord, cell) in &reveals {
            if let Cell::Label(num) = cell {
                let mut neighbors = ArrayVec::new();
                for neighbor in coord.neighbours().iter() {
                    // TODO: Possible optimization for later
                    // match minefield.field.get(*neighbor) {
                    //     Cell::Flag => num -= 1,
                    //     Cell::Hidden => neighbors.push(*neighbor),
                    //     _ => {}
                    // };
                    if matches!(minefield.field.get(*neighbor), Cell::Flag | Cell::Hidden) {
                        neighbors.push(*neighbor);
                    }
                }
                if !neighbors.is_empty() {
                    let constraint = Constraint {
                        label: *num,
                        variables: neighbors,
                    };
                    if let Some(res) = self
                        .constraint_sets
                        .insert(constraint, &mut self.known_fields)
                    {
                        decisions.extend(res);
                    }
                }
            }
        }

        for set in &mut self.constraint_sets.0 {
            if !decisions.is_empty() {
                decisions.extend(set.clear_known_variables(&self.known_fields));
            }
            set.reduce();
        }
        self.constraint_sets.check_splits();

        let mut prev_decisions = decisions.len();
        while {
            for set in &mut self.constraint_sets.0 {
                let mut res = set.solve_trivial_cases(&mut self.known_fields);
                res.extend(set.clear_known_variables(&self.known_fields));
                if !res.is_empty() {
                    set.reduce();
                }
                decisions.extend(res);
            }
            self.constraint_sets.check_splits();
            decisions.len() != prev_decisions
        } {
            prev_decisions = decisions.len()
        }

        decisions.sort();
        decisions.dedup();

        decisions
            .into_iter()
            .filter(|decision| match decision {
                Decision::Flag(c) => minefield.field.get(*c) == Cell::Hidden,
                Decision::Reveal(c) => {
                    !matches!(minefield.field.get(*c), Cell::Empty | Cell::Label(_))
                }
            })
            .collect()
    }
}

/// Make a purely random guess. At least for now, this function is meant for use
/// simply so that the game will never stagnate entirely.
///
/// Still not very good, but at least it's trying
pub fn guess<const W: usize, const H: usize>(minefield: &Minefield<W, H>) -> Decision<W, H> {
    let corners: Vec<Coord<W, H>> = vec![
        Coord(0, 0),
        Coord(W as u8 - 1, 0),
        Coord(0, H as u8 - 1),
        Coord(W as u8 - 1, H as u8 - 1),
    ]
    .into_iter()
    .filter(|coord| minefield.field.get(*coord) == Cell::Hidden)
    .collect();

    if !corners.is_empty() {
        let mut rng = rand::thread_rng();
        Decision::Reveal(*corners.choose(&mut rng).unwrap())
    } else {
        let mut coord = Coord::random();
        while minefield.field.get(coord) != Cell::Hidden {
            coord = Coord::random();
        }
        Decision::Reveal(coord)
    }
}
