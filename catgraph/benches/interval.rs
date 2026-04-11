//! Interval algebra benchmarks.
//!
//! Three benchmark groups:
//! 1. `interval_compose` — chaining contiguous `DiscreteInterval`s via `.then()`
//! 2. `parallel_tensor` — `ParallelIntervals::tensor` combining single-branch intervals
//! 3. `parallel_direct_sum` — `ParallelIntervals::direct_sum` at same counts

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use catgraph::interval::{DiscreteInterval, ParallelIntervals};

fn bench_interval_compose(c: &mut Criterion) {
    let mut group = c.benchmark_group("interval_compose");

    for chain_len in [4, 16, 64, 256] {
        group.bench_with_input(
            BenchmarkId::from_parameter(chain_len),
            &chain_len,
            |bencher, &n| {
                // Build a chain of contiguous intervals: [0,1], [1,2], [2,3], ...
                let intervals: Vec<DiscreteInterval> =
                    (0..n).map(|i| DiscreteInterval::new(i, i + 1)).collect();

                bencher.iter(|| {
                    let mut acc = intervals[0];
                    for iv in &intervals[1..] {
                        acc = acc.then(*iv).unwrap();
                    }
                    acc
                });
            },
        );
    }

    group.finish();
}

fn bench_parallel_tensor(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_tensor");

    for count in [2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &count,
            |bencher, &n| {
                // Build n single-branch ParallelIntervals to tensor together.
                let branches: Vec<ParallelIntervals> = (0..n)
                    .map(|i| ParallelIntervals::from_branch(DiscreteInterval::new(i * 10, i * 10 + 5)))
                    .collect();

                bencher.iter(|| {
                    let mut acc = branches[0].clone();
                    for pi in &branches[1..] {
                        acc = acc.tensor(pi.clone());
                    }
                    acc
                });
            },
        );
    }

    group.finish();
}

fn bench_parallel_direct_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_direct_sum");

    for count in [2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &count,
            |bencher, &n| {
                let branches: Vec<ParallelIntervals> = (0..n)
                    .map(|i| ParallelIntervals::from_branch(DiscreteInterval::new(i * 10, i * 10 + 5)))
                    .collect();

                bencher.iter(|| {
                    let mut acc = branches[0].clone();
                    for pi in &branches[1..] {
                        acc = acc.direct_sum(pi.clone());
                    }
                    acc
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_interval_compose,
    bench_parallel_tensor,
    bench_parallel_direct_sum
);
criterion_main!(benches);
