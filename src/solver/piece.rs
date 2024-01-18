use super::prelude::*;

use lazy_static::lazy_static;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Piece<const W: usize, const H: usize> {
    masks: [Option<BoardMask>; Placement::NUM_PLACEMENTS],
}

lazy_static! {
    //
    // ---------------------------------+---------------------------------
    //                                  |     []
    //    [][]                          |     []
    //      []      "SHAPE 0" (3x3)     |     [][][]   "SHAPE 1" (3x4)
    //    [][][]                        |     []
    //                                  |
    // ---------------------------------+---------------------------------
    //
    pub static ref PIECE_0: Piece<3, 3> = Piece::with_mask([
        [true, true, false],
        [false, true, false],
        [true, true, true],
    ]);

    pub static ref PIECE_1: Piece<3, 4> = Piece::with_mask([
        [true, false, false],
        [true, false, false],
        [true, true, true],
        [true, false, false]
    ]);

    //
    // ---------------------------------+---------------------------------
    //                                  |
    //    [][][]                        |       []
    //    []                            |     [][]
    //    []        "SHAPE 2" (3x4)     |       []     "SHAPE 3" (2x5)
    //    []                            |       []
    //                                  |       []
    //                                  |
    // ---------------------------------+---------------------------------
    //
    pub static ref PIECE_2: Piece<3, 4> = Piece::with_mask([
        [true, true, true],
        [true, false, false],
        [true, false, false],
        [true, false, false],
    ]);
    pub static ref PIECE_3: Piece<2, 5> = Piece::with_mask([
        [false, true],
        [true, true],
        [false, true],
        [false, true],
        [false, true],
    ]);

    //
    // ---------------------------------+---------------------------------
    //                                  |
    //                                  |       []
    //    []                            |       []
    //    [][]                          |       []
    //      []      "SHAPE 4" (2x4)     |       []     "SHAPE 5" (2x5)
    //      []                          |     [][]
    //                                  |
    //                                  |
    // ---------------------------------+---------------------------------
    //
    pub static ref PIECE_4: Piece<2, 4> = Piece::with_mask([
        [true, false],
        [true, true],
        [false, true],
        [false, true],
    ]);
    pub static ref PIECE_5: Piece<2, 5> = Piece::with_mask([
        [false, true],
        [false, true],
        [false, true],
        [false, true],
        [true, true],
    ]);

    //
    // ---------------------------------+---------------------------------
    //                                  |
    //      []                          |
    //      []                          |       []
    //      []      "SHAPE 6"  (2x4)    |     [][]     "SHAPE 7" (2x3)
    //    [][]                          |     [][]
    //                                  |
    // ---------------------------------+---------------------------------
    //
    pub static ref PIECE_6: Piece<2, 4> = Piece::with_mask([
        [false, true],
        [false, true],
        [false, true],
        [true, true],
    ]);
    pub static ref PIECE_7: Piece<2, 3> = Piece::with_mask([
        [false, true],
        [true, true],
        [true, true],
    ]);

    //
    // ---------------------------------+
    //                                  |
    //    [][]                          |
    //      [][][]  "SHAPE 8" (4x3)     |
    //      []                          |
    //                                  |
    //                                  |
    // ---------------------------------+
    //
    pub static ref PIECE_8: Piece<4, 3> = Piece::with_mask([
        [true, true, false, false],
        [false, true, true, true],
        [false, true, false, false],
    ]);

}

impl<const W: usize, const H: usize> Piece<W, H> {
    pub fn with_mask(mask: [[bool; W]; H]) -> Self {
        let mut masks = [None; Placement::NUM_PLACEMENTS];
        for placement in Placement::iter_all() {
            let code = placement.code().unwrap();
            masks[code] = Self::mask_for_placement(mask, &placement);
        }

        Self { masks }
    }

    pub fn mask(&self, placement: &Placement) -> Option<BoardMask> {
        placement.code().and_then(|code| self.masks[code])
    }

    fn mask_for_placement(mask: [[bool; W]; H], placement: &Placement) -> Option<BoardMask> {
        let rotated_mask = Self::transformed(mask, placement.rotation, placement.flipped);
        let piece_width = rotated_mask.width();
        let piece_height = rotated_mask.height();
        let mut out = BoardMask::default();
        for y_offset in 0..piece_height {
            for x_offset in 0..piece_width {
                let x = (placement.x as usize) + x_offset;
                let y = (placement.y as usize) + y_offset;
                if x >= PUZZLE_WIDTH || y >= PUZZLE_HEIGHT {
                    return None;
                }

                out.set_covered(x, y, rotated_mask.get(x_offset, y_offset));
            }
        }

        Some(out)
    }

    fn transformed(mut mask: [[bool; W]; H], rotation: u8, flip: bool) -> RotatedMask<W, H> {
        if flip {
            mask = flipped(mask);
        }

        use RotatedMask as RM;
        let mut mask = RM::Horizontal(mask);
        for _ in 0..(rotation % 4) {
            mask = match mask {
                RM::Horizontal(last_mask) => RM::Vertical(rotated(last_mask)),
                RM::Vertical(last_mask) => RM::Horizontal(rotated(last_mask)),
            };
        }

        mask
    }

    pub fn size(&self, rotation: u8) -> (usize, usize) {
        let is_odd_rotation = rotation % 2 == 1;
        if is_odd_rotation {
            (H, W)
        } else {
            (W, H)
        }
    }
}

fn rotated<const W: usize, const H: usize, E: Default + Copy>(input: [[E; W]; H]) -> [[E; H]; W] {
    let mut output = [[E::default(); H]; W];
    for dst_x in (0..H).rev() {
        for dst_y in 0..W {
            output[dst_y][dst_x] = input[dst_x][W - dst_y - 1];
        }
    }
    output
}

fn flipped<const W: usize, const H: usize, E: Copy>(data: [[E; W]; H]) -> [[E; W]; H] {
    let half_w = W / 2;
    let mut out = data;
    for x in 0..half_w {
        for y in 0..H {
            out[y][x] = data[y][W - 1 - x];
            out[y][W - 1 - x] = data[y][x];
        }
    }

    out
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RotatedMask<const W: usize, const H: usize> {
    Horizontal([[bool; W]; H]),
    Vertical([[bool; H]; W]),
}

impl<const W: usize, const H: usize> RotatedMask<W, H> {
    fn width(&self) -> usize {
        use RotatedMask::{Horizontal as Hz, Vertical as Vt};
        match self {
            Hz(_) => W,
            Vt(_) => H,
        }
    }

    fn height(&self) -> usize {
        use RotatedMask::{Horizontal as Hz, Vertical as Vt};
        match self {
            Hz(_) => H,
            Vt(_) => W,
        }
    }

    fn get(&self, x: usize, y: usize) -> bool {
        use RotatedMask::{Horizontal as Hz, Vertical as Vt};
        match self {
            Hz(data) => data[y][x],
            Vt(data) => data[y][x]
        }
    }
}

pub fn mask_for_piece(piece_idx: usize, placement: &Placement) -> Option<BoardMask> {
    match piece_idx {
        0 => PIECE_0.mask(placement),
        1 => PIECE_1.mask(placement),
        2 => PIECE_2.mask(placement),
        3 => PIECE_3.mask(placement),
        4 => PIECE_4.mask(placement),
        5 => PIECE_5.mask(placement),
        6 => PIECE_6.mask(placement),
        7 => PIECE_7.mask(placement),
        8 => PIECE_8.mask(placement),
        _ => None,
    }
}

pub fn size_for_piece(piece_idx: usize, rotation: u8) -> Option<(usize, usize)> {
    Some(match piece_idx {
        0 => PIECE_0.size(rotation),
        1 => PIECE_1.size(rotation),
        2 => PIECE_2.size(rotation),
        3 => PIECE_3.size(rotation),
        4 => PIECE_4.size(rotation),
        5 => PIECE_5.size(rotation),
        6 => PIECE_6.size(rotation),
        7 => PIECE_7.size(rotation),
        8 => PIECE_8.size(rotation),
        _ => return None,
    })
}

