pub mod macros;
pub mod solver;
pub mod web;

#[cfg(test)]
mod tests {
    use crate::solver::*;

    #[test]
    fn test_solve() {
        for step in solve(TargetDate {
            month: Month::January,
            day_of_month: 19,
            day_of_week: Weekday::Friday,
        }) {
            println!("{}", step);
        }
    }
}
