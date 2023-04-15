use arrayvec::ArrayVec;
use bitvec::vec::BitVec;
use miinaharava::minefield::{Coord, Matrix};

use super::{
    constraint_sets::{ConstraintSet, CoupledSets},
    constraints::Constraint,
    CellContent, Decision, KnownMinefield,
};

/// Type alias for BitVec for better readibility.
type PossibleSolution = BitVec;

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
    /// are no solutions for that amount of mines, Some if there might be.
    pub fn get(&self, mine_count: u8) -> Option<&Vec<PossibleSolution>> {
        if self.min_mines > mine_count || mine_count > self.max_mines {
            None
        } else {
            Some(&self.solutions_by_mines[mine_count as usize])
        }
    }

    /// Does the same as get, but a mutable list.
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
            let same_idx_solutions = self.iter().flatten().map(|s| s[i]).collect::<BitVec>();
            (*coord, same_idx_solutions)
        })
    }
}

impl<const W: usize, const H: usize> SolutionContainer<W, H> for SolutionList<W, H> {
    fn find_best_guess(&self) -> (Coord<W, H>, f32) {
        let mut best_guess = None;

        let len = self.iter().flatten().count();

        for (coord, guesses) in self.transposed_solution_coords() {
            let propability = guesses.count_zeros() as f32 / len as f32;
            if let Some((_, previous_guess_p)) = best_guess {
                if propability > previous_guess_p {
                    best_guess = Some((coord, propability));
                }
            } else {
                best_guess = Some((coord, propability));
            }
        }

        // if best_guess.unwrap().1 > 0.8 {
        //     dbg!(&self
        //         .iter()
        //         .filter(|s| !s.is_empty())
        //         .map(|s| s.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        //         .collect::<Vec<_>>());
        //     dbg!(self.coords.iter().position(|c| *c == best_guess.unwrap().0));
        //     dbg!(best_guess);
        // }

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

impl<const W: usize, const H: usize> CoupledSets<W, H> {
    /// TODO: Docs
    pub fn find_viable_solutions(
        &self,
        remaining_mines: u8,
        known_minefield: &KnownMinefield<W, H>,
    ) -> Vec<SolutionList<W, H>> {
        let mut solution_lists = Vec::with_capacity(self.0.len());
        let mut min_mines = 0;

        for set in &self.0 {
            let solutions = set.find_viable_solutions(remaining_mines, known_minefield);
            min_mines += solutions.min_mines;
            solution_lists.push(solutions);
        }

        for list in &mut solution_lists {
            let allowed_max_mines = list.min_mines + (remaining_mines - min_mines);
            for mine_count in (allowed_max_mines + 1)..=(list.solutions_by_mines.len() as u8) {
                if let Some(curr) = list.get_mut(mine_count) {
                    curr.clear()
                }
            }
            list.max_mines = allowed_max_mines.min(list.max_mines);
        }
        solution_lists
    }
}

impl<const W: usize, const H: usize> ConstraintSet<W, H> {
    /// Required for ARRAY_VEC_MATRIX
    const ARRAY_VEC_CONST: ArrayVec<usize, 8> = ArrayVec::new_const();
    /// Required for ARRAY_VEC_MATRIX
    const ARRAY_VEC_CONST_W: [ArrayVec<usize, 8>; W] = [ConstraintSet::<W, H>::ARRAY_VEC_CONST; W];
    /// An empty WxH matrix where each element is an ArrayVec of usize with a
    /// max capacity of 8, used for [find_ordered]
    const ARRAY_VEC_MATRIX: [[ArrayVec<usize, 8>; W]; H] =
        [ConstraintSet::<W, H>::ARRAY_VEC_CONST_W; H];

    /// Form a list from all variables that contain the variable and indexes of
    /// all constraints that include said variable. Sort the list by the amount
    /// of constraints for each variable from most constraints to least.
    ///
    /// Constraint indexes refer to the current order of constraints, if
    /// constraints are modified at any time, the indexes may not work correctly
    /// anymore!
    pub fn find_ordered(&self) -> Vec<(Coord<W, H>, ArrayVec<usize, 8>)> {
        let mut map = Matrix(ConstraintSet::<W, H>::ARRAY_VEC_MATRIX);

        for (i, constraint) in self.constraints.iter().enumerate() {
            for var in &constraint.variables {
                map.get_mut_ref(*var).push(i);
            }
        }

        let mut ordered = Vec::with_capacity(W * H);

        for (y, row) in map.into_iter().enumerate() {
            for (x, vec) in row.into_iter().enumerate() {
                if !vec.is_empty() {
                    ordered.push((Coord::<W, H>(x as u8, y as u8), vec));
                }
            }
        }

        ordered.sort_by_key(|c| -(c.1.len() as i8));

        ordered
    }

    /// TODO: Docs
    pub fn find_viable_solutions(
        &self,
        remaining_mines: u8,
        known_field: &KnownMinefield<W, H>,
    ) -> SolutionList<W, H> {
        let ordered = self.find_ordered();

        let mut solutions = if !ordered.is_empty() {
            self.find_solutions(&ordered, BitVec::new(), *known_field)
        } else {
            Vec::new()
        };
        solutions.sort();
        solutions.dedup();

        SolutionList::from(
            solutions,
            ordered.into_iter().map(|o| o.0).collect(),
            remaining_mines,
        )
    }

    /// TODO: Docs
    #[inline]
    pub fn find_solutions(
        &self,
        list: &[(Coord<W, H>, ArrayVec<usize, 8>)],
        history: PossibleSolution,
        testing_field: KnownMinefield<W, H>,
    ) -> Vec<PossibleSolution> {
        let res2 = self.guess_next(false, list, history.clone(), testing_field);
        let res1 = self.guess_next(true, list, history, testing_field);

        let mut results = Vec::with_capacity(2);
        if let Some(res) = res1 {
            results.extend(res);
        }
        if let Some(res) = res2 {
            results.extend(res);
        }
        results
    }

    /// TODO: Docs
    fn guess_next(
        &self,
        guess: bool,
        list: &[(Coord<W, H>, ArrayVec<usize, 8>)],
        mut history: PossibleSolution,
        mut testing_field: KnownMinefield<W, H>,
    ) -> Option<Vec<PossibleSolution>> {
        assert!(history.len() < list.len());
        let (coord, idx_vec) = &list[history.len()];
        testing_field.set(*coord, CellContent::Known(guess));
        for idx in idx_vec {
            let constraint = &self.constraints[*idx];
            let (hidden, mines) = guessed_count(constraint, &testing_field);

            if constraint.label > (hidden + mines) || mines > constraint.label {
                None?;
            }
        }
        history.push(guess);
        if history.len() >= list.len() {
            let mut returned = Vec::with_capacity(list.len());
            returned.push(history);
            Some(returned)
        } else {
            Some(self.find_solutions(list, history, testing_field))
        }
    }
}

/// TODO: Docs
fn guessed_count<const W: usize, const H: usize>(
    constraint: &Constraint<W, H>,
    guessed: &KnownMinefield<W, H>,
) -> (u8, u8) {
    let mut hidden = constraint.variables.len();
    let mut mines = 0;
    for var in &constraint.variables {
        if let CellContent::Known(val) = guessed.get(*var) {
            hidden -= 1;
            mines += val as i8;
        }
    }
    (hidden as u8, mines as u8)
}
