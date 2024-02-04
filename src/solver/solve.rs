use super::prelude::*;

use crate::return_matching;
use std::fmt;
use std::time::{Duration, Instant};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Solution {
    pub mask: TaggedMask,
    pub game: GameState,
    pub steps: usize,
    #[cfg(feature = "timed")]
    pub duration: Duration,
}

pub fn solve(target: TargetDate) -> impl Iterator<Item = SolverMsg> {
    target.winning_mask().into_iter().flat_map(Solver::create)
}

struct Solver {
    winning_mask: BoardMask,
    stats: SolverStats,
    frames: Option<Vec<SolveFrame>>,
}

impl Solver {
    pub fn create(winning_mask: BoardMask) -> Self {
        Self {
            winning_mask,
            stats: SolverStats {
                steps: 0,
                #[cfg(feature = "timed")]
                start_at: Instant::now(),
            },
            // this capacity of NUM_PIECES is because we actually can only have one frame per placed piece.
            // that is because we only create a frame after placing a piece.
            frames: Some(Vec::with_capacity(NUM_PIECES)),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SolverMsg {
    Unsolved(GameState, TaggedMask),
    Solved(Solution),
    Impossible,
}

impl fmt::Display for SolverMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolverMsg::Solved(solution) => {
                write!(f, "SOLVED in {} steps\n{}\n", solution.steps, solution.mask)
            }
            SolverMsg::Unsolved(_, tagged_mask) => write!(f, "UNSOLVED\n{}\n", tagged_mask),
            SolverMsg::Impossible => write!(f, "IMPOSSIBLE!!"),
        }
    }
}

impl fmt::Debug for SolverMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Iterator for Solver {
    type Item = SolverMsg;

    fn next(&mut self) -> Option<Self::Item> {
        // get the current solver frame
        //
        // if the frames Vec is taken, then we "fused" the iterator (solution already determined)
        // therefore the `?` here will return None in that case
        //
        let frames = self.frames.as_mut()?;

        // next_state = the next step of solving that we plan to return
        //
        // the initial state of this type Solver is `frames = Some(Vec::default())` essentially,
        // so when we encounter that first state, we initialize a default (empty) board as our "next_state"
        //
        // otherwise... check the else branch
        let next_state = if frames.is_empty() {
            GameState::default()
        } else {
            // this is the other state of solver... non-default state, where we've made some progress, and want to
            // continue solving

            // this loop calculates the next board state
            'l: loop {
                // get the latest frame (peek the last SolverFrame in frames)
                let num_frames = frames.len();
                let current_frame = &mut frames[num_frames - 1];

                // go through the iterator `piece_placements` to find the next valid move to make in this frame
                while let Some((piece_idx, placement)) = current_frame.next_piece_placement() {
                    // if we find a move, and can make it, then that is our next state!
                    if let Some(next_state) = current_frame.state.with_piece_placed(
                        piece_idx,
                        placement,
                        self.winning_mask,
                    ) {
                        break 'l next_state;
                    }
                }

                // if we never found a move to make, then this frame is impossible, so we should remove it
                // due to this being in a `loop` called 'l, this will cause us to simply move up one frame
                frames.pop();

                // if that was the last frame, then technically the entire puzzle is impossible, so we completely fuse
                if frames.is_empty() {
                    self.frames.take();
                    return Some(SolverMsg::Impossible);
                }
            }
        };

        // we either solved the puzzle or we didn't
        Some(if next_state.mask() == self.winning_mask {
            // if we solve the puzzle, just return immediately
            self.frames.take();
            SolverMsg::Solved(Solution {
                game: next_state,
                mask: next_state.tagged_mask(self.winning_mask),
                steps: self.stats.steps,
                #[cfg(feature = "timed")]
                duration: Instant::now() - self.stats.start_at,
            })
        } else {
            // otherwise, push a new frame
            frames.push(SolveFrame::create(next_state, self.winning_mask));
            self.stats.steps += 1;
            SolverMsg::Unsolved(next_state, next_state.tagged_mask(self.winning_mask))
        })
    }
}

struct SolveFrame {
    state: GameState,
    cover_x: u8,
    cover_y: u8,
    // Bitfield of piece indexes left to try, includes the index currently used by piece_positions
    pieces_to_try: u16,
    piece_positions: PiecePositions,
}

impl SolveFrame {
    fn create(state: GameState, winning_mask: BoardMask) -> Self {
        let (cover_x, cover_y) = state
            .mask()
            .next_to_cover(winning_mask)
            .expect("solve frame for a solved board");
        let pieces_to_try: u16 = state
            .available_piece_idxes()
            .fold(0, |acc, piece_idx| acc | (1 << piece_idx));
        let piece = piece(pieces_to_try.trailing_zeros() as usize)
            .expect("solve frame with no pieces left");
        let piece_positions = PiecePositions::new(piece);
        Self {
            state,
            cover_x,
            cover_y,
            pieces_to_try,
            piece_positions,
        }
    }

    fn next_piece_placement(&mut self) -> Option<(usize, Placement)> {
        loop {
            if let Some(placement) = self
                .piece_positions
                .next_covering(self.cover_x, self.cover_y)
            {
                let piece_idx = self.pieces_to_try.trailing_zeros() as usize;
                return Some((piece_idx, placement));
            }
            // Unset the bit for this piece, and try the next one
            self.pieces_to_try &= self.pieces_to_try - 1;
            self.piece_positions =
                PiecePositions::new(piece(self.pieces_to_try.trailing_zeros() as usize)?);
        }
    }
}

struct SolverStats {
    steps: usize,
    #[cfg(feature = "timed")]
    start_at: Instant,
}
