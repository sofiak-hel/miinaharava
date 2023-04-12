use arrayvec::ArrayVec;
use bitvec::vec::BitVec;
use miinaharava::minefield::{Coord, Matrix};

use super::{
    constraint_sets::{ConstraintSet, CoupledSets},
    constraints::Constraint,
    CellContent, KnownMinefield,
};

type PossibleSolution = BitVec;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
/// TODO: Docs
pub struct SolutionListMap {
    solutions_by_mines: Vec<Vec<PossibleSolution>>,
    min_mines: u8,
    max_mines: u8,
}

impl SolutionListMap {
    /// TODO: Docs
    pub fn get(&self, mine_count: u8) -> Option<&Vec<PossibleSolution>> {
        if self.min_mines > mine_count || mine_count > self.max_mines {
            None
        } else {
            unsafe { Some(self.solutions_by_mines.get_unchecked(mine_count as usize)) }
        }
    }

    /// TODO: Docs
    pub fn get_mut(&mut self, mine_count: u8) -> Option<&mut Vec<PossibleSolution>> {
        if self.min_mines > mine_count || mine_count > self.max_mines {
            None
        } else {
            unsafe {
                Some(
                    self.solutions_by_mines
                        .get_unchecked_mut(mine_count as usize),
                )
            }
        }
    }
}

impl<const W: usize, const H: usize> CoupledSets<W, H> {
    /// TODO: Docs
    pub fn find_viable_solutions(
        &self,
        remaining_mines: u8,
        known_minefield: &KnownMinefield<W, H>,
    ) -> Vec<SolutionListMap> {
        let mut solution_lists = Vec::with_capacity(self.0.len());
        let mut min_mines = 0;

        for set in &self.0 {
            let solutions = set.find_viable_solutions(remaining_mines, known_minefield);
            min_mines += solutions.min_mines;
            solution_lists.push(solutions);
        }

        let allowed_max_mines = min_mines + (remaining_mines - min_mines);
        for list in &mut solution_lists {
            for mine_count in (allowed_max_mines + 1)..=remaining_mines {
                if let Some(curr) = list.get_mut(mine_count) {
                    if !curr.is_empty() {
                        dbg!(&curr);
                    }
                    curr.clear()
                }
            }
            list.max_mines = allowed_max_mines.min(list.max_mines);
        }
        solution_lists
    }
}

impl<const W: usize, const H: usize> ConstraintSet<W, H> {
    const ARRAY_VEC_CONST: ArrayVec<usize, 8> = ArrayVec::new_const();
    const ARRAY_VEC_CONST_W: [ArrayVec<usize, 8>; W] = [ConstraintSet::<W, H>::ARRAY_VEC_CONST; W];
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
    ) -> SolutionListMap {
        let ordered = self.find_ordered();

        let mut results = self.test_both(&ordered, BitVec::new(), *known_field);
        results.sort();
        results.dedup();

        let mut returned = SolutionListMap {
            solutions_by_mines: vec![Vec::new(); (remaining_mines + 1) as usize],
            min_mines: remaining_mines + 1,
            max_mines: 0,
        };
        for result in results {
            let mine_count = result.iter().filter(|c| **c).count() as u8;
            if mine_count <= remaining_mines {
                unsafe {
                    returned
                        .solutions_by_mines
                        .get_unchecked_mut(mine_count as usize)
                        .push(result);
                };
            }
            returned.min_mines = returned.min_mines.min(mine_count);
            returned.max_mines = returned.max_mines.max(mine_count);
        }

        returned
    }

    /// TODO: Docs
    #[inline]
    pub fn test_both(
        &self,
        list: &[(Coord<W, H>, ArrayVec<usize, 8>)],
        history: PossibleSolution,
        testing_field: KnownMinefield<W, H>,
    ) -> Vec<PossibleSolution> {
        let res2 = self.test(false, list, history.clone(), testing_field);
        let res1 = self.test(true, list, history, testing_field);

        // TODOS:
        // - Tests for test_both
        // - Tests for find_viable_solutions
        // - simplify this method?
        // - Rest of the algorithm that uses these viable solutions, from step
        //   5., check for crapshoots first though

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
    fn test(
        &self,
        guess: bool,
        list: &[(Coord<W, H>, ArrayVec<usize, 8>)],
        mut history: PossibleSolution,
        mut testing_field: KnownMinefield<W, H>,
    ) -> Option<Vec<PossibleSolution>> {
        if let Some((coord, idx_vec)) = list.get(history.len()) {
            testing_field.set(*coord, CellContent::Known(guess));
            for idx in idx_vec {
                let constraint = unsafe { self.constraints.get_unchecked(*idx) };
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
                Some(self.test_both(list, history, testing_field))
            }
        } else {
            let mut returned = Vec::with_capacity(list.len());
            returned.push(history);
            Some(returned)
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
