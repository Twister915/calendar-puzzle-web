use super::prelude::*;

use std::fmt;
use std::time::{Duration, Instant};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Solution {
    pub mask: TaggedMask,
    pub steps: usize,
    pub boards: usize,
    pub duration: Duration,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SolverDecision {
    Solved(Solution),
    Impossible,
    Tired,
}

impl fmt::Display for SolverDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolverDecision::Impossible => f.write_str("Impossible"),
            SolverDecision::Tired => f.write_str("Tired..."),
            SolverDecision::Solved(solution) => {
                let secs = solution.duration.as_secs_f64();
                let boards_per_sec = solution.boards as f64 / secs;
                writeln!(f, "Solved in {} steps ({} boards evaluated in {:.02} sec... {:.02}/sec)",
                         solution.steps, solution.boards, secs, boards_per_sec)?;
                solution.mask.fmt(f)
            }
        }
    }
}

pub fn solve(target: &TargetDate) -> SolverDecision {
    let winning_mask = if let Some(mask) = target.winning_mask() {
        mask
    } else {
        return SolverDecision::Impossible;
    };

    let mut stats = SolverStats { boards: 0, steps: 0, start_at: Instant::now() };
    loop {
        let state = GameState::default();
        stats.steps = 0;
        match solve_step(state, winning_mask, &mut stats) {
            s @ SolverDecision::Solved(_) | s @ SolverDecision::Impossible => return s,
            SolverDecision::Tired => {}
        }
    }
}

struct SolverStats {
    steps: usize,
    boards: usize,
    start_at: Instant,
}

fn solve_step(state: GameState, winning_mask: BoardMask, stats: &mut SolverStats) -> SolverDecision {
    stats.steps += 1;
    stats.boards += 1;
    let mask = state.mask();
    if mask == winning_mask {
        return SolverDecision::Solved(Solution {
            mask: state.tagged_mask(winning_mask),
            boards: stats.boards,
            steps: stats.steps,
            duration: Instant::now() - stats.start_at
        });
    }

    if stats.steps == 50 {
        return SolverDecision::Tired;
    }

    for piece_idx in state.random_available_piece_idxes() {
        for placement in Placement::random_iter(piece_idx) {
            let mut next_state = state;
            if next_state.place_piece(piece_idx, Some(placement), winning_mask) {
                if next_state.contains_no_islands(winning_mask) {
                    match solve_step(next_state, winning_mask, stats) {
                        s @ SolverDecision::Solved(_) | s @ SolverDecision::Tired => return s,
                        SolverDecision::Impossible => {}
                    }
                }
            }
        }
    }

    SolverDecision::Impossible
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TargetDate {
    pub month: Month,
    pub day_of_week: Weekday,
    pub day_of_month: i8,
}

impl TargetDate {
    pub fn winning_mask(&self) -> Option<BoardMask> {
        if self.day_of_month > 31 || self.day_of_month < 1 {
            return None;
        }

        let mut out = BoardMask::filled();

        let mut has_month = false;
        let mut has_day_of_week = false;
        let mut has_day = false;

        for y in 0..PUZZLE_HEIGHT {
            for x in 0..PUZZLE_WIDTH {
                match BOARD_LABELS[y][x] {
                    BoardLabel::MonthLabel(month) if month == self.month => {
                        if has_month {
                            panic!("duplicate month label for {:?}", month);
                        }
                        out.set_covered(x, y, false);
                        has_month = true;
                    }
                    BoardLabel::DayOfWeekLabel(weekday) if weekday == self.day_of_week => {
                        if has_day_of_week {
                            panic!("duplicate weekday label for {:?}", weekday);
                        }
                        out.set_covered(x, y, false);
                        has_day_of_week = true;
                    }
                    BoardLabel::DayLabel(day) if day == self.day_of_month => {
                        if has_day {
                            panic!("duplicate day label for {}", day);
                        }
                        out.set_covered(x, y, false);
                        has_day = true;
                    }
                    _ => {}
                }
            }
        }

        if has_month && has_day_of_week && has_day {
            Some(out)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl Month {
    fn next(&self) -> Option<Month> {
        use Month as M;
        Some(match self {
            M::January => M::February,
            M::February => M::March,
            M::March => M::April,
            M::April => M::May,
            M::May => M::June,
            M::June => M::July,
            M::July => M::August,
            M::August => M::September,
            M::September => M::October,
            M::October => M::November,
            M::November => M::December,
            M::December => return None,
        })
    }

    fn next_day(&self, day: i8, leap_year: bool) -> Option<i8> {
        if day >= self.number_days(leap_year) {
            None
        } else {
            Some(day + 1)
        }
    }

    fn number_days(&self, leap_year: bool) -> i8 {
        use Month as M;
        match self {
            M::January => 31,
            M::February if leap_year => 29,
            M::February => 28,
            M::March => 31,
            M::April => 30,
            M::May => 31,
            M::June => 30,
            M::July => 31,
            M::August => 31,
            M::September => 30,
            M::October => 31,
            M::November => 30,
            M::December => 31,
        }
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Weekday {
    fn next(&self) -> Weekday {
        use Weekday as WD;
        match self {
            WD::Sunday => WD::Monday,
            WD::Monday => WD::Tuesday,
            WD::Tuesday => WD::Wednesday,
            WD::Wednesday => WD::Thursday,
            WD::Thursday => WD::Friday,
            WD::Friday => WD::Saturday,
            WD::Saturday => WD::Sunday,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum BoardLabel {
    MonthLabel(Month),
    DayLabel(i8),
    DayOfWeekLabel(Weekday),
    Unlabeled,
}

pub const BOARD_LABELS: [[BoardLabel; PUZZLE_WIDTH]; PUZZLE_HEIGHT] = {
    use {Month as M, Weekday as WD, BoardLabel::{MonthLabel as ML, DayLabel as DL, DayOfWeekLabel as WL, Unlabeled}};
    [
        [ML(M::January), ML(M::February), ML(M::March), ML(M::April), ML(M::May), ML(M::June)],
        [ML(M::July), ML(M::August), ML(M::September), ML(M::October), ML(M::November), ML(M::December)],
        [DL(1), DL(2), DL(3), DL(4), DL(5), DL(6)],
        [DL(7), DL(8), DL(9), DL(10), DL(11), DL(12)],
        [DL(13), DL(14), DL(15), DL(16), DL(17), DL(18)],
        [DL(19), DL(20), DL(21), DL(22), DL(23), DL(24)],
        [DL(25), DL(26), DL(27), DL(28), DL(29), DL(30)],
        [DL(31), Unlabeled, Unlabeled, WL(WD::Monday), WL(WD::Tuesday), WL(WD::Wednesday)],
        [Unlabeled, Unlabeled, WL(WD::Thursday), WL(WD::Friday), WL(WD::Saturday), WL(WD::Sunday)]
    ]
};

pub struct TargetDateIter {
    current: Option<TargetDate>,
    leap_year: bool,
}

impl Iterator for TargetDateIter {
    type Item = TargetDate;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current {
            self.advance();
            Some(current)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(if self.leap_year { 366 } else { 365 }))
    }
}

impl TargetDateIter {
    pub fn create(start_at: TargetDate, leap_year: bool) -> Self {
        Self { current: Some(start_at), leap_year }
    }

    fn advance(&mut self) {
        let current = if let Some(current) = self.current { current } else { return; };
        let next_weekday = current.day_of_week.next();
        self.current = if let Some(next_day) = current.month.next_day(current.day_of_month, self.leap_year) {
            let mut next = current;
            next.day_of_month = next_day;
            next.day_of_week = next_weekday;
            Some(next)
        } else if let Some(next_month) = current.month.next() {
            Some(TargetDate{month: next_month, day_of_month: 1, day_of_week: next_weekday})
        } else {
            None
        };
    }
}