use arrayvec::ArrayVec;
use miinaharava::minefield::{self, Cell, Coord, Minefield};
use std::fmt::Debug;

use crate::ai::Decision;

#[derive(Clone)]
pub struct Constraint<const W: usize, const H: usize> {
    label: u8,
    variables: Vec<Coord<W, H>>,
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

#[derive(Debug)]
pub enum CSPError {
    NoTrivialCases,
}

/// General state used for solving Constraint Satisfication Problem
#[derive(Debug, Clone)]
pub struct ConstaintSatisficationState<const W: usize, const H: usize> {
    constraints: Vec<Constraint<W, H>>,
}

impl<const W: usize, const H: usize> ConstaintSatisficationState<W, H> {
    /// Constructs a CPS-state from a given minefield. Goes through all labels
    /// in the visual field and creates a constraint from them.
    pub fn from(minefield: &Minefield<W, H>) -> Self {
        let mut constraints = Vec::new();

        for (y, row) in minefield.field.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if let Cell::Label(mut num) = cell {
                    let mut neighbors = Vec::new();
                    for neighbor in Coord::<W, H>(x, y).neighbours() {
                        match minefield.field[neighbor.1][neighbor.0] {
                            Cell::Flag => num -= 1,
                            Cell::Hidden => neighbors.push(neighbor),
                            _ => {}
                        };
                    }
                    constraints.push(Constraint {
                        label: num,
                        variables: neighbors,
                    });
                }
            }
        }
        ConstaintSatisficationState { constraints }
    }

    /// Solves trivial cases, meaning that it will reveal all variables that
    /// have an obvious answer.
    pub fn solve_trivial_cases(&self) -> Result<Vec<Decision<W, H>>, CSPError> {
        let mut decisions = Vec::new();
        for constraint in &self.constraints {
            if constraint.label as usize == constraint.variables.len() {
                for variable in &constraint.variables {
                    decisions.push(Decision::Flag(*variable));
                }
            }
            if constraint.label == 0 {
                for variable in &constraint.variables {
                    decisions.push(Decision::Reveal(*variable));
                }
            }
        }
        decisions.sort();
        decisions.dedup();

        Ok(decisions)
    }
}
