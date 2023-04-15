use miinaharava::minefield::Matrix;

use crate::ai::coord_set::CoordSet;

#[test]
fn test_coord_set_is_empty() {
    for _ in 0..1000 {
        let mut set = CoordSet {
            matrix: Matrix([[false; 30]; 30]),
        };
        set.matrix.0.fill_with(|| {
            let mut array = [false; 30];
            array.fill_with(rand::random);
            array
        });

        assert!(set.is_empty() == (set.iter().count() == 0))
    }
}
