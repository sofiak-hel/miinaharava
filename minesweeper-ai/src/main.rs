//! hello

#![deny(clippy::all)]
#![warn(missing_docs)]

use miinaharava::{
    game::{Game, GameWindow},
    minefield::{GameState, Minefield},
    sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton},
};

#[derive(Clone, Copy, Debug)]
pub enum Difficulty {
    Easy,
    Intermediate,
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

fn start_game(game: &mut Game, difficulty: Difficulty) -> Option<Difficulty> {
    game.timer = 0.;
    match difficulty {
        Difficulty::Easy => game_main::<10, 10>(game, 10),
        Difficulty::Intermediate => game_main::<16, 16>(game, 40),
        Difficulty::Expert => game_main::<30, 16>(game, 99),
    }
}

fn game_main<const W: usize, const H: usize>(game: &mut Game, mines: u8) -> Option<Difficulty> {
    let minefield = Minefield::<W, H>::generate(mines);
    let mut next_difficulty = None;

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
        game.timer_paused = minefield.game_state() != GameState::Pending;
        game.draw(&minefield, None);
    }
    next_difficulty
}
