//! TODO: Docs

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
            if !set.constraints.is_empty() {
                new_vec.extend(set.check_splits());
            }
        }
        self.0 = new_vec;
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

        while let Some(constraint) = constraints.pop() {
            let (mut indexes, found_sets): (Vec<usize>, Vec<&mut ConstraintSet<W, H>>) = sets
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
            let constraint_set = found_sets.into_iter().reduce(|a, b| a.drain_from(b));

            if let Some(set) = constraint_set {
                set.variables
                    .insert_many(constraint.variables.iter().copied());
                set.constraints.push(constraint);

                if !indexes.is_empty() {
                    indexes.remove(0);
                    for index in indexes.iter().rev() {
                        sets.remove(*index);
                    }
                }
            } else {
                let mut variables = CoordSet::default();
                variables.insert_many(constraint.variables.iter().copied());
                sets.push(ConstraintSet {
                    variables,
                    constraints: vec![constraint],
                });
            }
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

        for (exists, var) in self.variables.iter_mut() {
            if let CellContent::Known(_) = known_field.get(var) {
                *exists = false;
            }
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
}
