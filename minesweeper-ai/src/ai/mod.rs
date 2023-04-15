//! TODO: Docs

use arrayvec::ArrayVec;
use fixed::{types::extra::U20, FixedU32};
use miinaharava::minefield::{Cell, Coord, Matrix, Minefield, Reveal};
use rand::seq::SliceRandom;

use crate::ai::backtracking::SolutionContainer;

use self::{
    backtracking::SolutionList, constraint_sets::CoupledSets, constraints::Constraint,
    coord_set::CoordSet,
};

pub mod backtracking;
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
/// Represents a single decision
pub enum Decision<const W: usize, const H: usize> {
    /// Flag this coordinate as a mine
    Flag(Coord<W, H>),
    /// Reveal this coordinate
    Reveal(Coord<W, H>),
    /// Reveal this coordinate, but this reveal was actually guessed with
    /// propability the fixed point decimal
    GuessReveal(Coord<W, H>, FixedU32<U20>),
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
}

impl<const W: usize, const H: usize> CSPState<W, H> {
    /// TODO: Docs
    pub fn ponder(
        &mut self,
        reveals: Vec<Reveal<W, H>>,
        minefield: &Minefield<W, H>,
    ) -> Vec<Decision<W, H>> {
        // Set all newly revealed tiles as known
        let mut decisions = Vec::new();
        for (coord, cell) in &reveals {
            self.known_fields
                .set(*coord, CellContent::Known(*cell == Cell::Mine))
        }

        // Add new constraints to constraint sets
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

        // Solve initial trivial cases after insert
        for set in &mut self.constraint_sets.0 {
            if !decisions.is_empty() {
                decisions.extend(set.solve_trivial_cases(&mut self.known_fields));
            }
            set.reduce();
        }
        self.constraint_sets.check_splits();

        // Do the following until nothing new happens:
        // 1. solve trivial cases
        // 2. reduce constraint sets
        // 3. check if sets can be further split
        let mut prev_decisions = decisions.len();
        while {
            for set in &mut self.constraint_sets.0 {
                let res = set.solve_trivial_cases(&mut self.known_fields);
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

        // Make sure decisions are only unique and relevant
        decisions.sort();
        decisions.dedup();
        decisions.retain(|decision| match decision {
            Decision::Flag(c) => minefield.field.get(*c) == Cell::Hidden,
            Decision::Reveal(c) | Decision::GuessReveal(c, _) => {
                !matches!(minefield.field.get(*c), Cell::Empty | Cell::Label(_))
            }
        });

        // If no decisions could be made, make an educated guess.
        if decisions.is_empty() {
            self.perform_educated_guess(minefield)
        } else {
            decisions
        }
    }

    /// TODO: Docs
    pub fn perform_educated_guess(&mut self, minefield: &Minefield<W, H>) -> Vec<Decision<W, H>> {
        let found_mines = self
            .known_fields
            .iter()
            .flatten()
            .filter(|c| **c == CellContent::Known(true))
            .count() as u8;
        let remaining_mines = minefield.mines - found_mines;

        // Find all viable solutions
        let solution_lists = self
            .constraint_sets
            .find_viable_solutions(remaining_mines, &self.known_fields);

        // if no solutions, just make an educated guess
        if !solution_lists.is_empty() {
            // Find trivial solutions
            let mut trivials = Vec::new();
            for list in &solution_lists {
                let res = list.find_trivial_decisions(&mut self.known_fields);
                if !res.is_empty() {
                    trivials.extend(res);
                }
            }
            if !trivials.is_empty() {
                for set in &mut self.constraint_sets.0 {
                    trivials.extend(set.solve_trivial_cases(&mut self.known_fields));
                }
                trivials
            } else {
                self.perform_best_guess(solution_lists, remaining_mines)
            }
        } else {
            let vars = self
                .constraint_sets
                .unconstrained_variables(&self.known_fields);
            let len = vars.iter().count();
            let propability = 1. - (remaining_mines as f32 / len as f32);
            vec![Decision::GuessReveal(
                guess(vars),
                FixedU32::from_num(propability),
            )]
        }
    }

    /// TODO: Docs
    pub fn perform_best_guess(
        &mut self,
        solution_lists: Vec<SolutionList<W, H>>,
        remaining_mines: u8,
    ) -> Vec<Decision<W, H>> {
        // Find the best guess from solution lists
        let mut best_guess = solution_lists.find_best_guess();

        // if best_guess.1 > 0.8 {
        //     dbg!(&self
        //         .constraint_sets
        //         .0
        //         .iter()
        //         .map(|c| &c.constraints)
        //         .collect::<Vec<_>>());
        //     dbg!(best_guess);
        // }

        // Find out if unconstrained vars have a better propability
        let unconstrained_mines = (remaining_mines - solution_lists.min_mines()) as u32;
        let unconstrained_vars = self
            .constraint_sets
            .unconstrained_variables(&self.known_fields);
        if !unconstrained_vars.is_empty() {
            let len = unconstrained_vars.iter().count() as u32;
            assert!(len > 0);
            let non_mines = len - unconstrained_mines.min(len);
            let propability = non_mines as f32 / len as f32;
            if propability > best_guess.1 {
                best_guess = (guess(unconstrained_vars), propability);
            }
        }

        // Pick which one was better
        vec![Decision::GuessReveal(
            best_guess.0,
            FixedU32::from_num(best_guess.1),
        )]
    }
}

/// Make a purely random guess. At least for now, this function is meant for use
/// simply so that the game will never stagnate entirely.
///
/// Still not very good, but at least it's trying
pub fn guess<const W: usize, const H: usize>(mut available_vars: CoordSet<W, H>) -> Coord<W, H> {
    let mut rng = rand::thread_rng();

    let corners = CoordSet::corners().intersection(&available_vars);
    let edges = CoordSet::edges().intersection(&available_vars);
    available_vars.omit(&corners);
    available_vars.omit(&edges);

    let coord = if !corners.is_empty() {
        *corners.iter().collect::<Vec<_>>().choose(&mut rng).unwrap()
    } else if !edges.is_empty() {
        *edges.iter().collect::<Vec<_>>().choose(&mut rng).unwrap()
    } else {
        *available_vars
            .iter()
            .collect::<Vec<_>>()
            .choose(&mut rng)
            .unwrap()
    };

    coord
}
