use std::hint::black_box;

use arrayvec::ArrayVec;
use miinaharava::minefield::{Coord, GameState, Matrix, Minefield};
use rand::seq::SliceRandom;

use crate::ai::{
    constraint_sets::ConstraintSet, constraints::Constraint, coord_set::CoordSet, guess, CSPState,
    CellContent, Decision,
};

mod backtracking;
mod constraint_sets;

pub const TRIVIAL_MINES: Matrix<bool, 7, 7> = Matrix([
    [false, false, false, false, false, false, false],
    [false, false, false, false, false, false, false],
    [false, false, false, false, false, true, false],
    [true, false, false, true, false, false, false],
    [false, false, false, false, false, false, false],
    [false, false, false, false, false, false, true],
    [false, true, false, true, false, false, false],
]);

#[test]
fn solve_trivial_field() {
    let mut minefield = Minefield::<7, 7>::with_mines(TRIVIAL_MINES);
    let mut state = CSPState::<7, 7>::default();

    let mut reveals = minefield.reveal(Coord(0, 0)).unwrap();

    let mut max_decisions = 20;
    while minefield.game_state() == GameState::Pending && max_decisions > 0 {
        let decisions = state.ponder(reveals.drain(..).collect(), &minefield);
        for decision in decisions {
            if let Some(res) = match decision {
                Decision::Flag(coord) => minefield.flag(coord).ok(),
                Decision::Reveal(coord) => minefield.reveal(coord).ok(),
            } {
                reveals.extend(res);
            }
        }
        max_decisions -= 1;
    }
    assert_eq!(minefield.game_state(), GameState::Victory)
}

#[test]
fn test_csp_insert() {
    for _ in 0..50 {
        // multiplier is 0 = should reveal all constraints
        // 1 = should flag all constraints
        for multiplier in 0..=1 {
            let mut state = CSPState::<10, 10>::default();
            let mut known = Matrix([[CellContent::Unknown; 10]; 10]);

            let amount = black_box(rand::random::<u8>() % 9);
            let vec = vec![Coord::<10, 10>(0, 0); amount as usize];
            let mut variables = ArrayVec::try_from(&*vec).unwrap();
            variables.fill_with(Coord::random);

            dbg!(&variables);

            let mut decisions = Vec::new();
            if let Some(res) = state.constraint_sets.insert(
                Constraint {
                    label: black_box(amount * multiplier),
                    variables: variables.clone(),
                },
                &mut known,
            ) {
                decisions.extend(res);
            }
            dbg!(&state);
            decisions.sort();
            decisions.dedup();
            let mut expected = variables
                .iter()
                .map(|v| match multiplier {
                    1 => Decision::Flag(*v),
                    _ => Decision::Reveal(*v),
                })
                .collect::<Vec<_>>();
            expected.sort();
            expected.dedup();
            assert_eq!(decisions, expected);
        }
    }
}

#[test]
fn test_csp_reveals() {
    let mut minefield = Minefield::<7, 7>::with_mines(TRIVIAL_MINES);
    let mut state = CSPState::default();
    let reveals = minefield.reveal(Coord(0, 0)).unwrap();
    state.handle_reveals(reveals, &minefield);

    let mut expected = into_constraint_vec(&[
        (1, &[Coord(0, 3), Coord(1, 3)]),
        (1, &[Coord(1, 3), Coord(3, 3)]),
        (1, &[Coord(3, 3), Coord(4, 3)]),
    ]);

    expected.sort();

    let mut set = CoordSet::default();
    set.insert_many(expected.iter().flat_map(|c| c.variables.clone()));
    let expected_set = ConstraintSet {
        constraints: expected,
        variables: set,
    };

    dbg!(&expected_set.constraints);
    dbg!(&state.constraint_sets);

    assert_eq!(*state.constraint_sets.0.get(0).unwrap(), expected_set);
}

#[test]
fn guess_is_a_corner() {
    for _ in 0..100 {
        let minefield = Minefield::<10, 10>::generate(10).unwrap();
        let decision = guess(&minefield);
        assert!(matches!(decision, Decision::Reveal(Coord(0 | 9, 0 | 9))));
    }
}

fn into_constraint_vec(array: &[(u8, &[Coord<7, 7>])]) -> Vec<Constraint<7, 7>> {
    array
        .iter()
        .map(|i| Constraint {
            label: i.0,
            variables: ArrayVec::try_from(i.1).unwrap(),
        })
        .collect()
}

fn into_constraint(label: u8, coords: &[Coord<7, 7>]) -> Constraint<7, 7> {
    Constraint {
        label,
        variables: ArrayVec::try_from(coords).unwrap(),
    }
}

fn generate_valid_constraints(
    mine_cap: u8,
    constraint_count: u8,
    allow_trivial: bool,
) -> (ConstraintSet<10, 10>, Vec<Coord<10, 10>>) {
    let mut rnd = rand::thread_rng();

    // Generate a random set of mines
    let mine_amount = black_box(rand::random::<u8>()) % (mine_cap - 10) + 9;
    let mut mine_coords = Vec::with_capacity(mine_amount as usize);
    for _ in 0..mine_amount {
        let mut coord = Coord::random();
        while mine_coords.contains(&coord) {
            coord = Coord::random();
        }
        mine_coords.push(coord);
    }
    let mut non_mine_coords = Vec::new();
    for y in 0..10 {
        for x in 0..10 {
            let coord = Coord(x, y);
            if !mine_coords.contains(&coord) {
                non_mine_coords.push(coord);
            }
        }
    }

    // Generate a random amount of constraints
    let amount = black_box(rand::random::<u8>() % constraint_count + 10);
    let mut vec = vec![Constraint::<10, 10>::default(); amount as usize];
    vec.fill_with(|| {
        let amount = black_box(rand::random::<u8>() % 7 + 2);
        let mut vec = Vec::with_capacity(amount as usize);

        let max_mine_amount = amount + allow_trivial as u8;
        let min_mine_amount = (!allow_trivial) as u8;

        let mine_amount =
            black_box(rand::random::<u8>() % (max_mine_amount - min_mine_amount) + min_mine_amount);
        for i in 0..amount {
            let mut coord = if i < mine_amount {
                *mine_coords.choose(&mut rnd).unwrap()
            } else {
                *non_mine_coords.choose(&mut rnd).unwrap()
            };
            while vec.contains(&coord) {
                coord = if i < mine_amount {
                    *mine_coords.choose(&mut rnd).unwrap()
                } else {
                    *non_mine_coords.choose(&mut rnd).unwrap()
                };
            }
            vec.push(coord);
        }

        vec.sort();
        vec.dedup();
        let variables = ArrayVec::try_from(&*vec).unwrap();
        Constraint {
            // Make sure label is always correct
            label: variables
                .iter()
                .filter(|v| mine_coords.contains(*v))
                .count() as u8,
            variables,
        }
    });
    // Actually add create the constraint set
    let mut set = CoordSet::default();
    set.insert_many(vec.iter().flat_map(|c| c.variables.clone()));
    let constraint_set = ConstraintSet {
        constraints: vec,
        variables: set,
    };
    (constraint_set, mine_coords.into_iter().collect())
}
