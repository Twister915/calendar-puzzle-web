#![allow(unused_imports)]

mod prelude {
    pub const PUZZLE_WIDTH: usize = 6;
    pub const PUZZLE_HEIGHT: usize = 9;
    pub const NUM_PIECES: usize = 9;

    use std::ops::Range;
    pub use super::{state::*, solve::*, piece::*, mask::*};

    pub fn iter_coordinates() -> impl Iterator<Item=(usize, usize)> {
        iter_coordinate_range(0..PUZZLE_WIDTH, 0..PUZZLE_HEIGHT)
    }

    pub fn iter_coordinate_range(x_range: Range<usize>, y_range: Range<usize>) -> impl Iterator<Item=(usize, usize)> {
        y_range.flat_map(move |y| x_range.clone().map(move |x| (x, y)))
    }
}

mod state;
mod solve;
mod piece;
mod mask;

pub use solve::{TargetDate, Weekday, Month, solve, Solution, TargetDateIter};
pub use mask::{TaggedMask, CellTag};