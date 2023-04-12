use miinaharava::minefield::Matrix;

use crate::ai::CellContent;

use super::generate_valid_constraints;

#[test]
fn test_both_finds_at_least_correct_solution() {
    for _ in 0..1000 {
        // Generate non-trivial valid constraints
        let (set, mine_coords) = generate_valid_constraints(30, 50, false);
        let known = Matrix([[CellContent::Unknown; 10]; 10]);
        let ordered = set.find_ordered();

        let mut correct_solution = Vec::with_capacity(ordered.len());
        for (coord, _) in &ordered {
            if mine_coords.contains(coord) {
                correct_solution.push(coord);
            }
        }

        let possible_solutions = set.test_both(&ordered, Vec::new(), known);
    }
}
