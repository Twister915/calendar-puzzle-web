use super::prelude::*;

use fmt::Write;
use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct BoardMask(u64);

impl BoardMask {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn compute(positions: &[Option<Placement>; NUM_PIECES]) -> Self {
        let mut out = Self::default();
        positions
            .iter()
            .enumerate()
            .filter_map(|(idx, &placement)| placement.and_then(|p| mask_for_piece(idx, p)))
            .for_each(|mask| out.apply(mask));
        out
    }

    pub const fn filled() -> Self {
        Self(u64::MAX >> (64 - (PUZZLE_WIDTH * PUZZLE_HEIGHT)))
    }

    pub const fn is_covered(self, x: usize, y: usize) -> bool {
        self.0 & Self::mask(x, y) != 0
    }

    pub fn set_covered(&mut self, x: usize, y: usize, value: bool) {
        let mask = Self::mask(x, y);
        if value {
            self.0 |= mask;
        } else if self.0 & mask != 0 {
            self.0 ^= mask;
        }
    }

    /// Return a mask which is shifted by the given amount
    ///
    /// Any values which are shifted off the board are lost.
    pub const fn shifted(mut self, x: usize, y: usize) -> Self {
        if y >= PUZZLE_HEIGHT || x >= PUZZLE_WIDTH {
            return Self::new();
        }
        let shift = (y * PUZZLE_WIDTH) + x;
        self.0 <<= shift;

        let mut mask = Self::filled().0;
        if x != 0 {
            // contains the low `x` bits of the second row (because nothing could be shifted into the first row)
            let mut low_bits_per_row = ((1u64 << x) - 1) << PUZZLE_WIDTH;
            let mut rows_counted = 1;
            while rows_counted < PUZZLE_HEIGHT {
                // The first iteration, we'll have a single row, and add a second row to the mask
                // The second iteration, we'll shift our two rows up by two, to get four rows
                // Then finally, we'll shift our four rows up by four, to get all eight rows
                low_bits_per_row |= low_bits_per_row << (PUZZLE_WIDTH * rows_counted);
                rows_counted *= 2;
            }
            mask &= !low_bits_per_row;
        }
        self.0 &= mask;

        self
    }

    pub fn inverted(self) -> Self {
        Self(!self.0 & Self::filled().0)
    }

    const fn mask(x: usize, y: usize) -> u64 {
        1u64 << ((y * PUZZLE_WIDTH) + x) as u64
    }

    pub const fn conflicts_with(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub const fn covers_winning_mask(self, winning_mask: Self) -> bool {
        self.0 & !winning_mask.0 != 0
    }

    pub fn apply(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn next_to_cover(self, winning_mask: Self) -> Option<(u8, u8)> {
        let to_cover = !self.0 & winning_mask.0;
        // The number of trailing zeros is also the index of the first 1
        let pos = to_cover.trailing_zeros() as usize;
        let (x, y) = (pos % PUZZLE_WIDTH, pos / PUZZLE_WIDTH);
        if y < PUZZLE_HEIGHT {
            Some((x.try_into().unwrap(), y.try_into().unwrap()))
        } else {
            None
        }
    }

    pub fn iter_covered(self) -> BoardMaskIter {
        BoardMaskIter(self.0)
    }
}

#[derive(Default, Clone)]
pub struct BoardMaskIter(u64);

impl Iterator for BoardMaskIter {
    type Item = (u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.0.trailing_zeros() as usize;
        if pos == 64 {
            None
        } else {
            // Unset the lowest bit
            self.0 &= self.0 - 1;
            let (x, y) = (pos % PUZZLE_WIDTH, pos / PUZZLE_WIDTH);
            Some((x.try_into().unwrap(), y.try_into().unwrap()))
        }
    }
}

impl fmt::Display for BoardMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (x, y) in iter_coordinates() {
            f.write_char('[')?;
            if self.is_covered(x, y) {
                f.write_char('*')
            } else {
                f.write_char(' ')
            }?;
            f.write_char(']')?;

            if x == PUZZLE_WIDTH - 1 && y != PUZZLE_HEIGHT - 1 {
                f.write_char('\n')?;
            }
        }

        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CellTag {
    Covered(u8),
    Winner,
    Uncovered,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct TaggedMask([[CellTag; PUZZLE_WIDTH]; PUZZLE_HEIGHT]);

impl Default for TaggedMask {
    fn default() -> Self {
        Self([[CellTag::Uncovered; PUZZLE_WIDTH]; PUZZLE_HEIGHT])
    }
}

impl TaggedMask {
    pub fn get(&self, x: usize, y: usize) -> CellTag {
        self.0[y][x]
    }

    pub fn set(&mut self, x: usize, y: usize, tag: CellTag) {
        self.0[y][x] = tag;
    }
}

impl From<TaggedMask> for BoardMask {
    fn from(value: TaggedMask) -> Self {
        let mut out = Self::default();
        for (x, y) in iter_coordinates() {
            out.set_covered(x, y, matches!(value.get(x, y), CellTag::Covered(_)));
        }

        out
    }
}

impl fmt::Display for TaggedMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (x, y) in iter_coordinates() {
            f.write_char('[')?;

            match self.get(x, y) {
                CellTag::Winner => f.write_char('*'),
                CellTag::Uncovered => f.write_char(' '),
                CellTag::Covered(piece_idx) => write!(f, "{}", piece_idx),
            }?;

            f.write_char(']')?;

            if x == PUZZLE_WIDTH - 1 && y != PUZZLE_HEIGHT - 1 {
                f.write_char('\n')?;
            }
        }

        Ok(())
    }
}

impl fmt::Debug for TaggedMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

#[test]
fn shift_board() {
    let mut board = BoardMask::default();
    board.set_covered(0, 0, true);
    board.set_covered(2, 1, true);
    board.set_covered(PUZZLE_WIDTH - 1, 0, true);
    board.set_covered(PUZZLE_WIDTH - 1, PUZZLE_HEIGHT - 1, true);
    let shifted = board.shifted(1, 1);

    let mut expected_board = BoardMask::default();
    expected_board.set_covered(1, 1, true);
    expected_board.set_covered(3, 2, true);
    assert_eq!(shifted.0, expected_board.0);
}
