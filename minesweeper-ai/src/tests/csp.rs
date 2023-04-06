use std::{collections::HashSet, hint::black_box};

use arrayvec::ArrayVec;
use miinaharava::minefield::{Coord, GameState, Matrix, Minefield};

use crate::{
    ai::Decision,
    csp::{CellContent, Constraint, ConstraintSatisficationState, ConstraintSet},
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

// #[test]
// fn solve_trivial_field() {
//     let mut minefield = Minefield::<7, 7>::with_mines(TRIVIAL_MINES);

//     minefield.reveal(Coord(0, 0)).unwrap();

//     let mut max_decisions = 20;
//     while minefield.game_state() == GameState::Pending && max_decisions > 0 {
//         let decisions = ponder(&minefield);
//         for decision in decisions {
//             match decision {
//                 Decision::Flag(coord) => minefield.flag(coord),
//                 Decision::Reveal(coord) => minefield.reveal(coord),
//             }
//             .ok();
//         }
//         max_decisions -= 1;
//     }
//     assert_eq!(minefield.game_state(), GameState::Victory)
// }

// #[test]
// fn test_trivial_constraints() {
//     for _ in 0..50 {
//         // multiplier is 0 = should reveal all constraints
//         // 1 = should flag all constraints
//         for multiplier in 0..=1 {
//             let mut state = ConstaintSatisficationState::<10, 10>::default();
//             let amount = black_box(rand::random::<u8>() % 9);
//             let vec = vec![Coord::<10, 10>(0, 0); amount as usize];
//             let mut variables = ArrayVec::try_from(&*vec).unwrap();
//             variables.fill_with(Coord::random);

//             state.constraint_sets.insert(Constraint {
//                 label: black_box(amount * multiplier),
//                 variables: variables.clone(),
//             });
//             dbg!(&state);
//             let mut decisions = state.solve_trivial_cases().unwrap();
//             decisions.sort();
//             let mut expected = variables
//                 .iter()
//                 .map(|v| match multiplier {
//                     1 => Decision::Flag(*v),
//                     _ => Decision::Reveal(*v),
//                 })
//                 .collect::<Vec<_>>();
//             expected.sort();
//             expected.dedup();
//             assert_eq!(decisions, expected);
//         }
//     }
// }

#[test]
fn test_constraint_generation() {
    let mut minefield = Minefield::<7, 7>::with_mines(TRIVIAL_MINES);
    let mut state = ConstraintSatisficationState::default();
    let reveals = minefield.reveal(Coord(0, 0)).unwrap();
    state.handle_reveals(reveals, &minefield);

    let mut expected = into_constraint_vec(&[
        (1, &[Coord(0, 3), Coord(1, 3)]),
        (1, &[Coord(1, 3), Coord(3, 3)]),
        (1, &[Coord(3, 3), Coord(4, 3)]),
    ]);

    expected.sort();

    let mut set = HashSet::new();
    set.extend(expected.iter().flat_map(|c| c.variables.clone()));
    let expected_set = ConstraintSet {
        constraints: expected,
        variables: set,
    };

    dbg!(&expected_set.constraints);
    dbg!(&state.constraint_sets);

    assert_eq!(*state.constraint_sets.0.get(0).unwrap(), expected_set);
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
        let (mut constraint_set, mine_coords) = generate_valid_constraints(50);
        dbg!(&constraint_set);

        constraint_set.reduce();

        // 3. Make sure no two constraints can be further reduced
        for (i, c1) in constraint_set.constraints.iter().enumerate() {
            for (j, c2) in constraint_set.constraints.iter().enumerate() {
                if i != j {
                    assert!(!c2.is_superset_of(c1) && !c1.is_superset_of(c2))
                }
            }

            // 4. Also make sure c1 is still a valid constraint
            let true_value = c1
                .variables
                .iter()
                .filter(|v| mine_coords.contains(v))
                .count();
            assert_eq!(true_value as u8, c1.label);
        }

        // 5. Make sure reduce is idempotent.
        let clone = constraint_set.clone();
        constraint_set.reduce();
        assert_eq!(clone, constraint_set);
    }
}

#[test]
fn test_trivial_cases() {
    for _ in 0..50 {
        for multiplier in 0..=1 {
            let mut known = Matrix([[CellContent::Unknown; 10]; 10]);
            // 1. Generate some random variables
            let amount = black_box(rand::random::<u8>() % 9);
            let vec = vec![Coord::<10, 10>(0, 0); amount as usize];
            let mut variables = ArrayVec::try_from(&*vec).unwrap();
            variables.fill_with(Coord::random);

            // 2. Insert variables into the constrait, calculate label
            // multiplier = 0 = all of them are empty
            // multiplier = 1 = all of them are mines
            let mut set = HashSet::new();
            set.extend(variables.iter());
            let mut constraint_set = ConstraintSet {
                constraints: vec![Constraint {
                    label: black_box(amount * multiplier),
                    variables: variables.clone(),
                }],
                variables: set,
            };
            dbg!(&constraint_set);

            // 3. Make sure returned decisions are as expected
            let mut expected = variables
                .iter()
                .map(|v| match multiplier {
                    1 => Decision::Flag(*v),
                    _ => Decision::Reveal(*v),
                })
                .collect::<Vec<_>>();
            expected.sort();
            expected.dedup();
            let mut decisions = constraint_set.solve_trivial_cases(&mut known);
            decisions.sort();
            decisions.dedup();
            assert_eq!(decisions, expected);

            // 4. Make sure the correct cells got marked as known, and no other
            //    cells were touched.
            for (y, row) in known.iter().enumerate() {
                for (x, cell) in row.iter().enumerate() {
                    if variables.contains(&Coord(x as u8, y as u8)) {
                        assert_eq!(*cell, CellContent::Known(multiplier == 1));
                    } else {
                        assert_eq!(*cell, CellContent::Unknown)
                    }
                }
            }

            // 5. Make sure all constraints were processed and removed, they
            //    were trivial.
            assert_eq!(constraint_set.constraints.len(), 0);
        }
    }
}

#[test]
fn test_trivial_on_nontrivial() {
    for _ in 0..100 {
        let mut known = Matrix([[CellContent::Unknown; 10]; 10]);
        let mut set = ConstraintSet::default();

        // 1. Generate some random variables
        let amount = black_box(rand::random::<u8>() % 9);
        let vec = vec![Coord::<10, 10>(0, 0); amount as usize];
        let mut variables = ArrayVec::try_from(&*vec).unwrap();
        variables.fill_with(Coord::random);

        // 2. Generate constraints that always have a different label than the
        // number of variables thus making them nontrivial
        set.insert(
            Constraint {
                label: black_box((rand::random::<u8>() % 100 + 9) ^ amount),
                variables: variables.clone(),
            },
            &mut known,
        );

        // 3. Make sure trivial_solver does nothing with these constraints
        let old_length = set.constraints.len();
        dbg!(&set);
        let decisions = set.solve_trivial_cases(&mut known);

        assert!(decisions.is_empty());
        assert_eq!(known, Matrix([[CellContent::Unknown; 10]; 10]));
        assert_eq!(set.constraints.len(), old_length);

        // 4. Make sure trivial solver is idempotent
        let decisions = set.solve_trivial_cases(&mut known);

        assert!(decisions.is_empty());
        assert_eq!(known, Matrix([[CellContent::Unknown; 10]; 10]));
        assert_eq!(set.constraints.len(), old_length);
    }
}

#[test]
fn test_clearing_known_variables() {
    for _ in 0..1000 {
        let (mut set, mine_coords) = generate_valid_constraints(50);
        dbg!(&set);

        let mut known = Matrix([[CellContent::Unknown; 10]; 10]);
        // 1. Reveal about 30% of the field as known to the function
        let mut revealed = Vec::new();
        for y in 0..10 {
            for x in 0..10 {
                // about 30% chance
                if rand::random::<u8>() > 176 {
                    let coord = Coord(x, y);
                    known.set(coord, CellContent::Known(mine_coords.contains(&coord)));
                    revealed.push(coord);
                }
            }
        }

        // 2. Make sure the returned decisions are as expected
        let mut expected = Vec::new();
        for coord in &revealed {
            if set.variables.contains(coord) {
                match known.get(*coord) {
                    CellContent::Known(true) => expected.push(Decision::Flag(*coord)),
                    CellContent::Known(false) => expected.push(Decision::Reveal(*coord)),
                    CellContent::Unknown => {}
                }
            }
        }
        expected.sort();
        expected.dedup();

        let decisions = set.clear_known_variables(&known);
        assert_eq!(decisions, expected);

        // 3. Make sure all revealed fields are actually removed
        for coord in &revealed {
            if let CellContent::Known(_) = known.get(*coord) {
                let true_variables: Vec<Coord<10, 10>> = set
                    .constraints
                    .iter()
                    .flat_map(|c| c.variables.clone())
                    .collect();
                assert!(!set.variables.contains(coord));
                assert!(!true_variables.contains(coord));
            }
        }

        // 4. Also make sure all constraints are still valid
        for constraint in &set.constraints {
            let true_value = constraint
                .variables
                .iter()
                .filter(|v| mine_coords.contains(v))
                .count();
            assert_eq!(true_value as u8, constraint.label);
        }

        // 5. Make sure clearing known variables is idempotent
        let old_set = set.clone();
        set.clear_known_variables(&known);
        assert_eq!(old_set, set);
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

fn generate_valid_constraints(mine_cap: u8) -> (ConstraintSet<10, 10>, Vec<Coord<10, 10>>) {
    // Generate a random set of mines
    let mine_amount = black_box(rand::random::<u8>()) % mine_cap;
    let mut mine_coords = vec![Coord::<10, 10>(0, 0); mine_amount as usize];
    mine_coords.fill_with(Coord::random);

    // Generate a random amount of constraints
    let amount = black_box(rand::random::<u8>() % 70 + 20);
    let mut vec = vec![Constraint::<10, 10>::default(); amount as usize];
    vec.fill_with(|| {
        let amount = black_box(rand::random::<u8>() % 8 + 1);
        let vec = vec![Coord::<10, 10>(0, 0); amount as usize];
        let mut variables = ArrayVec::try_from(&*vec).unwrap();
        variables.fill_with(Coord::random);
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
    let mut set = HashSet::new();
    set.extend(vec.iter().flat_map(|c| c.variables.clone()));
    let constraint_set = ConstraintSet {
        constraints: vec,
        variables: set,
    };
    (constraint_set, mine_coords)
}
