use super::prelude::*;

use lazy_static::lazy_static;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    masks: [Option<BoardMask>; Placement::NUM_PLACEMENTS],
    width: usize,
    height: usize,
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
    //                                  |
    //    [][][]                        |       []
    //    []                            |     [][]
    //    []        "SHAPE 2" (3x4)     |       []     "SHAPE 3" (2x5)
    //    []                            |       []
    //                                  |       []
    //                                  |
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
    //                                  |
    //      []                          |
    //      []                          |       []
    //      []      "SHAPE 6"  (2x4)    |     [][]     "SHAPE 7" (2x3)
    //    [][]                          |     [][]
    //                                  |
    // ---------------------------------+---------------------------------
    //                                  |
    //    [][]                          |
    //      [][][]  "SHAPE 8" (4x3)     |
    //      []                          |
    //                                  |
    //                                  |
    // ---------------------------------+---------------------------------
    //
    pub static ref PIECES: [Piece; NUM_PIECES] = [
        Piece::with_mask([
            [true, true, false],
            [false, true, false],
            [true, true, true],
        ]),
        Piece::with_mask([
            [true, false, false],
            [true, false, false],
            [true, true, true],
            [true, false, false]
        ]),
        Piece::with_mask([
            [true, true, true],
            [true, false, false],
            [true, false, false],
            [true, false, false],
        ]),
        Piece::with_mask([
            [false, true],
            [true, true],
            [false, true],
            [false, true],
            [false, true],
        ]),
        Piece::with_mask([
            [true, false],
            [true, true],
            [false, true],
            [false, true],
        ]),
        Piece::with_mask([
            [false, true],
            [false, true],
            [false, true],
            [false, true],
            [true, true],
        ]),
        Piece::with_mask([
            [false, true],
            [false, true],
            [false, true],
            [true, true],
        ]),
        Piece::with_mask([
            [false, true],
            [true, true],
            [true, true],
        ]),
        Piece::with_mask([
            [true, true, false, false],
            [false, true, true, true],
            [false, true, false, false],
        ])
    ];
}

impl Piece {
    pub fn with_mask<const W: usize, const H: usize>(mask: [[bool; W]; H]) -> Self {
        let mut masks = [None; Placement::NUM_PLACEMENTS];
        for placement in Placement::iter_all() {
            let code = placement.code().unwrap();
            masks[code] = mask_for_placement(mask, &placement);
        }

        Self {
            masks,
            width: W,
            height: H,
        }
    }

    pub fn mask(&self, placement: &Placement) -> Option<BoardMask> {
        placement.code().and_then(|code| self.masks[code])
    }

    pub fn size(&self, rotation: u8) -> (usize, usize) {
        let is_odd_rotation = rotation % 2 == 1;
        if is_odd_rotation {
            (self.height, self.width)
        } else {
            (self.width, self.height)
        }
    }
}

fn mask_for_placement<const W: usize, const H: usize>(
    mask: [[bool; W]; H],
    placement: &Placement,
) -> Option<BoardMask> {
    let rotated_mask = transformed(mask, placement.rotation, placement.flipped);
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

fn transformed<const W: usize, const H: usize>(
    mut mask: [[bool; W]; H],
    rotation: u8,
    flip: bool,
) -> RotatedMask<W, H> {
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
            Vt(data) => data[y][x],
        }
    }
}

pub fn piece(piece_idx: usize) -> Option<&'static Piece> {
    PIECES.get(piece_idx)
}

pub fn mask_for_piece(piece_idx: usize, placement: &Placement) -> Option<BoardMask> {
    piece(piece_idx).and_then(|piece| piece.mask(placement))
}
