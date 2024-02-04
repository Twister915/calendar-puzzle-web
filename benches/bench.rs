use calendar_puzzle_web::solver::{solve, Month, SolverMsg, TargetDate, Weekday};
use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion, Throughput};
use std::hint::black_box;

fn bench_solver(group: &mut BenchmarkGroup<WallTime>, target_date: TargetDate) {
    let name = format!(
        "{}_{}_{}",
        target_date.month, target_date.day_of_month, target_date.day_of_week,
    );

    group.throughput(Throughput::Elements(run_solver(target_date) as u64));
    group.bench_function(&name, |b| b.iter(|| run_solver(target_date)));
}

fn run_solver(target_date: TargetDate) -> usize {
    let mut steps = solve(target_date);
    let mut last_step = steps.next().unwrap();
    let mut count = 0;
    for step in steps {
        black_box(&step);
        last_step = step;
        count += 1;
    }
    assert!(matches!(last_step, SolverMsg::Solved(_)));
    count
}

fn solver(c: &mut Criterion) {
    let mut group = c.benchmark_group("solve_board");
    bench_solver(
        &mut group,
        TargetDate {
            month: Month::January,
            day_of_week: Weekday::Monday,
            day_of_month: 1,
        },
    );
    // The solution with the most steps
    bench_solver(
        &mut group,
        TargetDate {
            month: Month::January,
            day_of_week: Weekday::Thursday,
            day_of_month: 24,
        },
    );
    bench_solver(
        &mut group,
        TargetDate {
            month: Month::February,
            day_of_week: Weekday::Saturday,
            day_of_month: 29,
        },
    );
    bench_solver(
        &mut group,
        TargetDate {
            month: Month::June,
            day_of_week: Weekday::Wednesday,
            day_of_month: 17,
        },
    );
    bench_solver(
        &mut group,
        TargetDate {
            month: Month::December,
            day_of_week: Weekday::Friday,
            day_of_month: 31,
        },
    );

    group.finish();
}

criterion_group!(benches, solver);
criterion_main!(benches);
