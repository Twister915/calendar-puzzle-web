use super::prelude::*;
use std::cmp::{max, min};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Placement {
    pub x: u8,
    pub y: u8,
    pub rotation: u8,
    pub flipped: bool,
}

impl Placement {
    pub const NUM_PLACEMENTS: usize = PUZZLE_WIDTH * PUZZLE_HEIGHT * 4 * 2;

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
            (0..4).flat_map(move |rotation| {
                let (width, height) = piece.size(rotation);
                let start_x = max(0isize, (x as isize) - (width as isize)) as u8;
                let end_x = min(PUZZLE_WIDTH as u8, x + 1);
                let start_y = max(0isize, (y as isize) - (height as isize)) as u8;
                let end_y = min(PUZZLE_HEIGHT as u8, y + 1);

                (start_y..end_y)
                    .flat_map(move |y| {
                        (start_x..end_x).flat_map(move |x| {
                            [false, true].map(move |flipped| Placement {
                                x,
                                y,
                                rotation,
                                flipped,
                            })
                        })
                    })
                    .filter(move |&placement| {
                        piece
                            .mask(placement)
                            .map(|mask| mask.is_covered(x as usize, y as usize))
                            .unwrap_or(false)
                    })
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

    pub fn place_piece(
        &mut self,
        piece_idx: usize,
        placement: Option<Placement>,
        winning_mask: BoardMask,
    ) -> bool {
        if piece_idx >= NUM_PIECES {
            return false;
        }

        // no change?
        if self.pieces[piece_idx] == placement {
            return false;
        }

        // are we placing a piece?
        if let Some(new_placement) = placement {
            // Some if piece_idx is valid and placement is on the board (valid)
            // None otherwise (therefore do not process the update)
            if let Some(mask_update) = mask_for_piece(piece_idx, new_placement) {
                let last_value = &self.pieces[piece_idx];

                // what is our current board mask, without this piece placed anywhere?
                let mut own_mask = if last_value.is_none() {
                    // we are placing a piece that has never been placed before... so we can just
                    // check if there are conflicts with the current mask
                    self.mask
                } else {
                    // we are "moving" this piece, so a mask without this piece placed at all will
                    // be calculated to check for conflicts
                    let mut new_pieces = self.pieces;
                    new_pieces[piece_idx] = None;
                    BoardMask::compute(&new_pieces)
                };

                if !mask_update.conflicts_with(own_mask)
                    && !mask_update.covers_winning_mask(winning_mask)
                {
                    self.pieces[piece_idx] = Some(new_placement);
                    own_mask.apply(mask_update);
                    self.mask = own_mask;
                    return true;
                }
            }
        } else if self.pieces[piece_idx].is_some() {
            // this is also if placement == None
            self.pieces[piece_idx] = None;
            self.mask = BoardMask::compute(&self.pieces);
            return true;
        }

        false
    }

    pub fn mask(&self) -> BoardMask {
        self.mask
    }

    pub fn tagged_mask(&self, winning_mask: BoardMask) -> TaggedMask {
        let mut out = TaggedMask::default();
        for (x, y) in iter_coordinates() {
            if !winning_mask.is_covered(x, y) {
                out.set(x, y, CellTag::Winner);
            }
        }
        for (piece_idx, &placement) in self.pieces.iter().enumerate() {
            if let Some(placement) = placement {
                if let Some(mask) = mask_for_piece(piece_idx, placement) {
                    for (x, y) in iter_coordinates() {
                        if mask.is_covered(x, y) {
                            out.set(x, y, CellTag::Covered(piece_idx as u8));
                        }
                    }
                }
            }
        }

        out
    }

    pub fn open_positions(&self, winning_mask: BoardMask) -> impl Iterator<Item = (u8, u8)> + '_ {
        iter_coordinates()
            .filter(move |(x, y)| {
                !self.mask.is_covered(*x, *y) // is it open?
                    && winning_mask.is_covered(*x, *y)
            }) // should it be open?
            .map(|(x, y)| (x as u8, y as u8))
    }

    pub fn available_piece_idxes(self) -> impl Iterator<Item = usize> {
        (0..NUM_PIECES).filter(move |&idx| self.pieces[idx].is_none())
    }
}
