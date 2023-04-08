//! This module contains everything related to specifically solving the
//! Constraint Satisfaction Problem.

use arrayvec::ArrayVec;
use miinaharava::minefield::Coord;
use std::fmt::Debug;

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
