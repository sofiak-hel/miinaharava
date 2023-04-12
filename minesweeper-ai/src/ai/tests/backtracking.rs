use super::generate_valid_constraints;

#[test]
fn test_both_finds_at_least_correct_solution() {
    // Generate non-trivial valid constraints
    let (mut set, mine_coords) = generate_valid_constraints(30, 50, false);
}
