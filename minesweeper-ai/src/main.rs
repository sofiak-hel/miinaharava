//! hello

#![deny(clippy::all)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use argh::FromArgs;
use miinaharava::{
    game::{Game, GameWindow},
    sdl2::{event::Event, keyboard::Keycode},
};
use std::time::{Duration, Instant};
use thread_controller::{Difficulty, GuessStats, StateStats, StateWrapper, ThreadController};

mod ai;
mod thread_controller;

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
        *self.controller.state.lock().unwrap() = StateWrapper::from(difficulty);
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

/// The main function, ran at the start of the program
fn main() {
    let args: CommandLineArguments = argh::from_env();
    let difficulty = args.difficulty.unwrap_or(Difficulty::Easy);
    let duration = args.seconds.map(|s| Duration::from_secs(s as u64));

    if args.headless {
        let max_games = if duration.is_none() {
            Some(args.games.unwrap_or(1000))
        } else {
            args.games
        };
        let (stats, time) = {
            let before = Instant::now();
            let thread_controller = ThreadController::start(difficulty.into(), false, max_games);
            loop {
                if let Some(max_games) = max_games {
                    let stats = {
                        let state = thread_controller.state.lock().unwrap();
                        state.stats()
                    };
                    let total = stats.games.0 + stats.games.1;
                    if total >= max_games {
                        break;
                    }
                    println!(" {} / {}", stats.games.0 + stats.games.1, max_games);
                }
                if let Some(duration) = duration {
                    let passed = Instant::now() - before;
                    if passed >= duration {
                        break;
                    }
                    println!(" Passed time: {:.1?}", passed);
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let lock = thread_controller.state.lock().unwrap();
            (lock.stats(), Instant::now() - before)
        };

        stats.print(difficulty, time);
    } else {
        start_with_window(difficulty);
    }
}

/// Start the program with a visual interface for a neat empiric feel.
fn start_with_window(difficulty: Difficulty) {
    let mut window = GameWindow::start();
    let mut game = Game::init(&mut window);

    game.timer = 0.;
    game.timer_paused = false;
    let mut state = VisualState {
        controller: ThreadController::start(difficulty.into(), game.timer_paused, None),
        delay: Duration::from_millis(25),
        game,
    };
    state.controller.set_delay(Some(state.delay));

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

/// Commandline arguments that are accepted
#[derive(FromArgs)]
struct CommandLineArguments {
    /// run headlessly, more performant but no visuals
    #[argh(switch, short = 'h')]
    headless: bool,

    /// difficulty of the game, easy by default
    #[argh(option, from_str_fn(difficulty_from_str))]
    difficulty: Option<Difficulty>,

    /// number of games that should be played, 1000 by default (does not apply on windowed-mode)
    #[argh(option)]
    games: Option<u32>,

    /// number of seconds to play games (if used with games, first one to finish halts program)
    #[argh(option)]
    seconds: Option<u32>,
}

/// Try to parse difficulty from string
fn difficulty_from_str(value: &str) -> Result<Difficulty, String> {
    Ok(match value.to_lowercase().trim() {
        "easy" => Difficulty::Easy,
        "intermediate" | "med" | "medium" => Difficulty::Intermediate,
        "expert" | "ex" | "hard" => Difficulty::Expert,
        _ => Err("difficulty must be either 'easy', 'intermediate' or 'expert'")?,
    })
}

impl StateStats {
    /// Prints the state stats in a neat manner
    pub fn print(&self, difficulty: Difficulty, time: Duration) {
        let total_games = self.games.0 + self.games.1;
        let vic_perc = (self.games.0 as f32 / total_games as f32) * 100.;
        let loss_perc = (self.games.1 as f32 / total_games as f32) * 100.;

        println!("-----------------");
        println!("Statistics:");
        println!("Game difficulty: {:?}", difficulty);

        println!("\n  Total time spent: {:.1?}", time);

        print!("    AI thinking: {:.1?}", self.ai_time);
        println!(" ({:.1?} avg.)", self.ai_time / total_games);

        print!("    flagging + revealing: {:.1?}", self.decision_time);
        println!(" ({:.1?} avg.)", self.decision_time / total_games);

        print!("    board generating: {:.1?}", self.generation_time);
        println!(" ({:.1?} avg.)", self.generation_time / total_games);

        println!("\n  Total games played: {}", self.games.0 + self.games.1);
        println!("    Victories: {}, ({}%)", self.games.0, vic_perc);
        println!("    Losses: {}, ({}%)", self.games.1, loss_perc);

        let mut clone = self.guess_stats;
        let total_guesses = clone.iter_mut().reduce(|a, b| a.combine(b));

        if let Some(total_guesses) = total_guesses {
            println!("\nTotal guesses:");
            total_guesses.print(total_games);
        }

        for (i, guess_stats) in self.guess_stats.iter().enumerate() {
            if guess_stats.amount_of_guesses > 0 {
                println!("\nStats for guesses ~{}-{}%", i * 10, (i + 1) * 10);
                guess_stats.print(total_games);
            }
        }
    }
}

impl GuessStats {
    /// Prints some version of game stats in a neat manner
    pub fn print(&self, total_games: u32) {
        let guess_perc = (self.successful_guesses as f32 / self.amount_of_guesses as f32) * 100.;
        println!("  Amount of guesses: {}", self.amount_of_guesses);
        println!(
            "  Successful: {} ({}%)",
            self.successful_guesses, guess_perc
        );
        println!("  Average guess success: {}%", self.average_guess * 100.);
        println!(
            "  Average amount of guesses: {:.2}",
            self.amount_of_guesses as f32 / total_games as f32
        );
    }
}
