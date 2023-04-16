use std::hint::black_box;

use bitvec::prelude::*;
use bitvec::vec::BitVec;
use miinaharava::minefield::{Coord, Matrix};
use rand::seq::SliceRandom;
use rand::Rng;

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
/// Ensure min and max mines return the correct values
#[test]
fn test_solution_list_min_max() {
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

        let max_mines = black_box(rand::random::<u8>() % (coord_amount - 4) + 4);
        let min_mines = black_box(rand::random::<u8>() % (max_mines - 1) + 1);

        // Generate empty solution lists for now
        let solution_amount = black_box(rand::random::<u8>() % 10 + 5);
        let mut solutions = Vec::with_capacity(solution_amount as usize);

        let mut actual_min = u8::MAX;
        let mut actual_max = 0;

        for _ in 0..solution_amount {
            let mut solution = bitvec![0; coords.len()];

            let mine_amount = rng.gen_range(min_mines..=max_mines);
            actual_min = actual_min.min(mine_amount);
            actual_max = actual_max.max(mine_amount);

            let mut idx_list = (0..coord_amount).collect::<Vec<_>>();
            idx_list.shuffle(&mut rng);
            for idx in idx_list.iter().take(mine_amount as usize) {
                solution.set(*idx as usize, true);
            }
            solutions.push(solution);
        }

        let solution_list = SolutionList::from(solutions.clone(), coords.clone(), 100);
        assert_eq!(solution_list.min_mines(), actual_min);
        assert_eq!(solution_list.max_mines(), actual_max);
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
        let solution_amount = black_box(rand::random::<u8>() % 10 + 5);
        let mut solutions = Vec::with_capacity(solution_amount as usize);
        for _ in 0..solution_amount {
            solutions.push(bitvec![1; coords.len()]);
        }

        // Pick winning coord index and in how many solutions is it true
        let winning_coord_idx = black_box(rand::random::<usize>() % coords.len());

        let non_winning_max_false = solution_amount - 4;
        dbg!(non_winning_max_false);

        for (coord_idx, _) in coords.iter().enumerate() {
            if coord_idx == winning_coord_idx {
                for solution in &mut solutions {
                    solution.set(coord_idx, false);
                }
            } else {
                let falses = black_box(rand::random::<u8>() % non_winning_max_false);
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

struct BestGuessMock<const W: usize, const H: usize>(Coord<W, H>, f32, u8, u8);

impl<const W: usize, const H: usize> SolutionContainer<W, H> for BestGuessMock<W, H> {
    fn find_best_guess(&self) -> (Coord<W, H>, f32) {
        (self.0, self.1)
    }

    fn max_mines(&self) -> u8 {
        self.3
    }

    fn min_mines(&self) -> u8 {
        self.2
    }
}

/// Ensure that the best guess is always found correctly, at least if the best
/// guess is trivial.
#[test]
fn test_find_best_guess_from_vec() {
    let mut rng = rand::thread_rng();

    for _ in 0..5000 {
        // Get guaranteedly the coord_amount of coords.
        let mut random_coords = CoordSet::<7, 7>::from(true).iter().collect::<Vec<_>>();
        random_coords.shuffle(&mut rng);

        let hypothetical_max = 100.;

        let solution_amount = black_box(rand::random::<u8>() % 20 + 10);
        let mut mock_guesses = Vec::new();

        let best_guess_amount = black_box(rand::random::<u8>() % 50 + 50);
        let best_guess_coord = random_coords.pop().unwrap();
        let best_guess_propability = best_guess_amount as f32 / hypothetical_max;
        mock_guesses.push(BestGuessMock(
            best_guess_coord,
            best_guess_propability,
            10,
            10,
        ));

        let non_best_guess_max = best_guess_amount - 25;

        for _ in 0..solution_amount {
            let random_amount = black_box(rand::random::<u8>() % non_best_guess_max);
            mock_guesses.push(BestGuessMock(
                random_coords.pop().unwrap(),
                random_amount as f32 / hypothetical_max,
                10,
                10,
            ));
        }

        mock_guesses.shuffle(&mut rng);

        let best_guess = mock_guesses.find_best_guess();
        assert_eq!(best_guess, (best_guess_coord, best_guess_propability));
    }
}

/// Ensure that the best guess is always found correctly, at least if the best
/// guess is trivial.
#[test]
fn test_mines_min_max_from_vec() {
    let mut rng = rand::thread_rng();

    for _ in 0..5000 {
        // Get guaranteedly the coord_amount of coords.
        let mut random_coords = CoordSet::<7, 7>::from(true).iter().collect::<Vec<_>>();
        random_coords.shuffle(&mut rng);

        let hypothetical_max = 100.;

        let max_mines = black_box(rand::random::<u8>() % 5 + 5);
        let min_mines = black_box(rand::random::<u8>() % (max_mines - 4) + 4);

        dbg!(min_mines);
        dbg!(max_mines);

        let solution_amount = black_box(rand::random::<u8>() % 10 + 10);
        let mut mock_guesses = Vec::new();

        let mut actual_min_mines = 0;
        let mut actual_max_mines = 0;

        for _ in 0..solution_amount {
            let random_amount = black_box(rand::random::<u8>() % 100);
            let max = rng.gen_range(min_mines..=max_mines);
            let min = rng.gen_range(min_mines..=max);
            dbg!(min, max);
            actual_min_mines += min;
            actual_max_mines += max;
            mock_guesses.push(BestGuessMock(
                random_coords.pop().unwrap(),
                random_amount as f32 / hypothetical_max,
                min,
                max,
            ));
        }

        mock_guesses.shuffle(&mut rng);

        assert_eq!(mock_guesses.min_mines(), actual_min_mines);
        assert_eq!(mock_guesses.max_mines(), actual_max_mines);
    }
}
