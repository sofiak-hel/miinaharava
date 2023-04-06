//! This module contains everything related to specifically solving the
//! Constraint Satisfaction Problem.

use arrayvec::ArrayVec;
use miinaharava::minefield::{Cell, Coord, Matrix, Minefield, Reveal};
use std::{collections::HashSet, fmt::Debug, ops::Deref};

use crate::ai::{guess, Decision};

/// Represents a single constraint where the variables represent tiles that are
/// still unknown to some degree, and the label represents the value that the
/// variables need to add up to.
///
/// In concrete terms, variables are hidden unflagged cells and the label is how many
/// mines are still undiscovered in said cells.
#[derive(Clone, PartialOrd, Ord, Eq, Default)]
pub struct Constraint<const W: usize, const H: usize> {
    /// Value or label for the variables
    pub label: u8,
    /// List of coordinates to represent the variables that add up to the label.
    pub variables: ArrayVec<Coord<W, H>, 8>,
}

impl<const W: usize, const H: usize> Constraint<W, H> {
    /// TODO: Docs
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// TODO: Docs
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// TODO: Docs
    pub fn is_superset_of(&self, other: &Constraint<W, H>) -> bool {
        if self.len() > other.len() {
            other.variables.iter().all(|v| self.variables.contains(v))
        } else {
            false
        }
    }

    /// TODO: Docs
    pub fn subtract(&mut self, other: &Constraint<W, H>) {
        for other_var in &other.variables {
            if let Some(idx) = self.variables.iter().position(|v| v == other_var) {
                self.variables.remove(idx);
            }
        }
        self.label -= other.label;
    }
}

impl<const W: usize, const H: usize> Debug for Constraint<W, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = ", self.label)?;
        for (i, coord) in self.variables.iter().enumerate() {
            write!(f, "{:?}", coord)?;
            if i < self.variables.len() - 1 {
                write!(f, " + ")?;
            }
        }
        write!(f, "(len: {})", self.variables.len())?;
        Ok(())
    }
}

impl<const W: usize, const H: usize> PartialEq for Constraint<W, H> {
    fn eq(&self, other: &Self) -> bool {
        let mut a = self.variables.clone();
        let mut b = other.variables.clone();
        a.sort();
        b.sort();
        a == b && self.label == other.label
    }
}

/// Custom error type for any failure states that might occur.
#[derive(Debug)]
pub enum CSPError {}

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

type KnownMinefield<const W: usize, const H: usize> = Matrix<CellContent, W, H>;

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CellContent {
    Known(bool),
    #[default]
    Unknown,
}

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
            let set = self.0.get_mut(0).unwrap();
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

/// Todo: Transfer me elsewhere
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CoordSet<const W: usize, const H: usize> {
    pub matrix: Matrix<bool, W, H>,
}

impl<const W: usize, const H: usize> CoordSet<W, H> {
    /// TODO: Docs
    pub fn insert(&mut self, coord: Coord<W, H>) {
        self.matrix.set(coord, true);
    }

    /// TODO: Docs
    pub fn contains(&self, coord: Coord<W, H>) -> bool {
        self.matrix.get(coord)
    }

    /// TODO: Docs
    pub fn iter(&mut self) -> impl Iterator<Item = Coord<W, H>> + '_ {
        self.matrix
            .0
            .iter()
            .flatten()
            .enumerate()
            .filter(|(_, c)| **c)
            .map(|(i, _)| Coord((i % W) as u8, (i / W) as u8))
    }

    /// TODO: Docs
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&mut bool, Coord<W, H>)> + '_ {
        self.matrix
            .0
            .iter_mut()
            .flatten()
            .enumerate()
            .filter(|(_, c)| **c)
            .map(|(i, c)| (c, Coord((i % W) as u8, (i / W) as u8)))
    }

    /// TODO: Docs
    pub fn insert_many<T: Iterator<Item = Coord<W, H>>>(&mut self, coords: T) {
        for coord in coords {
            self.insert(coord);
        }
    }

    /// TODO: Docs
    pub fn extend(&mut self, other: &CoordSet<W, H>) {
        for (a, b) in self
            .matrix
            .0
            .iter_mut()
            .flatten()
            .zip(other.matrix.0.iter().flatten())
        {
            *a |= *b;
        }
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

    /// TOOD: Docs
    pub fn check_splits(self) -> Vec<ConstraintSet<W, H>> {
        let ConstraintSet {
            mut constraints,
            variables,
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

    /// TODO: Docs
    #[must_use]
    pub fn insert(
        &mut self,
        constraint: Constraint<W, H>,
        known_field: &mut KnownMinefield<W, H>,
    ) -> Option<Vec<Decision<W, H>>> {
        if !constraint.is_empty() && !self.constraints.contains(&constraint) {
            if let Some(decisions) = self.solve_trivial_constraint(&constraint, known_field) {
                Some(decisions)
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

    /// TODO: Docs
    #[must_use]
    pub fn clear_known_variables(
        &mut self,
        known_field: &KnownMinefield<W, H>,
    ) -> Vec<Decision<W, H>> {
        let mut decisions = Vec::new();
        for (mut exists, coord) in self.variables.iter_mut() {
            if let CellContent::Known(val) = known_field.get(coord) {
                let mut idx = 0;
                while let Some(constraint) = self.constraints.get_mut(idx) {
                    while let Some(idx) = constraint.variables.iter().position(|v| *v == coord) {
                        constraint.variables.remove(idx);
                        constraint.label -= val as u8;
                    }
                    if constraint.is_empty() {
                        self.constraints.remove(idx);
                    } else {
                        idx += 1;
                    }
                }

                *exists = false;
                if val {
                    decisions.push(Decision::Flag(coord));
                } else {
                    decisions.push(Decision::Reveal(coord));
                }
            }
        }
        decisions.sort();
        decisions.dedup();

        decisions
    }

    /// Solves trivial cases, meaning that it will reveal all variables that
    /// have an obvious answer.
    #[must_use]
    pub fn solve_trivial_cases(
        &mut self,
        known_field: &mut KnownMinefield<W, H>,
    ) -> Vec<Decision<W, H>> {
        let mut decisions = Vec::new();
        let mut idx = 0;
        while let Some(constraint) = self.constraints.get(idx) {
            if let Some(d) = self.solve_trivial_constraint(constraint, known_field) {
                decisions.extend(d);
                self.constraints.remove(idx);
            } else {
                idx += 1;
            }
        }
        decisions
    }

    /// TODO: Docs
    #[must_use]
    pub fn solve_trivial_constraint(
        &self,
        constraint: &Constraint<W, H>,
        known_field: &mut KnownMinefield<W, H>,
    ) -> Option<Vec<Decision<W, H>>> {
        let mut decisions = Vec::new();
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
