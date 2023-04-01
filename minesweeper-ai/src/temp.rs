//! hello

#![deny(clippy::all)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use std::time::Instant;

use ai::{ponder, Decision};
use miinaharava::{
    game::{Game, GameWindow},
    minefield::{GameState, Minefield},
    sdl2::{event::Event, keyboard::Keycode},
};

mod ai;
mod csp;

#[cfg(test)]
mod tests;

/// Represents a difficulty level
#[derive(Clone, Copy, Debug)]
enum Difficulty {
    /// 10x10 field with 10 mines
    Easy,
    /// 16x16 field with 40 mines
    Intermediate,
    /// 30x16 field with 99 mines
    Expert,
}

/// main docs
pub fn main() {
    // let mut difficulty = Some(Difficulty::Easy);
    // while let Some(diff) = difficulty {
    //     game.extra_layout.clear();
    //     game.append_extra(format!("Difficulty: {:?}\n\n", diff), None, None);
    //     game.append_keybind("1", format!("{:?}", Difficulty::Easy));
    //     game.append_keybind("2", format!("{:?}", Difficulty::Intermediate));
    //     game.append_keybind("3", format!("{:?}", Difficulty::Expert));
    //     difficulty = start_game(&mut game, diff);
    // }
}

/// qwe
fn start_game(game: &mut Game, difficulty: Difficulty) {
    game.timer = 0.;
    match difficulty {
        Difficulty::Easy => main_loop::<10, 10>(10),
        Difficulty::Intermediate => main_loop::<16, 16>(40),
        Difficulty::Expert => main_loop::<30, 16>(99),
    }
}

pub fn main_loop<const W: usize, const H: usize>(mine_count: u8) {
    let mut last_move = Instant::now();

    loop {
        let mut minefield = Minefield::generate(mine_count).unwrap();

        let now = Instant::now();
        if (now - last_move).as_secs_f32() > 1. {
            last_move = now;
            if let Some(decisions) = ponder(&minefield) {
                for decision in decisions {
                    match decision {
                        Decision::Reveal(coord) => minefield.reveal(coord),
                        Decision::Flag(coord) => minefield.flag(coord),
                    }
                    .ok();
                }
            }
        }
    }
}
