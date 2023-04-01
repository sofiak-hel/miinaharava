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
    let mut window = GameWindow::start();
    let mut game = Game::init(&mut window);

    let mut difficulty = Some(Difficulty::Easy);
    while let Some(diff) = difficulty {
        game.extra_layout.clear();
        game.append_extra(format!("Difficulty: {:?}\n\n", diff), None, None);
        game.append_keybind("1", format!("{:?}", Difficulty::Easy));
        game.append_keybind("2", format!("{:?}", Difficulty::Intermediate));
        game.append_keybind("3", format!("{:?}", Difficulty::Expert));
        difficulty = start_game(&mut game, diff);
    }
}

/// qwe
fn start_game(game: &mut Game, difficulty: Difficulty) -> Option<Difficulty> {
    game.timer = 0.;
    match difficulty {
        Difficulty::Easy => game_main::<10, 10>(game, 10),
        Difficulty::Intermediate => game_main::<16, 16>(game, 40),
        Difficulty::Expert => game_main::<30, 16>(game, 99),
    }
}

/// asd
fn game_main<const W: usize, const H: usize>(game: &mut Game, mines: u8) -> Option<Difficulty> {
    let mut minefield = Minefield::<W, H>::generate(mines).unwrap();
    let mut next_difficulty = None;
    let mut last_move = Instant::now();

    while let (Some(events), None) = (game.update(), next_difficulty) {
        for event in events.events {
            let next_diff = match event {
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Num1 => Some(Difficulty::Easy),
                    Keycode::Num2 => Some(Difficulty::Intermediate),
                    Keycode::Num3 => Some(Difficulty::Expert),
                    _ => None,
                },
                _ => None,
            };
            next_difficulty = next_diff.or(next_difficulty);
        }

        // AI decision making here
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

        game.timer_paused = minefield.game_state() != GameState::Pending;
        game.draw(&minefield, None);
    }
    next_difficulty
}
