use super::prelude::*;

use std::fmt;
use std::time::{Duration, Instant};
use crate::return_matching;

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
                start_at: Instant::now(),
            },
            frames: Some(Vec::with_capacity(NUM_PIECES)),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SolverMsg {
    Unsolved(TaggedMask),
    Solved(Solution)
}

impl fmt::Display for SolverMsg {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolverMsg::Solved(solution) =>
                write!(f, "SOVLED in {:.02}s with {} steps\n{}\n", solution.duration.as_secs_f64(), solution.steps, solution.mask),
            SolverMsg::Unsolved(partial) => write!(f, "UNSOLVED\n{}\n", partial),
        }
    }
}

impl Iterator for Solver {
    type Item = SolverMsg;

    fn next(&mut self) -> Option<Self::Item> {
        let frames = self.frames.as_mut()?;
        let next_state = if frames.is_empty() {
            GameState::default()
        } else {
            'l: loop {
                let num_frames = frames.len();
                let current_frame = &mut frames[num_frames - 1];
                for (piece_idx, placement) in &mut current_frame.piece_placements {
                    let mut next_state = current_frame.state;
                    if next_state.place_piece(piece_idx, Some(placement), self.winning_mask) {
                        break 'l next_state
                    }
                }

                frames.pop();
                if frames.is_empty() {
                    self.frames.take();
                    return None;
                }
            }
        };

        if next_state.mask() == self.winning_mask {
            self.frames.take();
            return Some(SolverMsg::Solved(Solution {
                mask: next_state.tagged_mask(self.winning_mask),
                steps: self.stats.steps,
                duration: Instant::now() - self.stats.start_at,
            }))
        }

        frames.push(SolveFrame::create(next_state, self.winning_mask));
        self.stats.steps += 1;
        return Some(SolverMsg::Unsolved(next_state.tagged_mask(self.winning_mask)))
    }
}

struct SolveFrame {
    state: GameState,
    piece_placements: Box<dyn Iterator<Item=(usize, Placement)>>
}

impl SolveFrame {
    fn create(state: GameState, winning_mask: BoardMask) -> Self {
        let piece_placements = Box::new(
            state.open_positions(winning_mask)
                .next()
                .into_iter()
                .flat_map(move |(x, y)|
                    state.available_piece_idxes()
                        .flat_map(move |piece_idx| Placement::iter_covering_coordinates(x, y, piece_idx)
                            .map(move |placement| (piece_idx, placement)))));
        Self { state, piece_placements }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Solution {
    pub mask: TaggedMask,
    pub steps: usize,
    pub duration: Duration,
}

pub fn solve(target: TargetDate) -> impl Iterator<Item=SolverMsg> {
    target.winning_mask().into_iter().flat_map(move |winning_mask| Solver::create(winning_mask))
}

struct SolverStats {
    steps: usize,
    start_at: Instant,
}

pub trait SolverItrExt {

    fn solution(self) -> Option<Solution>;

    fn sample(self, nth: usize) -> impl Iterator<Item=SolverMsg>;
}

impl<T> SolverItrExt for T where T: Iterator<Item=SolverMsg> {
    fn solution(mut self) -> Option<Solution> {
        loop {
            match self.next()? {
                SolverMsg::Solved(solution) => return Some(solution),
                _ => {},
            }
        }
    }

    fn sample(self, nth: usize) -> impl Iterator<Item=SolverMsg> {
        SolverSampler{ upstream: self, nth }
    }
}

struct SolverSampler<I> {
    upstream: I,
    nth: usize
}

impl<I> Iterator for SolverSampler<I>
    where I: Iterator<Item=SolverMsg>
{
    type Item = SolverMsg;

    fn next(&mut self) -> Option<Self::Item> {
        for _ in 0..self.nth {
            match self.upstream.next()? {
                s @ SolverMsg::Solved(_) => return Some(s),
                _ => {},
            }
        }

        self.upstream.next()
    }
}