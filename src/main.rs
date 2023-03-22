//! hello

#![deny(clippy::all)]
#![allow(missing_docs)]

use game::GameWindow;
use minefield::Minefield;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

use crate::game::Game;
use crate::minefield::GameState;

pub mod game;
pub mod minefield;
pub(crate) mod minefield_renderer;

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

pub fn game_main<const W: usize, const H: usize>(game: &mut Game, mines: u8) -> Option<Difficulty> {
    let mut mouse_pressed = false;
    let mut minefield = Minefield::<W, H>::generate(mines);
    let mut next_difficulty = None;

    while let (Some(events), None) = (game.update(), next_difficulty) {
        for event in events {
            let next_diff = match event {
                Event::MouseButtonUp {
                    mouse_btn, x, y, ..
                } => {
                    mouse_pressed = false;
                    match mouse_btn {
                        MouseButton::Left => {
                            if let Some(coord) = game.get_coord((x, y)) {
                                minefield.reveal(coord).unwrap();
                            }
                        }
                        MouseButton::Right => {
                            if let Some(coord) = game.get_coord((x, y)) {
                                minefield.flag(coord).unwrap();
                            }
                        }
                        _ => {}
                    }
                    None
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Num1 => Some(Difficulty::Easy),
                    Keycode::Num2 => Some(Difficulty::Intermediate),
                    Keycode::Num3 => Some(Difficulty::Expert),
                    _ => None,
                },
                Event::MouseButtonDown { .. } => {
                    mouse_pressed = true;
                    None
                }
                _ => None,
            };
            next_difficulty = next_diff.or(next_difficulty);
        }
        game.timer_paused = minefield.game_state() != GameState::Pending;
        game.draw(
            &minefield,
            mouse_pressed && minefield.game_state() == GameState::Pending,
        );
    }
    next_difficulty
}
