use miinaharava::minefield::{Coord, Matrix};

/// TODO: Docs
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
