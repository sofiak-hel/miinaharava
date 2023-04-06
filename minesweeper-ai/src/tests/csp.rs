use std::{collections::HashSet, hint::black_box};

use arrayvec::ArrayVec;
use miinaharava::minefield::{Coord, GameState, Matrix, Minefield};

use crate::{
    ai::{ponder, Decision},
    csp::{ConstaintSatisficationState, Constraint, ConstraintSet},
};

const TRIVIAL_MINES: Matrix<bool, 7, 7> = Matrix([
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

    minefield.reveal(Coord(0, 0)).unwrap();

    let mut max_decisions = 20;
    while minefield.game_state() == GameState::Pending && max_decisions > 0 {
        let decisions = ponder(&minefield);
        for decision in decisions {
            match decision {
                Decision::Flag(coord) => minefield.flag(coord),
                Decision::Reveal(coord) => minefield.reveal(coord),
            }
            .ok();
        }
        max_decisions -= 1;
    }
    assert_eq!(minefield.game_state(), GameState::Victory)
}

#[test]
fn test_trivial_constraints() {
    for _ in 0..50 {
        // multiplier is 0 = should reveal all constraints
        // 1 = should flag all constraints
        for multiplier in 0..=1 {
            let mut state = ConstaintSatisficationState::<10, 10>::default();
            let amount = black_box(rand::random::<u8>() % 9);
            let vec = vec![Coord::<10, 10>(0, 0); amount as usize];
            let mut variables = ArrayVec::try_from(&*vec).unwrap();
            variables.fill_with(Coord::random);

            state.constraints.insert(Constraint {
                label: black_box(amount * multiplier),
                variables: variables.clone(),
            });
            dbg!(&state);
            let mut decisions = state.solve_trivial_cases().unwrap();
            decisions.sort();
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
fn test_constraint_generation() {
    let mut minefield = Minefield::<7, 7>::with_mines(TRIVIAL_MINES);
    minefield.reveal(Coord(0, 0)).unwrap();

    let state = ConstaintSatisficationState::from(&minefield);
    let expected = into_constraint_vec(&[
        (1, &[Coord(0, 3), Coord(1, 3)]),
        (1, &[Coord(0, 3), Coord(1, 3), Coord(2, 3)]),
        (1, &[Coord(0, 3), Coord(1, 3)]),
        (1, &[Coord(0, 3), Coord(1, 3), Coord(2, 3)]),
        (1, &[Coord(1, 3), Coord(2, 3), Coord(3, 3)]),
        (1, &[Coord(2, 3), Coord(3, 3), Coord(4, 3)]),
        (2, &[Coord(3, 3), Coord(4, 3), Coord(5, 3), Coord(5, 2)]),
        (1, &[Coord(5, 2)]),
        (1, &[Coord(5, 2), Coord(6, 2)]),
        (1, &[Coord(5, 2), Coord(6, 2)]),
    ]);

    let mut set = HashSet::new();
    set.extend(expected.iter().flat_map(|c| c.variables.clone()));
    let mut expected_set = ConstraintSet {
        constraints: expected,
        variables: set,
    };
    expected_set.reduce();

    assert_eq!(*state.constraints.0.get(0).unwrap(), expected_set);
}

#[test]
fn test_known_reduces() {
    let known = vec![
        (
            into_constraint_vec(&[
                (1, &[Coord(0, 3), Coord(1, 3)]),
                (1, &[Coord(0, 3), Coord(1, 3), Coord(2, 3)]),
                (1, &[Coord(0, 3), Coord(1, 3)]),
                (1, &[Coord(0, 3), Coord(1, 3), Coord(2, 3)]),
                (1, &[Coord(1, 3), Coord(2, 3), Coord(3, 3)]),
                (1, &[Coord(2, 3), Coord(3, 3), Coord(4, 3)]),
                (2, &[Coord(3, 3), Coord(4, 3), Coord(5, 3), Coord(5, 2)]),
                (1, &[Coord(5, 2)]),
                (1, &[Coord(5, 2), Coord(6, 2)]),
                (2, &[Coord(5, 2), Coord(6, 2), Coord(7, 2)]),
            ]),
            into_constraint_vec(&[
                (1, &[Coord(0, 3), Coord(1, 3)]),
                (0, &[Coord(2, 3)]),
                (1, &[Coord(1, 3), Coord(3, 3)]),
                (1, &[Coord(3, 3), Coord(4, 3)]),
                (0, &[Coord(5, 3)]),
                (1, &[Coord(5, 2)]),
                (0, &[Coord(6, 2)]),
                (1, &[Coord(7, 2)]),
            ]),
        ),
        (
            into_constraint_vec(&[
                (1, &[Coord(5, 2)]),
                (1, &[Coord(5, 2), Coord(6, 2)]),
                (2, &[Coord(5, 2), Coord(6, 2), Coord(7, 2)]),
            ]),
            into_constraint_vec(&[
                (1, &[Coord(5, 2)]),
                (0, &[Coord(6, 2)]),
                (1, &[Coord(7, 2)]),
            ]),
        ),
        (
            into_constraint_vec(&[
                (1, &[]),
                (1, &[Coord(5, 2), Coord(6, 2)]),
                (2, &[Coord(5, 2), Coord(6, 2), Coord(7, 2)]),
            ]),
            into_constraint_vec(&[
                (1, &[]),
                (1, &[Coord(5, 2), Coord(6, 2)]),
                (1, &[Coord(7, 2)]),
            ]),
        ),
    ];

    for known_reduces in known {
        let initial = known_reduces.0;
        let mut expected = known_reduces.1;

        expected.sort();

        let mut set = HashSet::new();
        set.extend(initial.iter().flat_map(|c| c.variables.clone()));
        let mut initial_set = ConstraintSet {
            constraints: initial,
            variables: set,
        };
        initial_set.reduce();
        let mut initial_reduced = initial_set.constraints;
        initial_reduced.sort();

        assert_eq!(initial_reduced, expected);
    }
}

#[test]
fn test_random_reduces() {
    // Do all of this 100 times for good measure (random is hard)
    for _ in 0..200 {
        // 1. First generate some random amount of valid constraints
        // (valid, as in the labels in the set always correspond correctly)
        let mine_amount = black_box(rand::random::<u8>()) % 50;
        let mut mine_coords = vec![Coord::<10, 10>(0, 0); mine_amount as usize];
        mine_coords.fill_with(Coord::random);

        let amount = black_box(rand::random::<u8>() % 70 + 20);
        let mut vec = vec![Constraint::<10, 10>::default(); amount as usize];
        vec.fill_with(|| {
            let amount = black_box(rand::random::<u8>() % 8 + 1);
            let vec = vec![Coord::<10, 10>(0, 0); amount as usize];
            let mut variables = ArrayVec::try_from(&*vec).unwrap();
            variables.fill_with(Coord::random);
            Constraint {
                label: variables
                    .iter()
                    .filter(|v| mine_coords.contains(*v))
                    .count() as u8,
                variables,
            }
        });
        dbg!(&vec);

        // 2. Generate a ConstraintSet from them and reduce the set
        let mut set = HashSet::new();
        set.extend(vec.iter().flat_map(|c| c.variables.clone()));
        let mut constraint_set = ConstraintSet {
            constraints: vec,
            variables: set,
        };
        constraint_set.reduce();

        // 3. Make sure no two constraints can be further reduced
        for (i, c1) in constraint_set.constraints.iter().enumerate() {
            for (j, c2) in constraint_set.constraints.iter().enumerate() {
                if i != j {
                    assert!(!c2.is_superset_of(c1) && !c1.is_superset_of(c2))
                }
            }

            // Also make sure c1 is still a valid constraint
            let true_value = c1
                .variables
                .iter()
                .filter(|v| mine_coords.contains(v))
                .count();
            assert_eq!(true_value as u8, c1.label);
        }
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
