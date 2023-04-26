//! This module is seperated by the functionality of handling solutions once
//! they are found via the backtracking algorithm. Represents mostly
//! [SolutionList].

use bitvec::vec::BitVec;
use miinaharava::minefield::Coord;

use crate::ai::{CellContent, Decision, KnownMinefield};

/// Type alias for BitVec for better readibility.
pub type PossibleSolution = BitVec;

/// Trait that represents something that contains a set of solutions
pub trait SolutionContainer<const W: usize, const H: usize> {
    /// Find the best guess decision for this specific struct. Might not be the
    /// best overall guess.
    ///
    /// The best guess is when you divide the amount of falses in every solution
    /// for a given Coord and divide that by the amount of solutions. The Coord
    /// that has the greatest propability is the most likely then to be empty of
    /// a mine, and therefore the best guess.
    fn find_best_guess(&self) -> (Coord<W, H>, f32);
    /// Returns the minimum number of mines for these solutions
    fn min_mines(&self) -> u8;
    /// Returns the maximum number of mines for these solutions
    fn max_mines(&self) -> u8;
}

#[derive(Debug, Clone)]
/// Represents a list of solutions of a single set of coupled constraints a
/// ([ConstraintSet])
pub struct SolutionList<const W: usize, const H: usize> {
    /// All of the solutions found (2D array), partitioned by the number of
    /// mines, where the first index is the amount of mines, which then contains
    /// an array of the solutions that contain this amount of mines.
    pub solutions_by_mines: Vec<Vec<PossibleSolution>>,
    /// The smallest amount of mines in any solution
    pub min_mines: u8,
    /// The largest amount of mines
    pub max_mines: u8,
    /// The coordinates that the solutions indexes reflect.
    pub coords: Vec<Coord<W, H>>,
}

impl<const W: usize, const H: usize> SolutionList<W, H> {
    /// Create a SolutionList from a list of solutions, the coords that these
    /// solutions represent and the amount of remaining mines, that is used to
    /// filter out any impossible solutions
    pub fn from(
        solutions: Vec<PossibleSolution>,
        coords: Vec<Coord<W, H>>,
        remaining_mines: u8,
    ) -> SolutionList<W, H> {
        let mut solution_list = SolutionList {
            solutions_by_mines: vec![Vec::new(); (remaining_mines + 1) as usize],
            min_mines: remaining_mines + 1,
            max_mines: 0,
            coords,
        };
        for solution in solutions {
            let mine_count = solution.count_ones() as u8;
            if mine_count <= remaining_mines {
                solution_list.solutions_by_mines[mine_count as usize].push(solution);
                solution_list.min_mines = solution_list.min_mines.min(mine_count);
                solution_list.max_mines = solution_list.max_mines.max(mine_count);
            }
        }

        solution_list
    }

    /// Safely get a list of solutions for any amount of mines. None if there
    /// are no solutions for that amount of mines, Some if there might be. Used
    /// only in tests.
    #[cfg(test)]
    pub fn get(&self, mine_count: u8) -> Option<&Vec<PossibleSolution>> {
        if self.min_mines > mine_count || mine_count > self.max_mines {
            None
        } else {
            Some(&self.solutions_by_mines[mine_count as usize])
        }
    }

    /// Safely get a list of solutions mutably for any amount of mines. None if
    /// there are no solutions for that amount of mines, Some if there might be.
    /// Used only in tests.
    pub fn get_mut(&mut self, mine_count: u8) -> Option<&mut Vec<PossibleSolution>> {
        if self.min_mines > mine_count || mine_count > self.max_mines {
            None
        } else {
            Some(&mut self.solutions_by_mines[mine_count as usize])
        }
    }

    /// Iterate through all the possible amount of mines, where next() returns a
    /// vec of solutions which all have the same amount of mines.
    pub fn iter(&self) -> impl Iterator<Item = &Vec<PossibleSolution>> {
        (self.min_mines..=self.max_mines).map(|i| &self.solutions_by_mines[i as usize])
    }

    /// Find all coordinates in this set of solutions that are expected to be a
    /// mine or empty of a mine in every solution, meaning it is trivially
    /// solvable.
    pub fn find_trivial_decisions(&self, known: &mut KnownMinefield<W, H>) -> Vec<Decision<W, H>> {
        let mut decisions = Vec::new();

        for (coord, guesses) in self.transposed_solution_coords() {
            if let Some(value) = if guesses.all() {
                Some(true)
            } else if guesses.not_any() {
                Some(false)
            } else {
                None
            } {
                known.set(coord, CellContent::Known(value));
                decisions.push(if value {
                    Decision::Flag(coord)
                } else {
                    Decision::Reveal(coord)
                });
            }
        }

        decisions
    }

    /// Return an iterator that returns all the solutions but transposed.
    ///
    /// Imagine that we have N amount of M-long solutions, where M is also the
    /// amount of coords.
    ///
    /// This function then returns an iterator that goes through M amount of
    /// iterations and returns a Coord and N-long [BitVec], that represent the
    /// guess in every solution for this specific coord.
    pub fn transposed_solution_coords(
        &'_ self,
    ) -> impl Iterator<Item = (Coord<W, H>, PossibleSolution)> + '_ {
        self.coords.iter().enumerate().map(|(i, coord)| {
            let same_idx_solutions = self
                .iter()
                .flatten()
                .map(|s| s[i])
                .collect::<PossibleSolution>();
            (*coord, same_idx_solutions)
        })
    }
}

impl<const W: usize, const H: usize> SolutionContainer<W, H> for SolutionList<W, H> {
    fn find_best_guess(&self) -> (Coord<W, H>, f32) {
        let mut best_guess = None;

        let lens = self.iter().map(|c| c.len()).collect::<Vec<_>>();

        for (coord, guesses) in self.transposed_solution_coords() {
            let mut total_idx = 0;
            let mut total_propability = 0.;
            for len in &lens {
                if *len != 0 {
                    let curr_guesses = &guesses[total_idx..(total_idx + len)];
                    let curr_propability =
                        curr_guesses.count_zeros() as f32 / curr_guesses.len() as f32;
                    total_propability += curr_propability;
                    total_idx += len;
                }
            }
            let propability = total_propability / lens.len() as f32;
            assert!(propability <= 1.);
            if let Some((_, previous_guess_p)) = best_guess {
                if propability > previous_guess_p {
                    best_guess = Some((coord, propability));
                }
            } else {
                best_guess = Some((coord, propability));
            }
        }

        best_guess.unwrap()
    }

    fn max_mines(&self) -> u8 {
        self.max_mines
    }

    fn min_mines(&self) -> u8 {
        self.min_mines
    }
}

impl<const W: usize, const H: usize, T: SolutionContainer<W, H>> SolutionContainer<W, H>
    for Vec<T>
{
    fn find_best_guess(&self) -> (Coord<W, H>, f32) {
        let mut best_guess = None;

        for solution_list in self {
            let (coord, propability) = solution_list.find_best_guess();
            if let Some((_, old_propability)) = best_guess {
                if propability > old_propability {
                    best_guess = Some((coord, propability));
                }
            } else {
                best_guess = Some((coord, propability))
            }
        }
        best_guess.unwrap()
    }

    fn min_mines(&self) -> u8 {
        let mut min_mines = 0;
        for solution in self {
            min_mines += solution.min_mines()
        }
        min_mines
    }

    fn max_mines(&self) -> u8 {
        let mut min_mines = 0;
        for solution in self {
            min_mines += solution.max_mines()
        }
        min_mines
    }
}
