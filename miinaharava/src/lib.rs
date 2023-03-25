//!  hello

#![deny(clippy::all)]
#![warn(missing_docs)]

pub use sdl2;

pub mod game;
pub mod minefield;
pub(crate) mod minefield_renderer;
