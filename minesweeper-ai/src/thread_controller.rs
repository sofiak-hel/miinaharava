//! Represents a controller for the thread that is mainly used for the AI to do
//! it's things so that it will not clog up any possible user-interface that the
//! program is shipped and run with.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use miinaharava::minefield::{GameState, Minefield};

use crate::ai::{ponder, Decision};

/// Macro that is useful for measuring how long a certain expression took.
macro_rules! measure {
    ( $x:expr ) => {{
        let before = Instant::now();
        let value = $x;
        (value, Instant::now() - before)
    }};
}

/// Represents a difficulty level
#[derive(Clone, Copy, Debug)]
pub enum Difficulty {
    /// 10x10 field with 10 mines
    Easy,
    /// 16x16 field with 40 mines
    Intermediate,
    /// 30x16 field with 99 mines
    Expert,
}

/// Controller for the Ai's [State], which continually plays games by the AI.
pub struct ThreadController {
    /// The state that is being processed
    pub state: Arc<Mutex<StateWrapper>>,
    /// Represents the duration which is waited (if any) before processing the
    /// next thing
    delay: Arc<Mutex<Option<Duration>>>,
    /// Whether the thread should still continue, or if it should entirely shut
    /// down.
    running: Arc<AtomicBool>,
    /// Whether the thread should be paused.
    paused: Arc<AtomicBool>,
    /// Join handle, which represents the thread itself. Used in the
    /// Drop-implementation
    join_handle: Option<JoinHandle<()>>,
}

impl ThreadController {
    /// Start the thread that continually plays games
    pub fn start(state: StateWrapper, paused: bool) -> ThreadController {
        let state = Arc::new(Mutex::new(state));
        let running = Arc::new(AtomicBool::new(true));
        let paused = Arc::new(AtomicBool::new(paused));
        let delay = Arc::new(Mutex::new(None));

        let join_handle = Some(std::thread::spawn({
            let running = running.clone();
            let state = state.clone();
            let delay = delay.clone();
            let paused = paused.clone();
            move || {
                let mut last_move = Instant::now();
                while running.load(Ordering::Relaxed) {
                    if !paused.load(Ordering::Relaxed) {
                        let delay = delay.lock().unwrap();
                        if let Some(delay) = delay.as_ref() {
                            let now = Instant::now();
                            if now - last_move <= *delay {
                                continue;
                            }
                            last_move = now;
                        }
                        let mut lock = state.lock().unwrap();
                        lock.process(delay.is_none());
                    }
                }
            }
        }));

        ThreadController {
            state,
            running,
            join_handle,
            delay,
            paused,
        }
    }

    /// Change the delay used by the thread.
    pub fn set_delay(&mut self, delay: Option<Duration>) {
        *self.delay.lock().unwrap() = delay;
    }

    /// Toggle whether the thread should be paused
    pub fn toggle_pause(&mut self) -> bool {
        let value = !self.paused.load(Ordering::Relaxed);
        self.paused.store(value, Ordering::Relaxed);
        value
    }
}

impl Drop for ThreadController {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.join_handle.take().unwrap().join().unwrap()
    }
}

/// Represents the current State of the Game, but wrapped in an enum that can be
/// handled without having to define generics for the handling struct.
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum StateWrapper {
    /// Represents a state for the Easy-difficulty (10x10)
    Easy(State<10, 10>),
    /// Represents a state for the Intermediate-difficulty (16x16)
    Intermediate(State<16, 16>),
    /// Represents a state for the Expert-difficulty (30x16)
    Expert(State<30, 16>),
}

impl StateWrapper {
    /// Simply calls `process` on the current State, convenience function to
    /// avoid having to match generics.
    pub fn process(&mut self, super_speed: bool) {
        match self {
            StateWrapper::Easy(s) => s.process(super_speed),
            StateWrapper::Intermediate(s) => s.process(super_speed),
            StateWrapper::Expert(s) => s.process(super_speed),
        }
    }

    /// Returns the stats for the current State, convenience function to avoid
    /// having to match generics.
    pub fn stats(&self) -> StateStats {
        match self {
            StateWrapper::Easy(s) => s.stats,
            StateWrapper::Intermediate(s) => s.stats,
            StateWrapper::Expert(s) => s.stats,
        }
    }
}

/// State of the current set of games being played by the AI. This struct is
/// reset every time difficulty changes (or the game is otherwise reset).
#[derive(Clone)]
pub struct State<const W: usize, const H: usize> {
    /// The current minefield, regenerated after previous is completed.
    pub minefield: Minefield<W, H>,
    /// Current Stats for the state
    pub stats: StateStats,
    /// The current stack of decisions from the last ponder.
    decisions: Vec<Decision<W, H>>,
}

/// The common statistics from a State, that are not bound by generics.
#[derive(Debug, Default, Clone, Copy)]
pub struct StateStats {
    /// How many mines are in the current game state (re-used when regenerating
    /// minefield)
    pub mines: u8,
    /// How many games have been finished (Victories, Losses)
    pub games: (u32, u32),
    /// How much time has the AI spent [ponder]ing
    pub ai_time: Duration,
    /// How much time has been spent generating minefields
    pub generation_time: Duration,
    /// How much time has been spent revealing or flagging tiles.
    pub decision_time: Duration,
}

impl<const W: usize, const H: usize> State<W, H> {
    /// Creates a new state, only plays a certain difficulty.
    pub fn new(mine_count: u8) -> State<W, H> {
        State {
            minefield: Minefield::generate(mine_count).unwrap(),
            stats: StateStats {
                mines: mine_count,
                ..Default::default()
            },
            decisions: Vec::new(),
        }
    }

    /// 1. If game already over, generate a new map
    /// 2. If there are no [Decision]s left, [ponder] and measure the time
    /// 3. Act on the next [Decision] (multiple if super_speed is on)
    pub fn process(&mut self, super_speed: bool) {
        if self.minefield.game_state() != GameState::Pending {
            match self.minefield.game_state() {
                GameState::Victory => self.stats.games.0 += 1,
                GameState::GameOver => self.stats.games.1 += 1,
                _ => {}
            }
            let (minefield, time) = measure!(Minefield::generate(self.stats.mines).unwrap());
            self.minefield = minefield;
            self.stats.generation_time += time;
            self.decisions = Vec::new();
        } else if self.decisions.is_empty() {
            let (decisions, time) = measure!(ponder(&self.minefield));
            self.stats.ai_time += time;
            self.decisions = decisions;
        }
        while let Some(decision) = self.decisions.pop() {
            let (_, time) = measure!({
                match decision {
                    Decision::Reveal(coord) => self.minefield.reveal(coord),
                    Decision::Flag(coord) => self.minefield.flag(coord),
                }
                .ok();
            });
            self.stats.decision_time += time;
            if !super_speed {
                break;
            }
        }
    }
}
