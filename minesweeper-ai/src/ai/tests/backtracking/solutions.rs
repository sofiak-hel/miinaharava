use std::hint::black_box;

use bitvec::prelude::*;
use bitvec::vec::BitVec;
use miinaharava::minefield::{Coord, Matrix};
use rand::seq::SliceRandom;

use crate::ai::backtracking::solutions::{SolutionContainer, SolutionList};
use crate::ai::coord_set::CoordSet;
use crate::ai::tests::constraint_sets::*;
use crate::ai::{CellContent, Decision};

// Ensure that transposing works correctly
#[test]
fn test_transposed_solution_coord() {
    let mut rng = rand::thread_rng();

    for _ in 0..5000 {
        let solution_count = black_box(rand::random::<u8>() % 20 + 5);

        let coord_amount = black_box(rand::random::<u8>() % 20 + 5);
        let mut coord_set = CoordSet::<7, 7>::default();
        for _ in 0..coord_amount {
            coord_set.insert(Coord::random());
        }
        let coords = coord_set.iter().collect::<Vec<_>>();

        let mine_amount =
            black_box(rand::random::<u8>() % (coords.len() as u8) + 1).max(coords.len() as u8);

        let mut solutions = Vec::new();
        for _ in 0..solution_count {
            let mut solution: BitVec = BitVec::with_capacity(coords.len());
            for _ in 0..coords.len() {
                solution.push(false);
            }
            let mut idx_list = Vec::from_iter(0..coords.len());
            idx_list.shuffle(&mut rng);
            for _ in 0..mine_amount {
                solution.set(idx_list.pop().unwrap(), true);
            }
            solutions.push(solution);
        }

        let solution_list = SolutionList::from(solutions.clone(), coords.clone(), 100);

        assert_eq!(solution_list.get(mine_amount).unwrap(), &solutions);

        let mut transposed: Vec<(Coord<7, 7>, BitVec)> = Vec::new();
        for (idx, coord) in coords.iter().enumerate() {
            let mut bitvec = BitVec::with_capacity(solutions.len());
            for solution in &solutions {
                bitvec.push(*solution.get(idx).unwrap());
            }
            transposed.push((*coord, bitvec));
        }

        assert_eq!(
            solution_list
                .transposed_solution_coords()
                .collect::<Vec<_>>(),
            transposed
        );
    }

    // One more time with a known list
    let solution_list = SolutionList::from(
        vec![
            bitvec![1, 0, 1, 1, 0],
            bitvec![0, 1, 0, 1, 1],
            bitvec![0, 1, 1, 0, 1],
            bitvec![1, 1, 0, 0, 1],
            bitvec![0, 0, 1, 1, 1],
            bitvec![1, 1, 0, 0, 1],
        ],
        vec![A, B, C, D, E],
        10,
    );
    dbg!(&solution_list.solutions_by_mines);
    let list = solution_list
        .transposed_solution_coords()
        .collect::<Vec<_>>();
    assert_eq!(
        list,
        vec![
            (A, bitvec![1, 0, 0, 1, 0, 1]),
            (B, bitvec![0, 1, 1, 1, 0, 1]),
            (C, bitvec![1, 0, 1, 0, 1, 0]),
            (D, bitvec![1, 1, 0, 0, 1, 0]),
            (E, bitvec![0, 1, 1, 1, 1, 1])
        ]
    );
}

/// Ensure that all variables from all constraints are in the ordered list
/// correctly.
#[test]
fn test_solution_list_trivial_finder_with_nontrivial() {
    let solution_list = SolutionList::from(
        vec![
            bitvec![1, 0, 1, 1, 0],
            bitvec![0, 1, 0, 1, 1],
            bitvec![0, 1, 1, 0, 1],
            bitvec![1, 1, 0, 0, 1],
            bitvec![0, 0, 1, 0, 1],
            bitvec![1, 1, 0, 0, 1],
        ],
        vec![A, B, C, D, E],
        10,
    );
    let mut known = Matrix([[CellContent::Unknown; 7]; 7]);
    let decisions = solution_list.find_trivial_decisions(&mut known);
    assert!(decisions.is_empty());
}

/// Ensure that all variables from all constraints are in the ordered list
/// correctly.
#[test]
fn test_solution_list_trivial_finder_with_random() {
    for _ in 0..5000 {
        let mut known = Matrix([[CellContent::Unknown; 7]; 7]);
        let coord_amount = black_box(rand::random::<u8>() % 10 + 5);
        let mut coord_set = CoordSet::<7, 7>::default();
        for _ in 0..coord_amount {
            coord_set.insert(Coord::random());
        }

        let coords = coord_set.iter().collect::<Vec<_>>();

        let solution_amount = black_box(rand::random::<u8>() % 10 + 5);
        let mut solutions = Vec::with_capacity(solution_amount as usize);
        for _ in 0..solution_amount {
            let mut solution: BitVec = bitvec![0; coords.len()];
            solution.fill_with(|_| black_box(rand::random()));
            solutions.push(solution);
        }

        let solution_list = SolutionList::from(solutions.clone(), coords.clone(), 10);
        dbg!(&solution_list);

        let decisions = solution_list.find_trivial_decisions(&mut known);

        for solution in solutions.iter().filter(|s| s.count_ones() > 10) {
            assert!(solution_list.iter().flatten().all(|s| s != solution));
        }

        dbg!(&decisions);
        for (i, coord) in coords.iter().enumerate() {
            let same_idx_solutions = solutions
                .iter()
                .filter(|s| s.count_ones() <= 10)
                .map(|s| s.get(i).unwrap())
                .collect::<BitVec>();

            let as_reveal = Decision::Reveal(*coord);
            let as_flag = Decision::Flag(*coord);

            dbg!(&same_idx_solutions);
            dbg!(&coord);
            dbg!(i);
            if decisions.contains(&as_flag) {
                assert!(same_idx_solutions.all());
            }
            if decisions.contains(&as_reveal) {
                assert!(same_idx_solutions.not_any())
            }
        }
    }
}

/// Ensure that the best guess is always found correctly, at least if the best
/// guess is trivial.
#[test]
fn test_find_best_guess() {
    let mut rng = rand::thread_rng();

    for _ in 0..5000 {
        let coord_amount = black_box(rand::random::<u8>() % 10 + 5);
        let mut coord_set = CoordSet::<7, 7>::default();

        // Get guaranteedly the coord_amount of coords.
        let mut random_coords = CoordSet::<7, 7>::from(true).iter().collect::<Vec<_>>();
        random_coords.shuffle(&mut rng);
        for _ in 0..coord_amount {
            coord_set.insert(random_coords.pop().unwrap());
        }
        let coords = coord_set.iter().collect::<Vec<_>>();

        // Generate empty solution lists for now
        let solution_amount = rand::random::<u8>() % 10 + 5;
        let mut solutions = Vec::with_capacity(solution_amount as usize);
        for _ in 0..solution_amount {
            solutions.push(bitvec![1; coords.len()]);
        }

        // Pick winning coord index and in how many solutions is it true
        let winning_coord_idx = rand::random::<usize>() % coords.len();

        let non_winning_max_false = solution_amount - 4;
        dbg!(non_winning_max_false);

        for (coord_idx, _) in coords.iter().enumerate() {
            if coord_idx == winning_coord_idx {
                for solution in &mut solutions {
                    solution.set(coord_idx, false);
                }
            } else {
                let falses = rand::random::<u8>() % non_winning_max_false;
                let mut indexes = (0..solutions.len()).collect::<Vec<_>>();
                indexes.shuffle(&mut rng);
                let false_indexes = indexes.iter().take(falses as usize).collect::<Vec<_>>();
                dbg!(&falses);
                dbg!(&false_indexes);
                for (solution_idx, solution) in solutions.iter_mut().enumerate() {
                    if false_indexes.contains(&&solution_idx) {
                        solution.set(coord_idx, false);
                    }
                }
            }
        }

        dbg!(winning_coord_idx);
        dbg!(solutions.iter().map(|v| v.to_string()).collect::<Vec<_>>());

        let solution_list = SolutionList::from(solutions.clone(), coords.clone(), 100);

        let best_guess = solution_list.find_best_guess();

        assert_eq!(best_guess.0, *(coords.get(winning_coord_idx).unwrap()));
        assert!((best_guess.1 - 1.) <= 0.05);
    }
}
