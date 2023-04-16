use std::collections::HashSet;

use bitvec::prelude::*;
use miinaharava::minefield::{Coord, Matrix};
use rand::Rng;

use crate::ai::{
    constraint_sets::{ConstraintSet, CoupledSets},
    constraints::Constraint,
    CellContent,
};

mod solutions;

use super::generate_valid_constraints;

/// Ensure that all variables from all constraints are in the ordered list
/// correctly.
#[test]
fn test_find_ordered() {
    for _ in 0..5000 {
        let (set, _) = generate_valid_constraints(20, 20, false);
        let ordered = set.find_ordered();

        // 1. Ensure all constraints are listed for all the variables that should
        // list them
        for (i, constraint) in set.constraints.iter().enumerate() {
            for var in &constraint.variables {
                let item = ordered.iter().find(|(c, _)| c == var).unwrap();
                assert!(item.1.contains(&i));
            }
        }

        // 2. Ensure no items contain constraints that don't contain the
        //    variable
        for (var, indexes) in &ordered {
            for idx in indexes {
                assert!(set.constraints[*idx].variables.contains(var));
            }
        }

        // 3. Ensure the list is actually sorted by the amount of items in
        //    constraint-index-list
        let mut curr_len = 500;
        for (_, list) in &ordered {
            assert!(curr_len >= list.len());
            curr_len = list.len();
        }
    }
}

// Make sure test_both only returns valid solutions, and that among said
// solutions is the correct solution
#[test]
fn find_solutions_finds_only_valid_solutions_and_correct_solutions() {
    for _ in 0..1000 {
        // Generate valid constraints
        let (set, mine_coords) = get_fast_valid_constraints();

        let known = Matrix([[CellContent::Unknown; 10]; 10]);
        // Only take the variables that are in more than 1 constraint, otherwise
        // tests are slowed down a LOT!
        let ordered = set.find_ordered();

        let mut correct_solution: BitVec = BitVec::new();
        dbg!(&correct_solution);
        for (coord, _) in &ordered {
            correct_solution.push(mine_coords.contains(coord));
        }

        let possible_solutions = set.find_solutions(&ordered, BitVec::new(), known);
        dbg!(&possible_solutions);

        // 1. Make sure correct solution is found
        assert!(possible_solutions.contains(&correct_solution));

        // 2. Make sure all found solutions are viable
        for solution in possible_solutions {
            let mut constraint_to_mine_count = vec![0u8; set.constraints.len()];
            for (i, is_mine) in solution.iter().enumerate() {
                for c in &ordered.get(i).unwrap().1 {
                    dbg!(&constraint_to_mine_count);
                    constraint_to_mine_count[*c] += *is_mine as u8;
                }
            }
            for (i, constraint) in constraint_to_mine_count.iter().enumerate() {
                dbg!(&set.constraints[i]);
                assert_eq!(set.constraints[i].label, *constraint);
            }
        }
    }
}

#[test]
fn test_constraint_sets_find_viable_solutions() {
    for _ in 0..5000 {
        // Generate non-trivial valid constraints
        let (set, mine_coords) = get_fast_valid_constraints();
        let known = Matrix([[CellContent::Unknown; 10]; 10]);

        let mut solution_list_map = set.find_viable_solutions(mine_coords.len() as u8, &known);

        // Ensure that no duplicate solutions exist
        for solution_list in solution_list_map.solutions_by_mines.iter() {
            let mut previous_solutions = HashSet::with_capacity(solution_list.len());
            for solution in solution_list {
                assert!(previous_solutions.insert(solution));
            }
        }
        dbg!(&solution_list_map);

        // Find minimum and maximum amount of mines in solutions
        let mut min_mines = 1000;
        let mut max_mines = 0;
        for solutions in solution_list_map.iter() {
            for solution in solutions {
                min_mines = min_mines.min(solution.iter_ones().count());
                max_mines = max_mines.max(solution.iter_ones().count());
            }
        }

        // Make sure there are no mines outside of minimum and maximum
        dbg!(min_mines, max_mines);
        for i in ((min_mines as i32 - 10).max(0) as usize)..(max_mines + 10) {
            dbg!(i);
            assert_eq!(
                solution_list_map.get(i as u8).is_some(),
                i >= min_mines && i <= max_mines
            );
            assert_eq!(
                solution_list_map.get_mut(i as u8).is_some(),
                i >= min_mines && i <= max_mines
            );
        }
    }
}

#[test]
fn test_coupled_set_find_viable_solutions() {
    let mut rng = rand::thread_rng();
    for _ in 0..5000 {
        // Generate non-trivial valid constraints
        let (set1, mines1) = get_fast_valid_constraints();
        let (set2, mines2) = get_fast_valid_constraints();
        let known = Matrix([[CellContent::Unknown; 10]; 10]);

        // Get the minimum count of mines for each set
        let mine_count1 = set1
            .find_viable_solutions(20, &known)
            .iter()
            .next()
            .unwrap()[0]
            .iter_ones()
            .count() as u8;
        let mine_count2 = set2
            .find_viable_solutions(20, &known)
            .iter()
            .next()
            .unwrap()[0]
            .iter_ones()
            .count() as u8;
        let min_mines = mine_count1 + mine_count2;

        // Get the amount of mines actually revealed through constraints
        let mut mine_coords = mines1.iter().chain(mines2.iter()).collect::<Vec<_>>();
        mine_coords.sort();
        mine_coords.dedup();
        let revealed_mines = set1
            .variables
            .iter()
            .chain(set2.variables.iter())
            .filter(|v| mine_coords.contains(&v))
            .count() as u8;

        let sets = CoupledSets(vec![set1, set2]);

        // Randomize an arbitrary limit for remaining mines and make sure there
        // are no solutions that have more mines than that
        let remaining_mines = rng.gen_range(min_mines..=revealed_mines) + 1;
        dbg!(remaining_mines);
        dbg!(min_mines);
        let mut solution_map_lists = sets.find_viable_solutions(remaining_mines, &known);
        for map_list in &mut solution_map_lists {
            for i in (remaining_mines + 1)..revealed_mines {
                dbg!(i);
                assert!(map_list.get(i).is_none());
                assert!(map_list.get_mut(i).is_none());
            }
        }
    }
}

/// Returns valid constraints, and removes most variables that are only in one
/// constraint. Having them included will slow down the process very much.
fn get_fast_valid_constraints() -> (ConstraintSet<10, 10>, Vec<Coord<10, 10>>) {
    let (mut set, mine_coords) = generate_valid_constraints(20, 20, true);

    // How many single variables are allowed, that exist in only one constraint
    let mut max_individual_vars = 10;

    for (exists, var) in set.variables.iter_mut() {
        let mut constraints: Vec<&mut Constraint<10, 10>> = set
            .constraints
            .iter_mut()
            .filter(|c| c.variables.contains(&var))
            .collect();
        if constraints.len() == 1 {
            if max_individual_vars == 0 {
                *exists = false;
                constraints[0].variables.retain(|v| *v != var);
                constraints[0].label -= mine_coords.contains(&var) as u8;
            } else {
                max_individual_vars -= 1;
            }
        }
    }
    (set, mine_coords)
}
