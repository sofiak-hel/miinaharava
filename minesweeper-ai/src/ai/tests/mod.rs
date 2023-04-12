use std::hint::black_box;

use arrayvec::ArrayVec;
use miinaharava::minefield::{Coord, GameState, Matrix, Minefield};
use rand::{seq::SliceRandom, Rng};

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
    let mut rng = rand::thread_rng();

    // Generate a random set of mines, keep track of mine-coords and
    // non-mine-coords separately
    let mine_amount = black_box(rand::random::<u8>()) % (mine_cap - 10) + 9;
    let mine_coords = {
        let mut mine_coords = Vec::with_capacity(mine_amount as usize);
        for _ in 0..mine_amount {
            let mut coord = Coord::random();
            while mine_coords.contains(&coord) {
                coord = Coord::random();
            }
            mine_coords.push(coord);
        }
        mine_coords
    };

    // Generate a random amount of constraints
    let constraint_amount = black_box(rand::random::<u8>() % constraint_count + 10) as usize;

    let mut constraints = Vec::with_capacity(constraint_amount);
    let mut available_coords = Vec::new();
    for y in 0..10 {
        for x in 0..10 {
            available_coords.push(Coord::<10, 10>(x as u8, y as u8));
        }
    }
    while constraints.len() < constraint_amount {
        let constraint_coord = *available_coords.choose(&mut rng).unwrap();
        available_coords.retain(|c| *c != constraint_coord);

        let neighbors = constraint_coord.neighbours();
        let mut mine_neighbors: Vec<_> = neighbors
            .iter()
            .filter(|n| mine_coords.contains(n))
            .collect();
        let mut non_mine_neighbors: Vec<_> = neighbors
            .iter()
            .filter(|n| !mine_coords.contains(n))
            .collect();

        if !allow_trivial && (mine_neighbors.is_empty() || non_mine_neighbors.is_empty()) {
            continue;
        }

        let max_mines = mine_neighbors.len() as u8;
        let min_mines = (non_mine_neighbors.is_empty() || !allow_trivial) as u8;

        let mine_count = black_box(rng.gen_range(min_mines..=max_mines));

        let max_vars = non_mine_neighbors.len() as u8 + mine_count;
        let min_vars = 1 + (!allow_trivial as u8) * (mine_count);
        assert!(max_vars >= min_vars);

        let variable_count = black_box(rng.gen_range(min_vars..=max_vars));

        let mut variables = Vec::with_capacity(variable_count as usize);
        for i in 0..variable_count {
            let var = if i < mine_count {
                mine_neighbors.remove(black_box(rand::random::<usize>() % mine_neighbors.len()))
            } else {
                non_mine_neighbors.remove(black_box(
                    rand::random::<usize>() % non_mine_neighbors.len(),
                ))
            };
            variables.push(*var);
        }

        let variables = ArrayVec::try_from(
            &*neighbors
                .choose_multiple(&mut rng, variable_count as usize)
                .cloned()
                .collect::<Vec<Coord<10, 10>>>(),
        )
        .unwrap();
        let label = variables.iter().filter(|v| mine_coords.contains(v)).count() as u8;

        constraints.push(Constraint { label, variables });
    }

    // Form the set from the constraints
    let mut set = CoordSet::default();
    set.insert_many(constraints.iter().flat_map(|c| c.variables.clone()));
    let constraint_set = ConstraintSet {
        constraints,
        variables: set,
    };
    (constraint_set, mine_coords.into_iter().collect())
}
