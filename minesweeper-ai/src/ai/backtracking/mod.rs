//! This module contains all of the functional code that is used for the
//! backtracking algorithm described in the papers.

use arrayvec::ArrayVec;
use bitvec::vec::BitVec;
use miinaharava::minefield::{Coord, Matrix};

use self::solutions::{PossibleSolution, SolutionList};

use super::{
    constraint_sets::{ConstraintSet, CoupledSets},
    constraints::Constraint,
    CellContent, KnownMinefield,
};

pub mod solutions;

impl<const W: usize, const H: usize> CoupledSets<W, H> {
    /// Find all viable solutions for all constraint sets, so all coupled sets
    /// of constraints
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

    /// Find all the viable solutions only for this specific set of constraints
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

    /// Try and find solutions for a specific coordinate-to-constraints list,
    /// wtih a the specified history. Used in recursion, history can just be
    /// defined as an empty vec at the start, and list should be what
    /// [ConstraintSet::find_ordered] returns. testing_field parameter is simply
    /// the current status of the known field that is then copied and tested
    /// against.
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

    /// Make a specific guess for the next variable.
    ///
    /// See [ConstraintSet::find_solutions]
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
            let (hidden, mines) = constraint_counts(constraint, &testing_field);

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

/// Return the amount of hidden variables and known-to-be-mine variables for the
/// specified constraint. Used in [ConstraintSet::guess_next]
fn constraint_counts<const W: usize, const H: usize>(
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
