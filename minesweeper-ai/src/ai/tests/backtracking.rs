use std::time::Instant;

use bitvec::vec::BitVec;
use miinaharava::minefield::Matrix;

use crate::ai::CellContent;

use super::generate_valid_constraints;

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
