use super::prelude::*;

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

    pub fn next(&self, leap_year: bool) -> Option<Self> {
        let next_weekday = self.day_of_week.next();
        if let Some(next_day) = self.month.next_day(self.day_of_month, leap_year) {
            let mut next = *self;
            next.day_of_month = next_day;
            next.day_of_week = next_weekday;
            Some(next)
        } else { self.month.next().map(|next_month| TargetDate {
                month: next_month,
                day_of_month: 1,
                day_of_week: next_weekday,
            }) }
    }
}

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
        Self {
            current: Some(start_at),
            leap_year,
        }
    }

    fn advance(&mut self) {
        let current = if let Some(current) = self.current {
            current
        } else {
            return;
        };
        self.current = current.next(self.leap_year);
    }
}
