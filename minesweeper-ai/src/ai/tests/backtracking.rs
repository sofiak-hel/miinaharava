use std::time::Instant;

use bitvec::vec::BitVec;
use miinaharava::minefield::Matrix;

use crate::ai::CellContent;

use super::generate_valid_constraints;

#[test]
fn test_both_finds_at_least_correct_solution() {
    for _ in 0..200 {
        // Generate non-trivial valid constraints
        let (set, mine_coords) = generate_valid_constraints(20, 20, false);
        let known = Matrix([[CellContent::Unknown; 10]; 10]);
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
    }
}
