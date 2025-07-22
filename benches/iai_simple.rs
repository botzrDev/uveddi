//! Simple iai-callgrind benchmarks for deterministic CI performance testing

use iai_callgrind::{library_benchmark_group, main};

fn factorial(n: u64) -> u64 {
    match n {
        0 => 1,
        n => n * factorial(n - 1),
    }
}

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

library_benchmark_group!(
    name = simple_group;
    benchmarks = factorial, fibonacci
);

main!(library_benchmark_groups = simple_group);
