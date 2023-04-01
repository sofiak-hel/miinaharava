//! hello

#![deny(clippy::all)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use std::time::{Duration, Instant};

use miinaharava::{
    game::{Game, GameWindow},
    minefield::Minefield,
    sdl2::{event::Event, keyboard::Keycode},
};
use thread_controller::{Difficulty, StateWrapper, ThreadController};

mod ai;
mod csp;
mod thread_controller;

#[cfg(test)]
mod tests;

/// TODO: docs
// pub fn main() {
//     let true_start = Instant::now();
//     loop {
//         let stats = {
//             let state = thread_controller.state.lock().unwrap();
//             state.stats()
//         };
//         let games_played = stats.games.0 + stats.games.1;
//         if games_played > 1 {
//             dbg!(&stats);
//             let subtotal = stats.ai_time + stats.generation_time + stats.decision_time;
//             let total = (Instant::now() - true_start).as_secs_f32();
//             println!("total: {:?}, boilerplate: {:?}", total, total - subtotal);
//             println!("{:?}", total);
//             break;
//         }
//     }
// }

/// main docs
pub fn main() {
    let mut window = GameWindow::start();
    let mut game = Game::init(&mut window);
    let mut thread_controller = ThreadController::start();

    // let mut difficulty = Some(Difficulty::Easy);
    game_main(&mut game, &mut thread_controller);
    // while let Some(diff) = difficulty {
    //     game.extra_layout.clear();
    //     game.append_extra(format!("Difficulty: {:?}\n\n", diff), None, None);
    //     game.append_keybind("1", format!("{:?}", Difficulty::Easy));
    //     game.append_keybind("2", format!("{:?}", Difficulty::Intermediate));
    //     game.append_keybind("3", format!("{:?}", Difficulty::Expert));
    // }
}

/// asd
fn game_main(game: &mut Game, thread_controller: &mut ThreadController) {
    game.timer = 0.;
    game.timer_paused = false;
    let mut delay = Duration::from_millis(25);
    thread_controller.set_delay(Some(delay));

    while let Some(events) = game.update() {
        for event in events.events {
            if let Event::KeyDown {
                keycode: Some(c), ..
            } = event
            {
                match c {
                    Keycode::Num1 => thread_controller.reset_with_difficulty(Difficulty::Easy),
                    Keycode::Num2 => {
                        thread_controller.reset_with_difficulty(Difficulty::Intermediate)
                    }
                    Keycode::Num3 => thread_controller.reset_with_difficulty(Difficulty::Expert),
                    Keycode::Up => {
                        delay += Duration::from_millis(1);
                        thread_controller.set_delay(Some(delay));
                    }
                    Keycode::Down => {
                        delay -= Duration::from_millis(1).min(delay);
                        delay = delay.max(Duration::from_micros(100));
                        thread_controller.set_delay(Some(delay));
                    }
                    Keycode::Space => thread_controller.toggle_pause(),
                    _ => (),
                }
            }
        }

        let state = {
            let lock = thread_controller.state.lock().unwrap();
            lock.clone()
        };

        game.extra_layout.clear();
        game.append_keybind("1", "Easy");
        game.append_keybind("2", "Intermediate");
        game.append_keybind("3", "Expert");
        game.append_keybind("Space", "Toggle Pause");
        game.append_keybind("Up/Down", format!("Delay {:.1?}\n", delay));

        let total_games = state.stats().games.0 + state.stats().games.1;
        let victory_percent = (state.stats().games.0 as f32 / total_games as f32) * 100.;
        let average_game = state.stats().ai_time / total_games.max(1);

        game.append_extra(format!("Games played: {:?}\n", total_games), None, None);
        game.append_extra(
            format!(
                "  {} ({:.0}%) Success\n",
                state.stats().games.1,
                victory_percent
            ),
            None,
            None,
        );
        game.append_extra(
            format!("AI Time Spent: {:.0?}\n", state.stats().ai_time),
            None,
            None,
        );
        game.append_extra(format!("Avg. game: {:.0?}\n", average_game), None, None);

        match state {
            StateWrapper::Easy(state) => game.draw(&state.minefield, None),
            StateWrapper::Intermediate(state) => game.draw(&state.minefield, None),
            StateWrapper::Expert(state) => game.draw(&state.minefield, None),
        }
    }
}
