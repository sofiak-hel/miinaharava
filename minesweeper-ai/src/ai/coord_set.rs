//! TODO: Docs
//! Also, TODO: Tests for this module
use miinaharava::minefield::{Coord, Matrix};

/// TODO: Docs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CoordSet<const W: usize, const H: usize> {
    /// Inner matrix
    pub matrix: Matrix<bool, W, H>,
}

impl<const W: usize, const H: usize> CoordSet<W, H> {
    /// TODO: Docs
    pub fn insert(&mut self, coord: Coord<W, H>) {
        self.matrix.set(coord, true);
    }

    /// TODO: Docs
    pub fn remove(&mut self, coord: Coord<W, H>) {
        self.matrix.set(coord, false);
    }

    /// TODO: Docs
    pub fn contains(&self, coord: Coord<W, H>) -> bool {
        self.matrix.get(coord)
    }

    /// TODO: Docs
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

    /// TODO: Docs
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

    /// TODO: Docs
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

    /// TODO: Docs
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
