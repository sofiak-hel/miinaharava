use miinaharava::minefield::{Coord, Matrix};

use crate::ai::coord_set::CoordSet;

#[test]
fn test_is_empty() {
    for _ in 0..10000 {
        let set = get_random_set::<30, 30>();
        assert!(set.is_empty() == (set.iter().count() == 0))
    }
}

#[test]
fn test_corners() {
    let expected = CoordSet {
        matrix: Matrix([
            [true, false, false, false, true],
            [false; 5],
            [false; 5],
            [false; 5],
            [true, false, false, false, true],
        ]),
    };

    assert_eq!(CoordSet::corners(), expected);
}

#[test]
fn test_edges() {
    let expected = CoordSet {
        matrix: Matrix([
            [false, true, true, true, false],
            [true, false, false, false, true],
            [true, false, false, false, true],
            [true, false, false, false, true],
            [false, true, true, true, false],
        ]),
    };

    assert_eq!(CoordSet::edges(), expected);
}

#[test]
fn test_middle_with_omit() {
    let expected = CoordSet {
        matrix: Matrix([
            [false; 5],
            [false, true, true, true, false],
            [false, true, true, true, false],
            [false, true, true, true, false],
            [false; 5],
        ]),
    };

    let mut actual = CoordSet::from(true);
    actual.omit(&CoordSet::edges());
    actual.omit(&CoordSet::corners());

    assert_eq!(actual, expected);
}

#[test]
fn test_intersection() {
    for _ in 0..1000 {
        let set_a = get_random_set::<30, 30>();
        let set_b = get_random_set();

        let set_c = set_a.intersection(&set_b);

        for ((a, b), c) in set_a
            .matrix
            .iter()
            .flatten()
            .zip(set_b.matrix.iter().flatten())
            .zip(set_c.matrix.iter().flatten())
        {
            if *a && *b {
                assert!(c);
            } else {
                assert!(!c);
            }
        }
    }
}

#[test]
fn test_omit() {
    for _ in 0..1000 {
        let set_a_original = get_random_set::<30, 30>();
        let mut set_a = set_a_original;
        let set_b = get_random_set();

        set_a.omit(&set_b);

        for ((a_orig, b), a) in set_a_original
            .matrix
            .iter()
            .flatten()
            .zip(set_b.matrix.iter().flatten())
            .zip(set_a.matrix.iter().flatten())
        {
            // If statements could be made simpler with boolean algebra, but is
            // kept like this for clarity
            if *a_orig {
                if *b {
                    assert!(!a);
                } else {
                    assert!(a);
                }
            } else {
                assert!(!a);
            }
        }
    }
}

#[test]
fn test_extend() {
    for _ in 0..1000 {
        let set_a_original = get_random_set::<30, 30>();
        let mut set_a = set_a_original;
        let set_b = get_random_set();

        set_a.extend(&set_b);

        for ((a_orig, b), a) in set_a_original
            .matrix
            .iter()
            .flatten()
            .zip(set_b.matrix.iter().flatten())
            .zip(set_a.matrix.iter().flatten())
        {
            // If statements could be made simpler with boolean algebra, but is
            // kept like this for clarity
            if *a_orig || *b {
                assert!(a);
            } else {
                assert!(!a);
            }
        }
    }
}

#[test]
fn test_contains() {
    for _ in 0..100 {
        let set = get_random_set();
        let mut clone = CoordSet {
            matrix: Matrix([[false; 30]; 30]),
        };

        for (y, row) in clone.matrix.0.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                *cell = set.contains(Coord(x as u8, y as u8));
            }
        }

        assert_eq!(set, clone);
    }
}

#[test]
fn test_iter() {
    for _ in 0..100 {
        let set = get_random_set::<30, 30>();
        let mut definitely_contained = Vec::new();

        for y in 0..30 {
            for x in 0..30 {
                let c = Coord(x as u8, y as u8);
                if set.contains(c) {
                    definitely_contained.push(c);
                }
            }
        }

        let iter_collected: Vec<_> = set.iter().collect();

        assert_eq!(iter_collected, definitely_contained);
    }
}

#[test]
fn test_iter_mut() {
    for _ in 0..100 {
        let mut set = get_random_set::<5, 5>();
        let mut other = get_random_set();
        let clone = set;

        // make sure other is a subset of set, because iter_mut only loops through existant coords
        let mut inverted = CoordSet::from(true);
        inverted.omit(&clone);
        other.omit(&inverted);

        for (exists, coord) in set.iter_mut() {
            assert!(clone.contains(coord));
            *exists = other.contains(coord);
        }

        assert_eq!(set, other);
    }
}

fn get_random_set<const W: usize, const H: usize>() -> CoordSet<W, H> {
    let mut set = CoordSet {
        matrix: Matrix([[false; W]; H]),
    };
    set.matrix.0.fill_with(|| {
        let mut array = [false; W];
        array.fill_with(rand::random);
        array
    });
    set
}
