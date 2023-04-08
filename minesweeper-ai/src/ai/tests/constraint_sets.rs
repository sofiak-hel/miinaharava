use std::{collections::HashSet, hint::black_box};

use arrayvec::ArrayVec;
use miinaharava::minefield::{Coord, Matrix};
use rand::seq::SliceRandom;

use crate::ai::{
    constraint_sets::ConstraintSet, constraints::Constraint, coord_set::CoordSet, CellContent,
    Decision,
};

use super::into_constraint_vec;

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

        let mut set = CoordSet::default();
        set.insert_many(initial.iter().flat_map(|c| c.variables.clone()));
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
        let (mut constraint_set, mine_coords) = generate_valid_constraints(50, 50, true);
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
fn test_trivial_solver_on_trivial() {
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
            let mut set = CoordSet::default();
            set.insert_many(variables.iter().cloned());
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
fn test_trivial_solver_on_nontrivial() {
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
        let _ = set.insert(
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
fn test_trivial_solver_with_known_variables() {
    for _ in 0..1000 {
        // Generate non-trivial valid constraints
        let (mut set, mine_coords) = generate_valid_constraints(30, 50, false);
        dbg!(&set);

        let mut known = Matrix([[CellContent::Unknown; 10]; 10]);
        // 1. Reveal about 30% of the field as known to the function
        let mut revealed = HashSet::new();
        for y in 0..10 {
            for x in 0..10 {
                // about 30% chance
                if rand::random::<u8>() > 176 {
                    let coord = Coord(x, y);
                    known.set(coord, CellContent::Known(mine_coords.contains(&coord)));
                }
            }
        }

        // Generate an expected set of decisions to test later
        let mut expected = Vec::new();
        {
            let mut set_clone = set.clone();

            let mut hidden_known = known;
            let mut prev_revealed = revealed.len();

            while {
                let mut c_idx = 0;
                while let Some(constraint) = set_clone.constraints.get_mut(c_idx) {
                    let mut idx = 0;
                    while let Some(var) = constraint.variables.get(idx) {
                        match hidden_known.get(*var) {
                            CellContent::Known(val) => {
                                constraint.label -= val as u8;
                                constraint.variables.remove(idx);
                            }
                            _ => idx += 1,
                        }
                    }
                    if constraint.label == 0 || constraint.label == constraint.len() as u8 {
                        for var in &constraint.variables {
                            hidden_known.set(*var, CellContent::Known(constraint.label > 0));
                        }
                        revealed.extend(constraint.variables.iter());
                        set_clone.constraints.remove(c_idx);
                    } else {
                        c_idx += 1;
                    }
                }

                revealed.len() > prev_revealed
            } {
                prev_revealed = revealed.len();
            }

            for coord in &revealed {
                if set.variables.contains(*coord) {
                    match hidden_known.get(*coord) {
                        CellContent::Known(true) => expected.push(Decision::Flag(*coord)),
                        CellContent::Known(false) => expected.push(Decision::Reveal(*coord)),
                        CellContent::Unknown => {}
                    }
                }
            }
        }

        // Actually solve the trivial cases
        let mut decisions = set.solve_trivial_cases(&mut known);
        decisions.sort();
        decisions.dedup();

        // 2. Also make sure all constraints are still valid
        for constraint in &set.constraints {
            let true_value = constraint
                .variables
                .iter()
                .filter(|v| mine_coords.contains(v))
                .count();
            assert_eq!(true_value as u8, constraint.label);
        }

        // 3. Make sure no constraints are still trivial
        for constraint in &set.constraints {
            assert_ne!(constraint.label, constraint.len() as u8);
            assert_ne!(constraint.len(), 0);
        }

        // 4. Make sure all decided fields are actually removed from variables,
        //    and that they are now known
        for decision in &decisions {
            match decision {
                Decision::Reveal(c) | Decision::Flag(c) => {
                    let true_variables: Vec<Coord<10, 10>> = set
                        .constraints
                        .iter()
                        .flat_map(|c| c.variables.clone())
                        .collect();
                    assert!(!set.variables.contains(*c));
                    assert!(!true_variables.contains(c));
                    assert_eq!(known.get(*c), CellContent::Known(mine_coords.contains(c)));
                }
            }
        }

        // 5. Make sure the returned decisions are exactly what was expected
        expected.sort();
        expected.dedup();
        assert_eq!(decisions, expected);

        // 6. Make sure clearing known variables is idempotent
        let old_set = set.clone();
        let _ = set.solve_trivial_cases(&mut known);
        assert_eq!(old_set, set);
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
