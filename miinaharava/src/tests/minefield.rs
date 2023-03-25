use std::hint::black_box;

use crate::minefield::{Cell, Coord, GameState, Minefield, MinefieldError};

#[test]
fn test_generation() {
    for mines in 0..=100 {
        let mut minefield = Minefield::<10, 10>::generate(black_box(mines)).unwrap();

        assert_eq!(minefield.mines, black_box(mines));

        let indices = minefield.get_mine_indices();
        let mut mine_count = 0;
        for row in indices {
            for index in row {
                if *index {
                    mine_count += 1;
                }
            }
        }
        assert_eq!(mine_count, mines);
    }
}

#[test]
fn should_fail_on_too_many_mines() {
    assert_eq!(
        Minefield::<10, 10>::generate(101),
        Err(MinefieldError::TooManyMines)
    );
}

#[test]
fn test_game_state() {
    let mut minefield = Minefield::<10, 10>::generate(10).unwrap();
    assert_eq!(minefield.game_state(), GameState::Pending);

    // Test clicking a mine
    let mine_coord = find_cell(&mut minefield, true).unwrap();
    minefield.reveal(mine_coord).unwrap();
    assert_eq!(minefield.game_state(), GameState::GameOver);

    // Test clicking an empty cell
    let mut minefield = Minefield::<10, 10>::generate(10).unwrap();
    let empty_coord = find_cell(&mut minefield, false).unwrap();
    minefield.reveal(empty_coord).unwrap();
    assert_eq!(minefield.game_state(), GameState::Pending);

    // Test clicking every empty cell
    let mut minefield = Minefield::<10, 10>::generate(10).unwrap();
    let indices = *minefield.get_mine_indices();
    for (y, row) in indices.iter().enumerate() {
        for (x, item) in row.iter().enumerate() {
            if !*item {
                // Game might be victorious ahead of time because of automatic
                // recursive reveal, just ok() this
                minefield.reveal(Coord(x, y)).ok();
            }
        }
    }
    assert_eq!(minefield.game_state(), GameState::Victory);
}

#[test]
fn test_automatic_recursive_reveal() {
    use Cell::*;

    let mut minefield = Minefield::<5, 5>::generate(4).unwrap();
    let indices = minefield.get_mine_indices();
    *indices = [
        [false, false, false, false, false],
        [false, false, false, true, true],
        [false, false, false, false, false],
        [false, false, false, false, true],
        [false, false, false, false, true],
    ];

    minefield.reveal(Coord(1, 1)).unwrap();

    let expected = [
        [Empty, Empty, Label(1), Hidden, Hidden],
        [Empty, Empty, Label(1), Hidden, Hidden],
        [Empty, Empty, Label(1), Label(3), Hidden],
        [Empty, Empty, Empty, Label(2), Hidden],
        [Empty, Empty, Empty, Label(2), Hidden],
    ];

    assert_eq!(minefield.field, expected);
}

#[test]
fn test_flag() {
    let mut minefield = Minefield::<10, 10>::generate(10).unwrap();

    let random_coord = Coord::random();

    minefield.flag(random_coord).unwrap();

    for (y, row) in minefield.field.iter().enumerate() {
        for (x, item) in row.iter().enumerate() {
            if Coord(x, y) == random_coord {
                assert_eq!(*item, Cell::Flag);
            } else {
                assert_eq!(*item, Cell::Hidden);
            }
        }
    }

    minefield.flag(random_coord).unwrap();

    for row in minefield.field {
        for item in row {
            assert_eq!(item, Cell::Hidden);
        }
    }

    // Test clicking an empty cell
    let mut minefield = Minefield::<10, 10>::generate(10).unwrap();
    let empty_coord = find_cell(&mut minefield, false).unwrap();
    minefield.reveal(empty_coord).unwrap();
    let curr_cell = minefield.field[empty_coord.1][empty_coord.0];

    // Try to flag and unflag the now revealed cell, should do nothing
    minefield.flag(empty_coord).unwrap();
    assert_eq!(minefield.field[empty_coord.1][empty_coord.0], curr_cell);
    minefield.flag(empty_coord).unwrap();
    assert_eq!(minefield.field[empty_coord.1][empty_coord.0], curr_cell);
}

#[test]
fn test_reveal_and_flag_errors() {
    let mut minefield = Minefield::<10, 10>::generate(10).unwrap();

    let out_of_bounds = Coord(11, 11);

    assert_eq!(
        minefield.reveal(out_of_bounds),
        Err(MinefieldError::InvalidCoordinate)
    );

    assert_eq!(
        minefield.flag(out_of_bounds),
        Err(MinefieldError::InvalidCoordinate)
    );

    // Test clicking a mine
    let mine_coord = find_cell(&mut minefield, true).unwrap();
    minefield.reveal(mine_coord).unwrap();

    // Should now be impossible to click anywhere
    assert_eq!(
        minefield.reveal(Coord::random()),
        Err(MinefieldError::GameHasEnded)
    );
    assert_eq!(
        minefield.flag(Coord::random()),
        Err(MinefieldError::GameHasEnded)
    );
}

fn find_cell<const W: usize, const H: usize>(
    minefield: &mut Minefield<W, H>,
    is_mine: bool,
) -> Option<Coord<W, H>> {
    let indices = minefield.get_mine_indices();
    for (y, row) in indices.iter().enumerate() {
        for (x, item) in row.iter().enumerate() {
            if *item == is_mine {
                return Some(Coord(x, y));
            }
        }
    }
    None
}
