//! Library for drawing and using a simple minesweeper implementation.
//!
//! [minefield] module contains the whole nonvisual abstraction and is
//! sufficient if rendering is not needed.
//!
//! [game] contains everything related to drawing, rendering and capturing
//! events from the window in question.

#![deny(clippy::all)]
#![warn(missing_docs)]
#![warn(clippy::missing_errors_doc)]

pub use sdl2;

pub mod game;
pub mod minefield;
pub(crate) mod minefield_renderer;
