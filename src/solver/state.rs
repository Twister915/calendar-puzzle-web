use super::prelude::*;
use std::cmp::{max, min};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct RotationAndFlip(u8);

impl RotationAndFlip {
    pub const NUM: usize = 4 * 2;

    pub const fn new(rotation: u8, flipped: bool) -> Self {
        RotationAndFlip((rotation % 4) | (flipped as u8) << 2)
    }

    pub const fn rotation(self) -> u8 {
        self.0 & 0b11
    }

    pub const fn flipped(self) -> bool {
        (self.0 & 0b100) != 0
    }

    pub const fn code(self) -> u8 {
        self.0
    }

    pub fn iter_all() -> impl Iterator<Item = RotationAndFlip> {
        (0..(Self::NUM as u8)).map(Self)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Placement {
    pub x: u8,
    pub y: u8,
    pub rotation: u8,
    pub flipped: bool,
}

impl Placement {
    pub const NUM_PLACEMENTS: usize = PUZZLE_WIDTH * PUZZLE_HEIGHT * 4 * 2;

    pub fn rotation_and_flip(self) -> RotationAndFlip {
        RotationAndFlip::new(self.rotation, self.flipped)
    }

    pub fn code(self) -> Option<usize> {
        let x = self.x as usize;
        let y = self.y as usize;
        if x >= PUZZLE_WIDTH {
            return None;
        }

        if y >= PUZZLE_HEIGHT {
            return None;
        }

        let pos_code = y * PUZZLE_WIDTH + x;
        let rotation = (self.rotation % 4) as usize;
        let flipped = usize::from(self.flipped);

        Some(pos_code << 3 | (rotation << 1) | flipped)
    }

    pub fn iter_all() -> impl Iterator<Item = Placement> {
        iter_coordinates().flat_map(move |(x, y)| {
            (0..4).flat_map(move |rotation| {
                [false, true].map(move |flipped| Placement {
                    x: x as u8,
                    y: y as u8,
                    flipped,
                    rotation,
                })
            })
        })
    }

    pub fn iter_covering_coordinates(
        x: u8,
        y: u8,
        piece_idx: usize,
    ) -> impl Iterator<Item = Placement> {
        piece(piece_idx).into_iter().flat_map(move |piece| {
            (0..4)
                .flat_map(move |rotation| [false, true].map(|flipped| (rotation, flipped)))
                .flat_map(move |(rotation, flipped)| {
                    let (w, h) = piece.size(rotation);
                    piece.relative_offset_iter(rotation, flipped).filter_map(
                        // For each point in the piece, position it so that point is at (x, y)
                        // e.g if the piece has (0, 0), we place it at (x, y) to place that point at (x, y)
                        // if the piece has (1, 1), we place it at (x - 1, y - 1)
                        move |(dx, dy)| -> Option<Placement> {
                            let position_x = x.checked_sub(dx)?;
                            if position_x as usize + w > PUZZLE_WIDTH {
                                return None;
                            }
                            let position_y = y.checked_sub(dy)?;
                            if position_y as usize + h > PUZZLE_HEIGHT {
                                return None;
                            }
                            Some(Placement {
                                x: position_x,
                                y: position_y,
                                rotation,
                                flipped,
                            })
                        },
                    )
                })
        })
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct GameState {
    pieces: [Option<Placement>; NUM_PIECES],
    mask: BoardMask,
}

impl GameState {
    pub fn pieces(&self) -> [Option<Placement>; NUM_PIECES] {
        self.pieces
    }

    pub fn with_piece_placed(
        &self,
        piece_idx: usize,
        placement: Placement,
        winning_mask: BoardMask,
    ) -> Option<Self> {
        if piece_idx >= NUM_PIECES {
            return None;
        }
        debug_assert!(self.pieces[piece_idx].is_none());

        // are we placing a piece?
        // Some if piece_idx is valid and placement is on the board (valid)
        // None otherwise (therefore do not process the update)
        if let Some(mask_update) = mask_for_piece(piece_idx, placement) {
            if !mask_update.conflicts_with(self.mask)
                && !mask_update.covers_winning_mask(winning_mask)
            {
                let mut out = *self;
                out.pieces[piece_idx] = Some(placement);
                out.mask.apply(mask_update);
                return Some(out);
            }
        }

        None
    }

    pub fn mask(&self) -> BoardMask {
        self.mask
    }

    pub fn tagged_mask(&self, winning_mask: BoardMask) -> TaggedMask {
        let mut out = TaggedMask::default();
        for (x, y) in winning_mask.inverted().iter_covered() {
            out.set(usize::from(x), usize::from(y), CellTag::Winner);
        }
        for (piece_idx, &placement) in self.pieces.iter().enumerate() {
            if let Some(placement) = placement {
                let mask = mask_for_piece(piece_idx, placement)
                    .expect("game state should have a valid piece placement");
                for (x, y) in mask.iter_covered() {
                    out.set(
                        usize::from(x),
                        usize::from(y),
                        CellTag::Covered(piece_idx as u8),
                    );
                }
            }
        }

        out
    }

    pub fn available_piece_idxes(self) -> impl Iterator<Item = usize> {
        (0..NUM_PIECES).filter(move |&idx| self.pieces[idx].is_none())
    }
}
