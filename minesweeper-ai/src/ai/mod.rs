//! TODO: Docs

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
/// Describes cell content in a known field according to the AI state
pub enum CellContent {
    /// AI thinks this cell is (true = a mine, false = not a mine)
    Known(bool),
    #[default]
    /// AI does not know what is in this cell
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Represents a single decision
pub enum Decision<const W: usize, const H: usize> {
    /// Flag this coordinate as a mine
    Flag(Coord<W, H>),
    /// Reveal this coordinate
    Reveal(Coord<W, H>),
}

/// Represents the AI state's own opinion on fields
pub type KnownMinefield<const W: usize, const H: usize> = Matrix<CellContent, W, H>;

/// General state used for solving Constraint Satisfication Problem
#[derive(Debug, Clone, Default)]
pub struct CSPState<const W: usize, const H: usize> {
    /// List of label-mine-location-constraints for a given state
    pub constraint_sets: CoupledSets<W, H>,
    /// Represents the current state of the minefield, according to the AI. Not
    /// guarenteed to be correct.
    pub known_fields: KnownMinefield<W, H>,
    /// Represents the amount of mines that have been found so far
    found_mines: u8,
}

impl<const W: usize, const H: usize> CSPState<W, H> {
    /// TODO: Docs
    pub fn ponder(
        &mut self,
        reveals: Vec<Reveal<W, H>>,
        minefield: &Minefield<W, H>,
    ) -> Vec<Decision<W, H>> {
        if reveals.is_empty() {
            if self.constraint_sets.0.is_empty() {
                vec![guess(minefield)]
            } else {
                let remaining_mines = minefield.mines - self.found_mines;
                let solutions = self
                    .constraint_sets
                    .find_viable_solutions(remaining_mines, &self.known_fields);
                // dbg!(&solutions);
                vec![guess(minefield)]
            }
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
            if let Cell::Label(mut label) = cell {
                let mut neighbors = ArrayVec::new();
                for neighbor in coord.neighbours().iter() {
                    match (
                        minefield.field.get(*neighbor),
                        self.known_fields.get(*neighbor),
                    ) {
                        (Cell::Flag, _) | (_, CellContent::Known(true)) => label -= 1,
                        (Cell::Hidden, _) => neighbors.push(*neighbor),
                        _ => {}
                    }
                }
                if !neighbors.is_empty() {
                    let constraint = Constraint {
                        label,
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
                decisions.extend(set.solve_trivial_cases(&mut self.known_fields));
            }
            set.reduce();
        }
        self.constraint_sets.check_splits();

        let mut prev_decisions = decisions.len();
        while {
            for set in &mut self.constraint_sets.0 {
                let res = set.solve_trivial_cases(&mut self.known_fields);
                if !res.is_empty() {
                    set.reduce();
                }
                decisions.extend(res);
            }
            if decisions.len() != prev_decisions {
                self.constraint_sets.check_splits();
            }
            decisions.len() != prev_decisions
        } {
            prev_decisions = decisions.len()
        }

        decisions.sort();
        decisions.dedup();

        decisions.retain(|decision| match decision {
            Decision::Flag(c) => minefield.field.get(*c) == Cell::Hidden,
            Decision::Reveal(c) => !matches!(minefield.field.get(*c), Cell::Empty | Cell::Label(_)),
        });

        self.found_mines += decisions
            .iter()
            .filter(|d| matches!(d, Decision::Flag(_)))
            .count() as u8;

        decisions
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
