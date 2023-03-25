use miinaharava::minefield::{Coord, Minefield};

#[derive(Debug)]
pub enum Decision<const W: usize, const H: usize> {
    Flag(Coord<W, H>),
    Reveal(Coord<W, H>),
}

pub fn ponder<const W: usize, const H: usize>(minefield: &Minefield<W, H>) -> Vec<Decision<W, H>> {
    vec![Decision::Reveal(Coord(5, 5))]
}
