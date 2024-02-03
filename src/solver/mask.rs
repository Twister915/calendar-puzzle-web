use super::prelude::*;

use fmt::Write;
use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct BoardMask(u64);

impl BoardMask {
    pub fn compute(positions: &[Option<Placement>; NUM_PIECES]) -> Self {
        let mut out = Self::default();
        positions
            .iter()
            .enumerate()
            .filter_map(|(idx, &placement)| placement.and_then(|p| mask_for_piece(idx, p)))
            .for_each(|mask| out.apply(mask));
        out
    }

    pub fn filled() -> Self {
        Self(u64::MAX >> (64 - (PUZZLE_WIDTH * PUZZLE_HEIGHT)))
    }

    pub fn is_covered(self, x: usize, y: usize) -> bool {
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

    fn mask(x: usize, y: usize) -> u64 {
        1u64 << ((y * PUZZLE_WIDTH) + x) as u64
    }

    pub fn conflicts_with(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub fn covers_winning_mask(self, winning_mask: Self) -> bool {
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

    pub fn iter_covered(mut self) -> impl Iterator<Item = (u8, u8)> {
        std::iter::from_fn(move || {
            let pos = self.0.trailing_zeros() as usize;
            if pos == 64 {
                None
            } else {
                // Unset the lowest bit
                self.0 &= self.0 - 1;
                let (x, y) = (pos % PUZZLE_WIDTH, pos / PUZZLE_WIDTH);
                Some((x.try_into().unwrap(), y.try_into().unwrap()))
            }
        })
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
