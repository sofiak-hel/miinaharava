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
    pub state: Arc<Mutex<StateWrapper>>,
    delay: Arc<Mutex<Option<Duration>>>,
    running: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<()>>,
}

impl ThreadController {
    /// Start the thread that continually plays games
    pub fn start() -> ThreadController {
        let state = Arc::new(Mutex::new(StateWrapper::Easy(State::new(10))));
        let running = Arc::new(AtomicBool::new(true));
        let paused = Arc::new(AtomicBool::new(false));
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

    pub fn set_delay(&mut self, delay: Option<Duration>) {
        *self.delay.lock().unwrap() = delay;
    }

    pub fn toggle_pause(&mut self) {
        self.paused
            .store(!self.paused.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    pub fn reset_with_difficulty(&mut self, difficulty: Difficulty) {
        *self.state.lock().unwrap() = match difficulty {
            Difficulty::Easy => StateWrapper::Easy(State::new(10)),
            Difficulty::Intermediate => StateWrapper::Intermediate(State::new(40)),
            Difficulty::Expert => StateWrapper::Expert(State::new(99)),
        }
    }
}

impl Drop for ThreadController {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.join_handle.take().unwrap().join().unwrap()
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum StateWrapper {
    Easy(State<10, 10>),
    Intermediate(State<16, 16>),
    Expert(State<30, 16>),
}

impl StateWrapper {
    pub fn process(&mut self, super_speed: bool) {
        match self {
            StateWrapper::Easy(s) => s.process(super_speed),
            StateWrapper::Intermediate(s) => s.process(super_speed),
            StateWrapper::Expert(s) => s.process(super_speed),
        }
    }

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
    pub minefield: Minefield<W, H>,
    pub stats: StateStats,
    decisions: Vec<Decision<W, H>>,
}

/// The common statistics from a State, that are not bound by generics.
#[derive(Debug, Default, Clone, Copy)]
pub struct StateStats {
    pub mines: u8,
    pub games: (u32, u32),
    pub ai_time: Duration,
    pub generation_time: Duration,
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
