//! Contains the mechanical part for Minesweeper ([Minefield]), contains nothing related for
//! drawing and is entirely sufficient in of itself if a simple abstract
//! representation is only needed.

use std::fmt::Debug;

use arrayvec::ArrayVec;

/// Represents a tile coordinate on the minefield.
#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct Coord<const W: usize, const H: usize>(pub usize, pub usize);

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
                    list.push(Coord(newx as usize, newy as usize))
                }
            }
        }
        list
    }

    /// Returns a random valid coordinate
    pub fn random() -> Coord<W, H> {
        Coord(rand::random::<usize>() % W, rand::random::<usize>() % H)
    }
}

impl<const W: usize, const H: usize> Debug for Coord<W, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
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

impl<T: Copy, const W: usize, const H: usize> Matrix<T, W, H> {
    /// Get element in position of Coord from the matrix
    pub fn get(&self, coord: Coord<W, H>) -> T {
        self.0[coord.1][coord.0]
    }

    /// Set element in position of Coord from the matrix
    pub fn set(&mut self, coord: Coord<W, H>, item: T) {
        self.0[coord.1][coord.0] = item;
    }

    /// Return an iterator for the rows
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
}

impl<const W: usize, const H: usize> Minefield<W, H> {
    /// Generate a new minefield with the provided amount of mines.
    ///
    /// # Errors
    /// - [MinefieldError::TooManyMines] if the amount of mines is too large.
    pub fn generate(mines: u8) -> Result<Self, MinefieldError> {
        let mut mine_indices = [[false; W]; H];
        if mines as usize > W * H {
            Err(MinefieldError::TooManyMines)
        } else {
            for _ in 0..mines {
                let mut coord = Coord::<W, H>::random();
                while mine_indices[coord.1][coord.0] {
                    coord = Coord::random();
                }
                mine_indices[coord.1][coord.0] = true;
            }

            Ok(Minefield {
                mine_indices: Matrix(mine_indices),
                field: Matrix([[Cell::Hidden; W]; H]),
                mines,
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
        }
    }

    /// Returns the current state of the game.
    pub fn game_state(&self) -> GameState {
        if self.field.into_iter().flatten().any(|c| c == Cell::Mine) {
            GameState::GameOver
        } else if self
            .field
            .into_iter()
            .flatten()
            .zip(self.mine_indices.into_iter().flatten())
            .all(|(c, is_mine)| (c == Cell::Hidden || c == Cell::Flag) == is_mine)
        {
            GameState::Victory
        } else {
            GameState::Pending
        }
    }

    /// Attempts to reveal a tile.
    ///
    /// # Errors
    /// - [MinefieldError::GameHasEnded] if the game is already over
    /// - [MinefieldError::InvalidCoordinate] if the attempted coordinate was not valid.
    pub fn reveal(&mut self, coord: Coord<W, H>) -> Result<(), MinefieldError> {
        if self.game_state() != GameState::Pending {
            Err(MinefieldError::GameHasEnded)
        } else if coord.0 >= W || coord.1 >= H {
            Err(MinefieldError::InvalidCoordinate)
        } else {
            let field_cell = self.field.get(coord);
            if field_cell == Cell::Flag || field_cell == Cell::Hidden {
                let cell = self.cell_contents(coord);
                self.field.set(coord, cell);
                if cell == Cell::Empty {
                    for neighbor in coord.neighbours() {
                        match self.reveal(neighbor) {
                            Err(MinefieldError::GameHasEnded) => break,
                            e => e?,
                        };
                    }
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
    pub fn flag(&mut self, coord: Coord<W, H>) -> Result<(), MinefieldError> {
        if self.game_state() != GameState::Pending {
            Err(MinefieldError::GameHasEnded)
        } else if coord.0 >= W || coord.1 >= H {
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
            Ok(())
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
