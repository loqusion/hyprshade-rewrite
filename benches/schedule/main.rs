//! This file benchmarks implementations of `Schedule`, which finds an event matching a given time.
//!
//! An event has a start time, an end time, and some sort of payload which is used to "dispatch"
//! the event. In practice, Hyprshade (via systemd-timer activation) uses this to activate shaders
//! at specific times in the day based on a user-provided schedule.
//!
//! # Implementations
//!
//! Each implementation involves two operations: construction and search. Construction happens only
//! once, and search... also happens only once, so it's not necessarily worth optimizing search at
//! the expense of construction.
//!
//! * `impl_binary_search.rs`
//!     - Construction *O*(*n*log(*n*)): Collect and sort into a vector of `(range, event)` tuples
//!     - Search *O*(log(*n*)): Find an event using binary search
//! * `impl_btree.rs`
//!     - Construction *O*(*n*log(*n*)): Collect into a `BTreeMap<Time, Option<Event>>`\*
//!     - Search *O*(log(*n*)): Return last element of `BTreeMap::range()`
//! * `impl_linear.rs`
//!     - Construction *O*(*n*): Collect into a vector of `(range, event)` tuples
//!     - Search *O*(*n*): Linear search for an event matching the time
//!
//! \* Each entry indicates that the `Option<Event>` will occur between the given `Time`
//! (inclusive) and the next entry's `Time` (exclusive).
//!
//! Time complexity analysis considers average case.
//!
//! ---
//!
//! Sidenote: There's no real reason to do all of this since most users' schedules won't exceed 2
//! or 3 items. This is just for fun.

mod impl_binary_search;
mod impl_btree;
mod impl_linear;
mod util;

use std::str::FromStr;

use self::{
    impl_binary_search::Schedule as ScheduleBinarySearch, impl_btree::Schedule as ScheduleBTree,
    impl_linear::Schedule as ScheduleLinear, util::Event,
};
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};

fn benchmark_schedule(c: &mut Criterion) {
    // NOTE: Each half-open range must not overlap with any other.
    // Also, be sure that the last event has no end time so we can benchmark worst-case.
    let source_input = vec![
        Event::from(("06:30:00", Some("07:00:00"))),
        Event::from(("14:00:00", None)),
        Event::from(("03:00:00", Some("04:00:00"))),
        Event::from(("05:00:00", None)),
        Event::from(("09:00:00", Some("10:00:00"))),
        Event::from(("06:00:00", Some("06:30:00"))),
        Event::from(("11:00:00", None)),
        Event::from(("18:30:00", Some("19:00:00"))),
        Event::from(("08:00:00", None)),
        Event::from(("21:00:00", Some("22:00:00"))),
        Event::from(("12:30:00", Some("13:00:00"))),
        Event::from(("18:00:00", Some("18:30:00"))),
        Event::from(("02:00:00", None)),
        Event::from(("17:00:00", None)),
        Event::from(("12:00:00", Some("12:30:00"))),
        Event::from(("15:00:00", Some("16:00:00"))),
        Event::from(("20:00:00", None)),
        Event::from(("23:00:00", None)),
        Event::from(("16:00:00", Some("16:01:00"))),
        Event::from(("04:00:00", Some("04:01:00"))),
        Event::from(("22:00:00", Some("22:01:00"))),
        Event::from(("00:30:00", Some("01:00:00"))),
        Event::from(("00:01:00", Some("00:30:00"))),
        Event::from(("10:00:00", Some("10:01:00"))),
    ];

    let mut construction_group = c.benchmark_group("Schedule/Construction");

    for (name, input) in [
        ("1", &source_input[..1]),
        ("10", &source_input[..10]),
        ("many", &source_input[..]),
    ] {
        construction_group.bench_with_input(BenchmarkId::new("BTreeMap", name), input, |b, i| {
            b.iter(|| ScheduleBTree::from_iter(i));
        });
        construction_group.bench_with_input(
            BenchmarkId::new("Binary Search", name),
            input,
            |b, i| {
                b.iter(|| ScheduleBinarySearch::from_iter(i));
            },
        );
        construction_group.bench_with_input(BenchmarkId::new("Linear", name), input, |b, i| {
            b.iter(|| ScheduleLinear::from_iter(i));
        });
    }

    construction_group.finish();

    let mut search_group = c.benchmark_group("Schedule/Search");

    for (name, input) in [
        ("1", source_input.iter().take(1)),
        ("10", source_input.iter().take(10)),
        ("many", source_input.iter().take(source_input.len())),
    ] {
        let time = chrono::NaiveTime::from_str("05:00:00").unwrap();
        search_group.bench_with_input(BenchmarkId::new("BTreeMap", name), &time, |b, i| {
            b.iter_batched(
                || ScheduleBTree::from_iter(input.clone()),
                |schedule| schedule.get(i),
                BatchSize::SmallInput,
            );
        });
        search_group.bench_with_input(BenchmarkId::new("Binary Search", name), &time, |b, i| {
            b.iter_batched(
                || ScheduleBinarySearch::from_iter(input.clone()),
                |schedule| schedule.get(i),
                BatchSize::SmallInput,
            );
        });
        search_group.bench_with_input(BenchmarkId::new("Linear", name), &time, |b, i| {
            b.iter_batched(
                || ScheduleLinear::from_iter(input.clone()),
                |schedule| schedule.get(i),
                BatchSize::SmallInput,
            );
        });
    }

    {
        search_group.bench_with_input(
            BenchmarkId::new("BTreeMap", "worst"),
            &chrono::NaiveTime::from_str("00:00:00").unwrap(),
            |b, i| {
                b.iter_batched(
                    || ScheduleBTree::from_iter(&source_input),
                    |schedule| schedule.get(i),
                    BatchSize::SmallInput,
                )
            },
        );
        search_group.bench_with_input(
            BenchmarkId::new("Binary Search", "worst"),
            &chrono::NaiveTime::from_str("00:00:00").unwrap(),
            |b, i| {
                b.iter_batched(
                    || ScheduleBinarySearch::from_iter(&source_input),
                    |schedule| schedule.get(i),
                    BatchSize::SmallInput,
                )
            },
        );
        search_group.bench_with_input(
            BenchmarkId::new("Linear", "worst"),
            &chrono::NaiveTime::from_str("23:59:59").unwrap(),
            |b, i| {
                b.iter_batched(
                    || ScheduleLinear::from_iter(&source_input),
                    |schedule| schedule.get(i),
                    BatchSize::SmallInput,
                )
            },
        );
    }

    search_group.finish();
}

criterion_group!(benches, benchmark_schedule);
criterion_main!(benches);
