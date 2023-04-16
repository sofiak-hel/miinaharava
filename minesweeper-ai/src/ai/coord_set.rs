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
    /// ```
    /// # use miinaharava::minefield::*;
    /// # use minesweeper_ai::ai::coord_set::*;
    /// use miinaharava::minefield::Matrix;
    ///
    /// let mut set = CoordSet::<2, 2>::from(true);
    ///
    /// let coords = vec![Coord(0, 0), Coord(1, 0), Coord(0, 1), Coord(1, 1)];
    ///
    /// assert_eq!(set.iter().collect::<Vec<_>>(), coords);
    /// ```
    pub fn from(val: bool) -> CoordSet<W, H> {
        CoordSet {
            matrix: Matrix::from(val),
        }
    }

    /// TODO: Docs
    const fn row(middle: bool) -> [bool; W] {
        let mut row = [middle; W];
        row[0] = !middle;
        row[W - 1] = !middle;
        row
    }

    /// TODO: Docs
    pub const fn corners() -> CoordSet<W, H> {
        let mut c = CoordSet {
            matrix: Matrix([[false; W]; H]),
        };
        c.matrix.0[0] = CoordSet::<W, H>::row(false);
        c.matrix.0[H - 1] = CoordSet::<W, H>::row(false);
        c
    }

    /// TODO: Docs
    pub const fn edges() -> CoordSet<W, H> {
        let default = CoordSet::<W, H>::row(false);
        let mut c = CoordSet {
            matrix: Matrix([default; H]),
        };
        let top_bottom = CoordSet::<W, H>::row(true);

        c.matrix.0[0] = top_bottom;
        c.matrix.0[H - 1] = top_bottom;
        c
    }

    /// Inser the specified coordinate into the set.
    pub fn insert(&mut self, coord: Coord<W, H>) {
        self.matrix.set(coord, true);
    }

    /// Remove a specified coordinate from the set.
    /// ```
    /// # use miinaharava::minefield::*;
    /// # use minesweeper_ai::ai::coord_set::*;
    /// use miinaharava::minefield::Matrix;
    ///
    /// let mut set = CoordSet {
    ///     matrix: Matrix([
    ///         [true, true, false],
    ///         [false, true, false],
    ///         [false, false, true]
    ///     ])
    /// };
    ///
    /// set.remove(Coord(1, 1));
    ///
    /// assert_eq!(set.contains(Coord(1, 1)), false);
    /// ```
    pub fn remove(&mut self, coord: Coord<W, H>) {
        self.matrix.set(coord, false);
    }

    /// Check whether this coordinate exists in the set or not.
    /// ```
    /// # use miinaharava::minefield::*;
    /// # use minesweeper_ai::ai::coord_set::*;
    /// use miinaharava::minefield::Matrix;
    ///
    /// let mut set = CoordSet {
    ///     matrix: Matrix([
    ///         [true, true, false],
    ///         [false, true, false],
    ///         [false, false, true]
    ///     ])
    /// };
    ///
    /// assert_eq!(set.contains(Coord(1, 1)), true);
    /// ```
    pub fn contains(&self, coord: Coord<W, H>) -> bool {
        self.matrix.get(coord)
    }

    /// Return an iterator of all the existing coordinates.
    /// ```
    /// # use miinaharava::minefield::*;
    /// # use minesweeper_ai::ai::coord_set::*;
    /// use miinaharava::minefield::Matrix;
    ///
    /// let mut set = CoordSet {
    ///     matrix: Matrix([
    ///         [true, true, false],
    ///         [false, true, false],
    ///         [false, false, true]
    ///     ])
    /// };
    ///
    /// let coords = vec![Coord(0, 0), Coord(1, 0), Coord(1, 1), Coord(2, 2)];
    ///
    /// assert_eq!(set.iter().collect::<Vec<_>>(), coords);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = Coord<W, H>> + '_ {
        self.matrix.0.iter().enumerate().flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, c)| **c)
                .map(move |(x, _)| Coord(x as u8, y as u8))
        })
    }

    /// Return an iterator of all the existing coordinates.
    /// ```
    /// # use miinaharava::minefield::*;
    /// # use minesweeper_ai::ai::coord_set::*;
    /// use miinaharava::minefield::Matrix;
    ///
    /// let mut set = CoordSet {
    ///     matrix: Matrix([
    ///         [true, true, false],
    ///         [false, true, false],
    ///         [false, false, true]
    ///     ])
    /// };
    ///
    /// let coords = vec![Coord(0, 0), Coord(1, 0), Coord(1, 1), Coord(2, 2)];
    ///
    /// assert_eq!(set.iter().collect::<Vec<_>>(), coords);
    /// ```
    pub fn is_empty(&self) -> bool {
        for val in self.matrix.iter().flatten() {
            if *val {
                return false;
            }
        }
        true
    }

    /// Returns an iterator, that returns a mutable boolean which you can use to
    /// remove the specified coordinate, while also returning the specified
    /// coordinate.
    /// ```
    /// # use miinaharava::minefield::*;
    /// # use minesweeper_ai::ai::coord_set::*;
    ///
    /// let mut set = CoordSet::<10, 10>::default();
    ///
    /// let coords = vec![Coord(5, 5), Coord(6, 6), Coord(7, 7)];
    ///
    /// set.insert_many(coords.clone().into_iter());
    ///
    /// for (exists, coord) in set.iter_mut() {
    ///     if coord == Coord(5, 5) {
    ///         *exists = false;
    ///     }
    /// }
    ///
    /// assert_eq!(set.contains(Coord(5, 5)), false);
    /// ```
    #[allow(dead_code)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&mut bool, Coord<W, H>)> + '_ {
        self.matrix.0.iter_mut().enumerate().flat_map(|(y, row)| {
            row.iter_mut()
                .enumerate()
                .filter(|(_, c)| **c)
                .map(move |(x, c)| (c, Coord(x as u8, y as u8)))
        })
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
