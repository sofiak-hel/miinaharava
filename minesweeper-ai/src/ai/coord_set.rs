//! TODO: Docs
//! Also, TODO: Tests for this module
use miinaharava::minefield::{Coord, Matrix};

/// Represents a set of coordinates, exhibits similar behaviour to HashSet, but
/// for the purposes of this algorith, much much faster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CoordSet<const W: usize, const H: usize> {
    /// Inner matrix
    pub matrix: Matrix<bool, W, H>,
}

impl<const W: usize, const H: usize> CoordSet<W, H> {
    /// Return a CoordSet where the value defines whether every cell is in it or
    /// not.
    pub fn from(val: bool) -> CoordSet<W, H> {
        CoordSet {
            matrix: Matrix::from(val),
        }
    }

    /// Inser the specified coordinate into the set.
    pub fn insert(&mut self, coord: Coord<W, H>) {
        self.matrix.set(coord, true);
    }

    /// Remove a specified coordinate from the set.
    pub fn remove(&mut self, coord: Coord<W, H>) {
        self.matrix.set(coord, false);
    }

    /// Check whether this coordinate exists in the set or not.
    pub fn contains(&self, coord: Coord<W, H>) -> bool {
        self.matrix.get(coord)
    }

    /// Return an iterator of all the existing coordinates.
    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = Coord<W, H>> + '_ {
        self.matrix
            .0
            .iter()
            .flatten()
            .enumerate()
            .filter(|(_, c)| **c)
            .map(|(i, _)| Coord((i % W) as u8, (i / W) as u8))
    }

    /// Returns an iterator, that returns a mutable boolean which you can use to
    /// remove the specified coordinate, while also returning the specified
    /// coordinate.
    #[allow(dead_code)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&mut bool, Coord<W, H>)> + '_ {
        self.matrix
            .0
            .iter_mut()
            .flatten()
            .enumerate()
            .filter(|(_, c)| **c)
            .map(|(i, c)| (c, Coord((i % W) as u8, (i / W) as u8)))
    }

    /// Insert all of the coordinates from the given iterator of coords.
    /// ```
    /// # use miinaharava::minefield::*;
    /// # use minesweeper_ai::ai::coord_set::*;
    ///
    /// let mut set = CoordSet::<10, 10>::default();
    ///
    /// let coords = vec![Coord(5, 5), Coord(6, 6), Coord(7, 7)];
    ///
    /// set.insert_many(coords.clone().into_iter());
    /// assert_eq!(set.iter().collect::<Vec<_>>(), coords);
    /// ```
    pub fn insert_many<T: Iterator<Item = Coord<W, H>>>(&mut self, coords: T) {
        for coord in coords {
            self.insert(coord);
        }
    }

    /// Extends a CoordSet with another, meaning, meaning that the receiving
    /// CoordSet will have after this operation all of the coordinates of both
    /// self and the other.
    /// ```
    /// # use miinaharava::minefield::*;
    /// # use minesweeper_ai::ai::coord_set::*;
    ///
    /// let mut first = CoordSet::<10, 10>::default();
    /// let mut second = CoordSet::<10, 10>::default();
    ///
    /// first.insert(Coord(5, 5));
    /// second.insert(Coord(6, 6));
    /// first.extend(&second);
    ///
    /// assert!(first.contains(Coord(5, 5)));
    /// assert!(first.contains(Coord(6, 6)));
    /// assert_eq!(first.iter().count(), 2);
    /// ```
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

    /// Removes from this CoordSet every element that belongs in the other
    /// CoordSet
    /// ```
    /// # use miinaharava::minefield::*;
    /// # use minesweeper_ai::ai::coord_set::*;
    ///
    /// let mut first = CoordSet::<10, 10>::default();
    /// let mut second = CoordSet::<10, 10>::default();
    ///
    /// first.insert_many(vec![Coord(5, 5), Coord(6, 6), Coord(7, 7)].into_iter());
    /// second.insert_many(vec![Coord(6, 6), Coord(7, 7)].into_iter());
    /// first.omit(&second);
    ///
    /// assert!(first.contains(Coord(5, 5)));
    /// assert_eq!(first.iter().count(), 1);
    /// ```
    pub fn omit(&mut self, other: &CoordSet<W, H>) {
        for (a, b) in self
            .matrix
            .0
            .iter_mut()
            .flatten()
            .zip(other.matrix.0.iter().flatten())
        {
            *a &= !*b;
        }
    }

    /// Returns whether two coordsets have any values in common.
    pub fn have_coords_in_common(&self, other: &CoordSet<W, H>) -> bool {
        for (a, b) in self
            .matrix
            .0
            .iter()
            .flatten()
            .zip(other.matrix.0.iter().flatten())
        {
            if *a && *b {
                return true;
            }
        }
        false
    }

    /// Returns an intersection of two coordsets, meaning the returned CoordSet
    /// will have only the coordinates that exist in both sets.
    pub fn intersection(&self, other: &CoordSet<W, H>) -> CoordSet<W, H> {
        let mut new_coordset = CoordSet::default();

        for ((a, b), c) in new_coordset
            .matrix
            .0
            .iter_mut()
            .flatten()
            .zip(self.matrix.0.iter().flatten())
            .zip(other.matrix.0.iter().flatten())
        {
            *a = *b && *c;
        }

        new_coordset
    }
}
