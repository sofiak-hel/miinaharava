//! Contains the mechanical part for Minesweeper ([Minefield]), contains nothing related for
//! drawing and is entirely sufficient in of itself if a simple abstract
//! representation is only needed.

use std::{fmt::Debug, hash::Hasher};

use arrayvec::ArrayVec;

/// Represents a tile coordinate on the minefield.
#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct Coord<const W: usize, const H: usize>(pub u8, pub u8);

impl<const W: usize, const H: usize> Coord<W, H> {
    /// Returns all possible 8 neighboring coordinates for the given coordinate.
    /// Does not return impossible coordinates, such as below 0 or above
    /// minefield limits.
    pub fn neighbours(&self) -> ArrayVec<Coord<W, H>, 8> {
        let mut list = ArrayVec::new();
        for y in -1..=1 {
            for x in -1..=1 {
                let (newx, newy) = (self.0 as i8 + x, self.1 as i8 + y);
                if newx >= 0 && newy >= 0 && newx < W as i8 && newy < H as i8 && (x != 0 || y != 0)
                {
                    list.push(Coord(newx as u8, newy as u8))
                }
            }
        }
        list
    }

    /// Returns a random valid coordinate
    pub fn random() -> Coord<W, H> {
        Coord(
            rand::random::<u8>() % (W as u8),
            rand::random::<u8>() % (H as u8),
        )
    }
}

impl<const W: usize, const H: usize> Debug for Coord<W, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl<const W: usize, const H: usize> std::hash::Hash for Coord<W, H> {
    fn hash<Hash: Hasher>(&self, state: &mut Hash) {
        state.write_u8(self.1 * W as u8 + self.0);
    }
}

/// Custom error for minefields
#[derive(Debug, PartialEq, Eq)]
pub enum MinefieldError {
    /// Coordinate was invalid
    InvalidCoordinate,
    /// Can not have this many mines on a minefield this size.
    TooManyMines,
    /// Game already ended, unable to performa any actions.
    GameHasEnded,
}

/// Represents a cell on the "visible" board.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    /// Empty cell, does not contain a mine and does not have a mine in neighboring tiles.
    Empty,
    /// Has a mine in a neighboring tile, number will tell how many mines.
    Label(u8),
    /// Hidden tile but flagged as a suspected mine.
    Flag,
    /// Hidden tile.
    Hidden,
    /// Revealed to be a mine, having one in the board always results in a
    /// failed game state.
    Mine,
}

/// Represents the state of the game currently
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum GameState {
    /// The game has been won, all non-mines have been revealed.
    Victory,
    /// A mine has been revealed, the game is lost.
    GameOver,
    /// The game still pending; not yet won or lost.
    Pending,
}

/// Generic struct for a 2D matrix of type T
#[derive(Debug, PartialEq, Clone, Eq, Copy)]
pub struct Matrix<T: Copy, const W: usize, const H: usize>(pub [[T; W]; H]);

impl<T: Copy + Default, const W: usize, const H: usize> Default for Matrix<T, W, H> {
    fn default() -> Self {
        Matrix([[T::default(); W]; H])
    }
}

impl<T: Copy, const W: usize, const H: usize> Matrix<T, W, H> {
    /// Get element in position of Coord from the matrix
    #[inline]
    pub fn get(&self, coord: Coord<W, H>) -> T {
        unsafe {
            *self
                .0
                .get_unchecked(coord.1 as usize)
                .get_unchecked(coord.0 as usize)
        }
    }

    /// Set element in position of Coord from the matrix
    #[inline]
    pub fn set(&mut self, coord: Coord<W, H>, item: T) {
        unsafe {
            *self
                .0
                .get_unchecked_mut(coord.1 as usize)
                .get_unchecked_mut(coord.0 as usize) = item;
        }
    }

    /// Return an iterator for the rows
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &[T; W]> {
        self.0.iter()
    }
}

impl<T: Copy, const W: usize, const H: usize> IntoIterator for Matrix<T, W, H> {
    type Item = [T; W];
    type IntoIter = std::array::IntoIter<Self::Item, H>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Represents a mechanical abstract minefield in minesweeper
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Minefield<const W: usize, const H: usize> {
    mine_indices: Matrix<bool, W, H>,
    /// The visible field
    pub field: Matrix<Cell, W, H>,
    /// How many mines are in the field.
    pub mines: u8,
    game_state: GameState,
}

pub type Reveal<const W: usize, const H: usize> = (Coord<W, H>, Cell);

impl<const W: usize, const H: usize> Minefield<W, H> {
    /// Generate a new minefield with the provided amount of mines.
    ///
    /// # Errors
    /// - [MinefieldError::TooManyMines] if the amount of mines is too large.
    pub fn generate(mines: u8) -> Result<Self, MinefieldError> {
        let mut mine_indices = Matrix([[false; W]; H]);
        if mines as usize > W * H {
            Err(MinefieldError::TooManyMines)
        } else {
            for _ in 0..mines {
                let mut coord = Coord::<W, H>::random();
                while mine_indices.get(coord) {
                    coord = Coord::random();
                }
                mine_indices.set(coord, true);
            }

            Ok(Minefield {
                mine_indices,
                field: Matrix([[Cell::Hidden; W]; H]),
                mines,
                game_state: GameState::Pending,
            })
        }
    }
    /// Generate a new minefield with the provided amount of mines.
    ///
    /// # Errors
    /// - [MinefieldError::TooManyMines] if the amount of mines is too large.
    pub fn with_mines(mines: Matrix<bool, W, H>) -> Self {
        Minefield {
            mine_indices: mines,
            field: Matrix([[Cell::Hidden; W]; H]),
            mines: mines
                .into_iter()
                .map(|row| row.iter().filter(|i| **i).count() as u8)
                .sum(),
            game_state: GameState::Pending,
        }
    }

    /// Return the current state of the game immutably.
    #[inline]
    pub fn game_state(&self) -> GameState {
        self.game_state
    }

    /// Update the current state of the game.
    #[inline]
    fn update_game_state(&mut self) {
        self.game_state = if self.field.iter().flatten().any(|c| *c == Cell::Mine) {
            GameState::GameOver
        } else if self
            .field
            .into_iter()
            .flatten()
            .zip(self.mine_indices.iter().flatten())
            .all(|(c, is_mine)| (c == Cell::Hidden || c == Cell::Flag) == *is_mine)
        {
            GameState::Victory
        } else {
            GameState::Pending
        };
    }

    /// Attempts to reveal a tile.
    ///
    /// # Errors
    /// - [MinefieldError::GameHasEnded] if the game is already over
    /// - [MinefieldError::InvalidCoordinate] if the attempted coordinate was not valid.
    pub fn reveal(&mut self, coord: Coord<W, H>) -> Result<Vec<Reveal<W, H>>, MinefieldError> {
        let mut reveals = Vec::new();
        self._reveal(coord, true, &mut reveals)?;
        Ok(reveals)
    }

    /// Private reveal function that contains `update_state`-parameter simply to
    /// help with recursion, literally halving the amount of time that reveal
    /// would otherwise take.
    fn _reveal(
        &mut self,
        coord: Coord<W, H>,
        update_state: bool,
        reveals: &mut Vec<Reveal<W, H>>,
    ) -> Result<(), MinefieldError> {
        if self.game_state() != GameState::Pending {
            Err(MinefieldError::GameHasEnded)
        } else if coord.0 >= (W as u8) || coord.1 >= (H as u8) {
            Err(MinefieldError::InvalidCoordinate)
        } else {
            let field_cell = self.field.get(coord);
            if field_cell == Cell::Flag || field_cell == Cell::Hidden {
                let cell = self.cell_contents(coord);
                self.field.set(coord, cell);
                reveals.push((coord, cell));
                if cell == Cell::Empty {
                    for neighbor in coord.neighbours() {
                        match self._reveal(neighbor, false, reveals) {
                            Err(MinefieldError::GameHasEnded) => break,
                            e => e?,
                        };
                    }
                }
                if update_state {
                    self.update_game_state();
                }
            }
            Ok(())
        }
    }

    /// Attempts to flag a tile.
    ///
    /// # Errors
    /// - [MinefieldError::GameHasEnded] if the game is already over
    /// - [MinefieldError::InvalidCoordinate] if the attempted coordinate was not valid.
    pub fn flag(&mut self, coord: Coord<W, H>) -> Result<Vec<Reveal<W, H>>, MinefieldError> {
        if self.game_state() != GameState::Pending {
            Err(MinefieldError::GameHasEnded)
        } else if coord.0 >= (W as u8) || coord.1 >= (H as u8) {
            Err(MinefieldError::InvalidCoordinate)
        } else {
            self.field.set(
                coord,
                match self.field.get(coord) {
                    Cell::Flag => Cell::Hidden,
                    Cell::Hidden => Cell::Flag,
                    c => c,
                },
            );
            Ok(Vec::new())
        }
    }

    fn cell_contents(&self, coord: Coord<W, H>) -> Cell {
        if self.is_mine(coord) {
            Cell::Mine
        } else {
            let mines = coord
                .neighbours()
                .iter()
                .filter(|c| self.is_mine(**c))
                .count() as u8;
            if mines == 0 {
                Cell::Empty
            } else {
                Cell::Label(mines)
            }
        }
    }

    fn is_mine(&self, coord: Coord<W, H>) -> bool {
        self.mine_indices.get(coord)
    }
}

#[cfg(test)]
impl<const W: usize, const H: usize> Minefield<W, H> {
    pub fn get_mine_indices(&mut self) -> &mut Matrix<bool, W, H> {
        &mut self.mine_indices
    }
}
