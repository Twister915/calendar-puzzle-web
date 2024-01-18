mod solver;

use solver::*;
use rayon::prelude::*;

pub fn main() {
    TargetDateIter::create(TargetDate{month: Month::January, day_of_month: 1, day_of_week: Weekday::Monday}, true)
        .par_bridge()
        .for_each(|target_date| println!("FOR {:?} :: {}\n", target_date, solve(&target_date)));
}