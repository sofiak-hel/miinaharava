use bitvec::vec::BitVec;
use miinaharava::minefield::Matrix;

use crate::ai::CellContent;

use super::generate_valid_constraints;

// TODOS:
// - Tests for find_ordered
// - Tests for find_viable_solutions
// - Rest of the algorithm that uses these viable solutions, from step
//   5., check for crapshoots first though

/// Ensure that all variables from all constraints are in the ordered list
/// correctly.
#[test]
fn test_find_ordered() {
    for _ in 0..1000 {
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
fn test_both_finds_only_valid_solutions_and_correct_solutions() {
    for _ in 0..200 {
        // Generate non-trivial valid constraints
        let (set, mine_coords) = generate_valid_constraints(20, 20, false);
        let known = Matrix([[CellContent::Unknown; 10]; 10]);
        // Only take the variables that are in more than 1 constraint, otherwise
        // tests are slowed down insanely much.
        let ordered = set
            .find_ordered()
            .into_iter()
            .filter(|(_, v)| v.len() > 1)
            .collect::<Vec<_>>();

        let mut correct_solution: BitVec = BitVec::new();
        for (coord, _) in &ordered {
            correct_solution.push(mine_coords.contains(coord));
        }

        let possible_solutions = set.test_both(&ordered, BitVec::new(), known);
        assert!(possible_solutions.contains(&correct_solution));

        for solution in possible_solutions {
            let mut constraint_to_mine_count = vec![0u8; set.constraints.len()];
            for (i, is_mine) in solution.iter().enumerate() {
                for c in &ordered.get(i).unwrap().1 {
                    dbg!(&constraint_to_mine_count);
                    constraint_to_mine_count[*c] += *is_mine as u8;
                }
            }
            for (i, constraint) in constraint_to_mine_count.iter().enumerate() {
                assert!(set.constraints[i].label >= *constraint);
            }
        }
    }
}
