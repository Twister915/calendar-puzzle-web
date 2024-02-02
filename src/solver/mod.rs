#![allow(unused_imports)]

mod prelude {
    pub const PUZZLE_WIDTH: usize = 6;
    pub const PUZZLE_HEIGHT: usize = 9;
    pub const NUM_PIECES: usize = 9;

    pub use super::{board::*, mask::*, piece::*, solve::*, state::*, target::*};
    use std::ops::Range;

    pub fn iter_coordinates() -> impl Iterator<Item = (usize, usize)> {
        iter_coordinate_range(0..PUZZLE_WIDTH, 0..PUZZLE_HEIGHT)
    }

    pub fn iter_coordinate_range(
        x_range: Range<usize>,
        y_range: Range<usize>,
    ) -> impl Iterator<Item = (usize, usize)> {
        y_range.flat_map(move |y| x_range.clone().map(move |x| (x, y)))
    }
}

mod board;
mod mask;
mod piece;
mod solve;
mod state;
mod target;

pub use board::{BoardLabel, Month, Weekday, BOARD_LABELS};
pub use mask::{CellTag, TaggedMask};
pub use prelude::{iter_coordinates, NUM_PIECES, PUZZLE_HEIGHT, PUZZLE_WIDTH};
pub use solve::{solve, Solution, SolverMsg};
pub use target::{TargetDate, TargetDateIter};
