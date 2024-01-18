use super::prelude::*;

use std::fmt;
use std::time::{Duration, Instant};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Solution {
    pub mask: TaggedMask,
    pub steps: usize,
    pub duration: Duration,
}

pub fn solve(target: &TargetDate) -> Option<Solution> {
    let mut stats = SolverStats { steps: 0, start_at: Instant::now() };
    solve_step(GameState::default(), target.winning_mask()?, &mut stats)
}

struct SolverStats {
    steps: usize,
    start_at: Instant,
}

fn solve_step(state: GameState, winning_mask: BoardMask, stats: &mut SolverStats) -> Option<Solution> {
    stats.steps += 1;
    if state.mask() == winning_mask {
        return Some(Solution {
            mask: state.tagged_mask(winning_mask),
            steps: stats.steps,
            duration: Instant::now() - stats.start_at
        });
    }

    // pick a position to cover
    let (x, y) = state.open_positions(winning_mask).next()?;

    // all piece placement pairs that cover the position x, y
    let piece_placements = state.available_piece_idxes()
        .flat_map(move |piece_idx| Placement::iter_covering_coordinates(x, y, piece_idx)
            .map(move |placement| (piece_idx, placement)));

    // attempt to place the pieces on the board, if possible recurse
    for (piece_idx, placement) in piece_placements {
        let mut next_state = state;
        if next_state.place_piece(piece_idx, Some(placement), winning_mask) {
            crate::return_matching!(solve_step(next_state, winning_mask, stats), Some(_))
        }
    }

    None
}