mod solver;
mod macros;

use solver::*;
use rayon::prelude::*;

pub fn main() {
    solve(TargetDate{month: Month::January, day_of_month: 1, day_of_week: Weekday::Monday})
        .sample(10)
        .for_each(|msg| println!("{}", msg));
    // TargetDateIter::create(TargetDate{month: Month::January, day_of_month: 1, day_of_week: Weekday::Monday}, true)
    //     .par_bridge()
    //     .for_each(|target_date| println!("FOR {:?} :: \n{}\n", target_date, solve(&target_date).unwrap().mask));
}