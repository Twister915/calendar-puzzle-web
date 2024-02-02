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
    target
        .winning_mask()
        .into_iter()
        .flat_map(move |winning_mask| Solver::create(winning_mask))
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
                for (piece_idx, placement) in &mut current_frame.piece_placements {
                    // if we find a move, and can make it, then that is our next state!
                    let mut next_state = current_frame.state;
                    if next_state.place_piece(piece_idx, Some(placement), self.winning_mask) {
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
    piece_placements: Box<dyn Iterator<Item = (usize, Placement)>>,
}

impl SolveFrame {
    fn create(state: GameState, winning_mask: BoardMask) -> Self {
        let piece_placements = Box::new(
            state
                .open_positions(winning_mask)
                .next()
                .into_iter()
                .flat_map(move |(x, y)| {
                    state.available_piece_idxes().flat_map(move |piece_idx| {
                        Placement::iter_covering_coordinates(x, y, piece_idx)
                            .map(move |placement| (piece_idx, placement))
                    })
                }),
        );
        Self {
            state,
            piece_placements,
        }
    }
}

struct SolverStats {
    steps: usize,
    #[cfg(feature = "timed")]
    start_at: Instant,
}
