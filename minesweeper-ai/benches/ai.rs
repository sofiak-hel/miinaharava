use criterion::{black_box, criterion_group, criterion_main, Criterion};
use miinaharava::minefield::{Coord, GameState, Minefield};
use minesweeper_ai::ai::{ponder, Decision::*};

pub fn benchmark_specific_difficulty<const W: usize, const H: usize>(mines: u8) {
    let mut minefield = Minefield::<W, H>::generate(mines).unwrap();
    while minefield.game_state() == GameState::Pending {
        let decisions = ponder(&minefield);

        for decision in decisions {
            match decision {
                Flag(c) => minefield.flag(c),
                Reveal(c) => minefield.reveal(c),
            }
            .ok();
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("easy solving", |b| {
        b.iter(|| benchmark_specific_difficulty::<10, 10>(black_box(10)))
    });
    c.bench_function("intermediate solving", |b| {
        b.iter(|| benchmark_specific_difficulty::<16, 16>(black_box(40)))
    });
    c.bench_function("expert solving", |b| {
        b.iter(|| benchmark_specific_difficulty::<30, 16>(black_box(99)))
    });

    c.bench_function("easy generating", |b| {
        b.iter(|| Minefield::<10, 10>::generate(black_box(10)))
    });
    c.bench_function("intermediate generating", |b| {
        b.iter(|| Minefield::<16, 16>::generate(black_box(40)))
    });
    c.bench_function("expert generating", |b| {
        b.iter(|| Minefield::<30, 16>::generate(black_box(99)))
    });

    c.bench_function("easy revealing", |b| {
        b.iter(|| {
            Minefield::<10, 10>::generate(black_box(10))
                .unwrap()
                .reveal(Coord::random())
        })
    });
    c.bench_function("intermediate revealing", |b| {
        b.iter(|| {
            Minefield::<16, 16>::generate(black_box(40))
                .unwrap()
                .reveal(Coord::random())
        })
    });
    c.bench_function("expert revealing", |b| {
        b.iter(|| {
            Minefield::<30, 16>::generate(black_box(99))
                .unwrap()
                .reveal(Coord::random())
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
