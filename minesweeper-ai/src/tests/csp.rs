use std::hint::black_box;

use arrayvec::ArrayVec;
use miinaharava::minefield::{Coord, GameState, Minefield};

use crate::{
    ai::{ponder, Decision},
    csp::{ConstaintSatisficationState, Constraint},
};

const TRIVIAL_MINES: [[bool; 7]; 7] = [
    [false, false, false, false, false, false, false],
    [false, false, false, false, false, false, false],
    [false, false, false, false, false, true, false],
    [true, false, false, true, false, false, false],
    [false, false, false, false, false, false, false],
    [false, false, false, false, false, false, true],
    [false, true, false, true, false, false, false],
];

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
            .unwrap();
        }
        max_decisions -= 1;
    }
    assert_eq!(minefield.game_state(), GameState::Victory)
}

#[test]
fn first_ponder_is_a_corner() {
    for _ in 0..100 {
        let minefield = Minefield::<10, 10>::generate(10).unwrap();
        let decisions = ponder(&minefield);
        let decision = decisions.get(0).unwrap();
        assert!(matches!(decision, Decision::Reveal(Coord(0 | 9, 0 | 9))));
    }
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

            state.constraints.push(Constraint {
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
    let mut expected: Vec<Constraint<7, 7>> = vec![
        Constraint {
            label: 1,
            variables: ArrayVec::try_from(&[Coord(0, 3), Coord(1, 3)][..]).unwrap(),
        },
        Constraint {
            label: 1,
            variables: ArrayVec::try_from(&[Coord(0, 3), Coord(1, 3), Coord(2, 3)][..]).unwrap(),
        },
        Constraint {
            label: 1,
            variables: ArrayVec::try_from(&[Coord(1, 3), Coord(2, 3), Coord(3, 3)][..]).unwrap(),
        },
        Constraint {
            label: 1,
            variables: ArrayVec::try_from(&[Coord(2, 3), Coord(3, 3), Coord(4, 3)][..]).unwrap(),
        },
        Constraint {
            label: 2,
            variables: ArrayVec::try_from(
                &[Coord(3, 3), Coord(4, 3), Coord(5, 3), Coord(5, 2)][..],
            )
            .unwrap(),
        },
        Constraint {
            label: 1,
            variables: ArrayVec::try_from(&[Coord(5, 2)][..]).unwrap(),
        },
        Constraint {
            label: 1,
            variables: ArrayVec::try_from(&[Coord(5, 2), Coord(6, 2)][..]).unwrap(),
        },
        Constraint {
            label: 1,
            variables: ArrayVec::try_from(&[Coord(5, 2), Coord(6, 2)][..]).unwrap(),
        },
    ];
    let mut constraints = state.constraints;
    expected.sort();
    constraints.sort();

    assert_eq!(constraints, expected);
}
