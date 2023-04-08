//! TODO: Docs
use std::collections::{BinaryHeap, HashMap};

use arrayvec::ArrayVec;
use miinaharava::minefield::{Coord, Matrix};

use super::{constraints::Constraint, coord_set::CoordSet, CellContent, Decision, KnownMinefield};

#[derive(Debug, Clone, Default)]
/// TODO: Docs
pub struct CoupledSets<const W: usize, const H: usize>(pub Vec<ConstraintSet<W, H>>);

impl<const W: usize, const H: usize> CoupledSets<W, H> {
    /// TODO: Docs
    #[must_use]
    pub fn insert(
        &mut self,
        constraint: Constraint<W, H>,
        known_minefield: &mut KnownMinefield<W, H>,
    ) -> Option<Vec<Decision<W, H>>> {
        // Returns mutably all the constraint sets that contain any of the
        // variables in the new constraints, and their indexes
        let (mut indexes, sets): (Vec<usize>, Vec<&mut ConstraintSet<W, H>>) = self
            .0
            .iter_mut()
            .enumerate()
            .filter(|(_, constraint_set)| {
                constraint
                    .variables
                    .iter()
                    .any(|v| constraint_set.variables.contains(*v))
            })
            .unzip();

        // Combine all retrieved constraints into the first constraint
        let constraint_set = sets.into_iter().reduce(|a, b| a.drain_from(b));

        // If a constraint set was found, insert the constraint set in it,
        // otherwise create a new set.
        let decisions = if let Some(set) = constraint_set {
            set.insert(constraint, known_minefield)
        } else {
            self.0.push(ConstraintSet::default());
            let last_idx = self.0.len() - 1;
            let set = self.0.get_mut(last_idx).unwrap();
            set.insert(constraint, known_minefield)
        };

        // Remove all other constraint sets
        if !indexes.is_empty() {
            indexes.remove(0);
            for index in indexes.iter().rev() {
                self.0.remove(*index);
            }
        }

        decisions
    }

    /// TODO: Docs
    pub fn check_splits(&mut self) {
        let mut new_vec = Vec::new();
        while let Some(set) = self.0.pop() {
            new_vec.extend(set.check_splits());
        }
        self.0 = new_vec;
    }

    /// TODO: Docs
    pub fn find_viable_solutions(
        &self,
        remaining_mines: u8,
        known_minefield: &KnownMinefield<W, H>,
    ) -> Vec<SolutionListMap<W, H>> {
        let mut solution_lists = Vec::with_capacity(self.0.len());
        let mut min_mines = 0;

        for set in &self.0 {
            if let Ok(solution) = set.find_viable_solutions(remaining_mines, known_minefield) {
                min_mines += solution.min_mines;
                solution_lists.push(solution);
            }
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

/// Coupled Constraints
#[derive(Debug, Clone, Default)]
pub struct ConstraintSet<const W: usize, const H: usize> {
    /// List of label-mine-location-constraints for a given state
    pub constraints: Vec<Constraint<W, H>>,
    /// List of all the variables that are in this set of coupled constraints
    pub variables: CoordSet<W, H>,
}

impl<const W: usize, const H: usize> PartialEq for ConstraintSet<W, H> {
    fn eq(&self, other: &Self) -> bool {
        let mut a = self.constraints.clone();
        let mut b = other.constraints.clone();
        a.sort();
        b.sort();
        a == b && self.variables == other.variables
    }
}

impl<const W: usize, const H: usize> ConstraintSet<W, H> {
    const ARRAY_VEC_CONST: ArrayVec<usize, 8> = ArrayVec::new_const();
    const ARRAY_VEC_CONST_W: [ArrayVec<usize, 8>; W] = [ConstraintSet::<W, H>::ARRAY_VEC_CONST; W];
    const ARRAY_VEC_MATRIX: [[ArrayVec<usize, 8>; W]; H] =
        [ConstraintSet::<W, H>::ARRAY_VEC_CONST_W; H];

    /// TODO: Docs
    pub fn drain_from(&mut self, other: &mut ConstraintSet<W, H>) -> &mut ConstraintSet<W, H> {
        self.constraints.append(&mut other.constraints);
        self.variables.extend(&other.variables);
        self.constraints.sort();
        self.constraints.dedup();
        self
    }

    /// TODO: Docs
    #[must_use]
    pub fn insert(
        &mut self,
        mut constraint: Constraint<W, H>,
        known_field: &mut KnownMinefield<W, H>,
    ) -> Option<Vec<Decision<W, H>>> {
        if !constraint.is_empty() && !self.constraints.contains(&constraint) {
            if let Some(d) = ConstraintSet::solve_trivial_constraint(&mut constraint, known_field) {
                Some(d)
            } else {
                self.variables
                    .insert_many(constraint.variables.iter().copied());
                self.constraints.push(constraint);
                None
            }
        } else {
            None
        }
    }

    /// TOOD: Docs
    pub fn check_splits(self) -> Vec<ConstraintSet<W, H>> {
        let ConstraintSet {
            mut constraints,
            variables: _,
        } = self;

        let mut sets: Vec<ConstraintSet<W, H>> = Vec::new();

        'outer: while let Some(constraint) = constraints.pop() {
            for set in &mut sets {
                if constraint
                    .variables
                    .iter()
                    .any(|v| set.variables.contains(*v))
                {
                    set.variables
                        .insert_many(constraint.variables.iter().copied());
                    set.constraints.push(constraint);
                    continue 'outer;
                }
            }
            let mut variables = CoordSet::default();
            variables.insert_many(constraint.variables.iter().copied());
            sets.push(ConstraintSet {
                constraints: vec![constraint],
                variables,
            })
        }

        sets
    }

    /// Solves trivial cases, meaning that it will reveal all variables that
    /// have an obvious answer.
    #[must_use]
    pub fn solve_trivial_cases(
        &mut self,
        known_field: &mut KnownMinefield<W, H>,
    ) -> Vec<Decision<W, H>> {
        let mut decisions = Vec::new();
        let mut old_decisions_len = 0;

        while {
            let mut idx = 0;
            while let Some(constraint) = self.constraints.get_mut(idx) {
                if let Some(d) = ConstraintSet::solve_trivial_constraint(constraint, known_field) {
                    decisions.extend(d);
                    self.constraints.remove(idx);
                } else {
                    idx += 1;
                }
            }
            old_decisions_len < decisions.len()
        } {
            old_decisions_len = decisions.len();
        }

        for decision in &decisions {
            match decision {
                Decision::Reveal(c) | Decision::Flag(c) => self.variables.remove(*c),
            }
        }

        decisions
    }

    /// TODO: Docs
    #[must_use]
    pub fn solve_trivial_constraint(
        constraint: &mut Constraint<W, H>,
        known_field: &mut KnownMinefield<W, H>,
    ) -> Option<Vec<Decision<W, H>>> {
        let mut decisions = Vec::new();

        let mut idx = 0;
        while let Some(var) = constraint.variables.get(idx) {
            if let CellContent::Known(val) = known_field.get(*var) {
                constraint.label -= val as u8;
                constraint.variables.remove(idx);
            } else {
                idx += 1;
            }
        }

        if constraint.label == 0 {
            for variable in &constraint.variables {
                known_field.set(*variable, CellContent::Known(false));
                decisions.push(Decision::Reveal(*variable));
            }
            Some(decisions)
        } else if constraint.label as usize == constraint.variables.len() {
            for variable in &constraint.variables {
                known_field.set(*variable, CellContent::Known(true));
                decisions.push(Decision::Flag(*variable));
            }
            Some(decisions)
        } else {
            None
        }
    }

    /// TODO: Docs
    pub fn reduce(&mut self) {
        let mut edited = true;
        while edited {
            edited = false;
            // TODOS:
            // 3. make tests for CoordSet
            self.constraints.sort_by_key(|i| i.len());

            for smallest_idx in 0..self.constraints.len() {
                let (smaller, others) = self.constraints.split_at_mut(smallest_idx + 1);
                let smallest = &mut smaller[smaller.len() - 1];

                if !smallest.is_empty() {
                    for other in &mut *others {
                        if other.len() > smallest.len() && other.is_superset_of(smallest) {
                            #[cfg(test)]
                            dbg!(&other, &smallest);
                            other.subtract(smallest);
                            edited = true;
                        }
                    }
                    if edited {
                        break;
                    }
                }
            }
        }

        self.constraints.sort();
        self.constraints.dedup();
    }

    /// TODO: Docs
    #[allow(clippy::result_unit_err)]
    pub fn find_viable_solutions(
        &self,
        remaining_mines: u8,
        known_field: &KnownMinefield<W, H>,
    ) -> Result<SolutionListMap<W, H>, ()> {
        let mut map = Matrix(ConstraintSet::<W, H>::ARRAY_VEC_MATRIX);

        for (i, constraint) in self.constraints.iter().enumerate() {
            for var in &constraint.variables {
                map.get_mut_ref(*var).push(i);
            }
        }

        let mut ordered = Vec::with_capacity(W * H);

        for (y, row) in map.iter().enumerate() {
            for (x, vec) in row.iter().enumerate() {
                if !vec.is_empty() {
                    ordered.push((Coord::<W, H>(x as u8, y as u8), vec));
                }
            }
        }

        ordered.sort_by_key(|c| -(c.1.len() as i8));

        // dbg!(&ordered);

        let mut results = self.test_both(&ordered, 0, *known_field)?;
        results.sort();
        results.dedup();

        let mut returned = SolutionListMap {
            solutions_by_mines: vec![Vec::new(); (remaining_mines + 1) as usize],
            min_mines: remaining_mines + 1,
            max_mines: 0,
        };
        for result in results {
            let mine_count = result.iter().filter(|c| c.1).count() as u8;
            dbg!(&result);
            dbg!(mine_count);
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

        Ok(returned)
    }

    /// TODO: Docs
    #[inline]
    fn test_both(
        &self,
        list: &[(Coord<W, H>, &ArrayVec<usize, 8>)],
        idx: usize,
        testing_field: KnownMinefield<W, H>,
    ) -> Result<Vec<PossibleSolution<W, H>>, ()> {
        let res2 = self.test(false, list, idx, testing_field);
        let res1 = self.test(true, list, idx, testing_field);
        if let (Err(_), Err(_)) = (&res1, &res2) {
            Err(())?;
        }

        let mut results = Vec::new();
        if let Ok(res) = res1 {
            results.extend(res);
        }
        if let Ok(res) = res2 {
            results.extend(res);
        }
        Ok(results)
    }

    /// TODO: Docs
    fn test(
        &self,
        guess: bool,
        list: &[(Coord<W, H>, &ArrayVec<usize, 8>)],
        idx: usize,
        mut testing_field: KnownMinefield<W, H>,
    ) -> Result<Vec<PossibleSolution<W, H>>, ()> {
        if let Some((coord, idx_vec)) = list.get(idx) {
            // dbg!("guessing", guess, &coord);
            testing_field.set(*coord, CellContent::Known(guess));
            for idx in *idx_vec {
                let constraint = unsafe { self.constraints.get_unchecked(*idx) };
                let (hidden, mines) = guessed_count(constraint, &testing_field);

                // dbg!(constraint.label, hidden, mines);
                if constraint.label > (hidden + mines) || mines > constraint.label {
                    // Oh no
                    // dbg!("failure at", guess, &coord, constraint, hidden, mines);
                    Err(())?;
                }
            }
            let mut results = self.test_both(list, idx + 1, testing_field)?;
            for res in &mut results {
                res.push((*coord, guess));
            }
            // dbg!("success:", &results);
            Ok(results)
        } else {
            let mut outer_vec = Vec::with_capacity(idx.pow(2) + 1);
            let res = Vec::with_capacity(list.len());
            outer_vec.push(res);
            // dbg!("Ending found");
            Ok(outer_vec)
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
/// TODO: Docs
pub struct SolutionListMap<const W: usize, const H: usize> {
    solutions_by_mines: Vec<Vec<PossibleSolution<W, H>>>,
    min_mines: u8,
    max_mines: u8,
}

impl<const W: usize, const H: usize> SolutionListMap<W, H> {
    /// TODO: Docs
    pub fn get(&self, mine_count: u8) -> Option<&Vec<PossibleSolution<W, H>>> {
        if self.min_mines > mine_count || mine_count > self.max_mines {
            None
        } else {
            unsafe { Some(self.solutions_by_mines.get_unchecked(mine_count as usize)) }
        }
    }

    /// TODO: Docs
    pub fn get_mut(&mut self, mine_count: u8) -> Option<&mut Vec<PossibleSolution<W, H>>> {
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

type PossibleSolution<const W: usize, const H: usize> = Vec<(Coord<W, H>, bool)>;

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
