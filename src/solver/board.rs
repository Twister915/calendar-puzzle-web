use super::prelude::*;

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

impl Month {
    pub fn next(&self) -> Option<Month> {
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

    pub fn next_day(&self, day: i8, leap_year: bool) -> Option<i8> {
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

impl Weekday {
    pub fn next(&self) -> Weekday {
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