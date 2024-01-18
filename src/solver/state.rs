use super::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Placement {
    pub x: u8,
    pub y: u8,
    pub rotation: u8,
    pub flipped: bool,
}

impl Placement {
    pub const NUM_PLACEMENTS: usize = PUZZLE_WIDTH * PUZZLE_HEIGHT * 4 * 2;

    pub fn code(&self) -> Option<usize> {
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
        let flipped = if self.flipped { 1usize } else { 0usize };

        Some(pos_code << 3 | (rotation << 1) | flipped)
    }

    pub fn linear_iter(piece_idx: usize) -> impl Iterator<Item=Placement> {
        (0..4).flat_map(move |r| {
            let rotation = r as u8;
            size_for_piece(piece_idx, rotation).into_iter().flat_map(move |(piece_width, piece_height)| {
                let x_limit = PUZZLE_WIDTH - (piece_width - 1);
                let y_limit = PUZZLE_HEIGHT - (piece_height - 1);

                iter_coordinate_range(0..x_limit, 0..y_limit)
                    .flat_map(move |(x, y)|
                        [false, true].map(move |flipped|
                            Placement {
                                x: x as u8,
                                y: y as u8,
                                flipped,
                                rotation,
                            }))
            })
        })
    }

    pub fn random_iter(piece_idx: usize) -> impl Iterator<Item=Placement> {
        RandomSeqIter::<4>::default().flat_map(move |r| {
            let rotation = r as u8;
            size_for_piece(piece_idx, rotation).into_iter().flat_map(move |(piece_width, piece_height)| {
                let mut ys = RandomSeqIter::<PUZZLE_HEIGHT>::default();
                for y in (PUZZLE_HEIGHT - (piece_height - 1))..PUZZLE_HEIGHT {
                    ys.mark_chosen(y);
                }

                ys.flat_map(move |y| {
                    let mut xs = RandomSeqIter::<PUZZLE_WIDTH>::default();
                    for x in (PUZZLE_WIDTH - (piece_width - 1))..PUZZLE_WIDTH {
                        xs.mark_chosen(x);
                    }

                    xs.flat_map(move |x| {
                        let x = x as u8;
                        let y = y as u8;
                        let initial_flipped = fastrand::bool();
                        [initial_flipped, !initial_flipped].map(move |flipped| {
                            Placement {
                                x,
                                y,
                                flipped,
                                rotation,
                            }
                        })
                    })
                })
            })
        })
    }

    pub fn iter_all() -> impl Iterator<Item=Placement> {
        iter_coordinates().flat_map(move |(x, y)|
            (0..4).flat_map(move |rotation|
                [false, true].map(move |flipped|
                    Placement { x: x as u8, y: y as u8, flipped, rotation })))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct GameState {
    pieces: [Option<Placement>; NUM_PIECES],
    mask: BoardMask,
}

impl GameState {
    pub fn place_piece(&mut self, piece_idx: usize, placement: Option<Placement>, winning_mask: BoardMask) -> bool {
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
            if let Some(mask_update) = mask_for_piece(piece_idx, &new_placement) {
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

                if !mask_update.conflicts_with(own_mask) && !mask_update.covers_winning_mask(winning_mask) {
                    self.pieces[piece_idx] = Some(new_placement);
                    own_mask.apply(mask_update);
                    self.mask = own_mask;
                    return true;
                }
            }
        } else if self.pieces[piece_idx].is_some() { // this is also if placement == None
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
                out.set(x, y, CellTag::Winner)
            }
        }
        for (piece_idx, placement) in self.pieces.iter().enumerate() {
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

    pub fn contains_no_islands(&self, winning_mask: BoardMask) -> bool {
        // not ( any position is island )
        // position is island = not (can any piece cover position)
        !self.open_positions(winning_mask).any(move |(x, y)|
            !self.can_any_piece_cover_position(winning_mask, x, y))
    }

    fn open_positions(
        &self,
        winning_mask: BoardMask,
    ) -> impl Iterator<Item=(u8, u8)> + '_
    {
        iter_coordinates()
            .filter(move |(x, y)|
                !self.mask.is_covered(*x, *y) // is it open?
                    && winning_mask.is_covered(*x, *y)) // should it be open?
            .map(|(x, y)| (x as u8, y as u8))
    }

    pub fn can_any_piece_cover_position(&self, winning_mask: BoardMask, x: u8, y: u8) -> bool {
        self.available_piece_idxes()
            .flat_map(move |piece_idx| Placement::linear_iter(piece_idx)
                .filter_map(move |placement| mask_for_piece(piece_idx, &placement)))
            .any(|piece_mask|
                piece_mask.is_covered(x as usize, y as usize) // does the piece cover (x, y)?
                    && !self.mask.conflicts_with(piece_mask) // conflict with other pieces?
                    && !piece_mask.covers_winning_mask(winning_mask)) // conflict with winning mask?
    }

    pub fn available_piece_idxes(&self) -> impl Iterator<Item=usize> + '_ {
        (0..NUM_PIECES).filter_map(|idx|
            if self.pieces[idx].is_none() { Some(idx) } else { None })
    }

    pub fn random_available_piece_idxes(&self) -> impl Iterator<Item=usize> {
        let mut out = RandomSeqIter::<NUM_PIECES>::default();
        for piece_idx in 0..NUM_PIECES {
            if self.pieces[piece_idx].is_some() {
                out.mark_chosen(piece_idx);
            }
        }

        out
    }
}

pub struct RandomSeqIter<const N: usize> {
    chosen: [bool; N],
}

impl<const N: usize> Default for RandomSeqIter<N> {
    fn default() -> Self {
        Self { chosen: [false; N] }
    }
}

impl<const N: usize> Iterator for RandomSeqIter<N> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let num_available = self.available().count();
        if num_available == 0 {
            return None;
        }

        let next_idx = if num_available == 1 {
            self.available().next()
        } else {
            // let skip = 0;
            let skip = fastrand::usize(0..num_available);
            self.available().nth(skip)
        }.unwrap();

        self.mark_chosen(next_idx);
        Some(next_idx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(N))
    }
}

impl<const N: usize> RandomSeqIter<N> {
    fn available(&self) -> impl Iterator<Item=usize> + '_ {
        self.chosen.iter().enumerate().filter_map(|(idx, chosen)| if !chosen {
            Some(idx)
        } else {
            None
        })
    }

    pub fn mark_chosen(&mut self, idx: usize) {
        if N > idx {
            self.chosen[idx] = true;
        }
    }
}