//! hello

#![deny(clippy::all)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use std::time::Duration;

use miinaharava::{
    game::{Game, GameWindow},
    sdl2::{event::Event, keyboard::Keycode},
};
use thread_controller::{Difficulty, State, StateStats, StateWrapper, ThreadController};

mod ai;
mod csp;
mod thread_controller;

#[cfg(test)]
mod tests;

/// Represents the current visual state, contains controller for the thread
/// where AI is run, the actual visual game and a delay which controls the speed
/// of the AI.
struct VisualState<'a> {
    /// Controller for the AI Thread
    controller: ThreadController,
    /// Contains everything required to render the visual components.
    game: Game<'a>,
    /// Delay, which determines how long the AI will wait before acting again.
    delay: Duration,
}

impl<'a> VisualState<'a> {
    /// Reset the current state with the specified difficulty.
    pub fn reset_with_difficulty(&mut self, difficulty: Difficulty) {
        *self.controller.state.lock().unwrap() = match difficulty {
            Difficulty::Easy => StateWrapper::Easy(State::new(10)),
            Difficulty::Intermediate => StateWrapper::Intermediate(State::new(40)),
            Difficulty::Expert => StateWrapper::Expert(State::new(99)),
        };
        self.game.timer = 0.;
    }

    /// Adds delay
    pub fn add_delay(&mut self, amount: Duration) {
        self.delay += amount.min(self.delay);
        self.controller.set_delay(Some(self.delay));
    }

    /// Subtracts delay
    pub fn sub_delay(&mut self, amount: Duration) {
        self.delay = (self.delay - amount.min(self.delay)).max(Duration::from_micros(100));
        self.controller.set_delay(Some(self.delay));
    }

    /// Toggles pause
    pub fn toggle_pause(&mut self) {
        self.game.timer_paused = self.controller.toggle_pause();
    }

    /// Draws the actual minefield
    pub fn draw(&mut self) {
        let state = {
            let lock = self.controller.state.lock().unwrap();
            lock.clone()
        };

        self.draw_layout(&state.stats());

        match state {
            StateWrapper::Easy(state) => self.game.draw(&state.minefield, None),
            StateWrapper::Intermediate(state) => self.game.draw(&state.minefield, None),
            StateWrapper::Expert(state) => self.game.draw(&state.minefield, None),
        }
    }

    /// Draws necessary text on the extra layout for Game, such as keybinds and
    /// other useful information about the current game.
    fn draw_layout(&mut self, stats: &StateStats) {
        self.game.extra_layout.clear();
        self.game.append_keybind("1", "Easy");
        self.game.append_keybind("2", "Intermediate");
        self.game.append_keybind("3", "Expert");
        self.game.append_keybind("Space", "Toggle Pause");
        self.game
            .append_keybind("Up/Down", format!("Delay {:.1?}\n", self.delay));

        let total_games = stats.games.0 + stats.games.1;
        let victory_percent = (stats.games.0 as f32 / total_games as f32) * 100.;
        let average_game = stats.ai_time / total_games.max(1);

        self.game
            .append_extra(format!("Games played: {:?}\n", total_games), None, None);
        self.game.append_extra(
            format!("  {} ({:.0}%) Success\n", stats.games.1, victory_percent),
            None,
            None,
        );
        self.game.append_extra(
            format!("AI Time Spent: {:.0?}\n", stats.ai_time),
            None,
            None,
        );
        self.game
            .append_extra(format!("Avg. game: {:.0?}\n", average_game), None, None);
    }
}

/// asd
fn main() {
    let mut window = GameWindow::start();
    let mut game = Game::init(&mut window);

    game.timer = 0.;
    game.timer_paused = false;
    let mut state = VisualState {
        controller: ThreadController::start(StateWrapper::Easy(State::new(10)), game.timer_paused),
        delay: Duration::from_millis(25),
        game,
    };

    while let Some(events) = state.game.update() {
        for event in events.events {
            if let Event::KeyDown {
                keycode: Some(c), ..
            } = event
            {
                use Difficulty::*;
                match c {
                    Keycode::Num1 => state.reset_with_difficulty(Easy),
                    Keycode::Num2 => state.reset_with_difficulty(Intermediate),
                    Keycode::Num3 => state.reset_with_difficulty(Expert),
                    Keycode::Up => state.add_delay(Duration::from_millis(1)),
                    Keycode::Down => state.sub_delay(Duration::from_millis(1)),
                    Keycode::Space => state.toggle_pause(),
                    _ => (),
                }
            }
        }

        state.draw();
    }
}
