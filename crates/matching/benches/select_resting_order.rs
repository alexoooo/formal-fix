use std::hint::black_box;

use divan::Bencher;
use matching::{OrderId, PriceMicros, RestingOrder, Side, select_resting_order};

const DEPTHS: &[usize] = &[16, 256, 4096];
const LIMIT: PriceMicros = PriceMicros(10_000_000);

fn main() {
    divan::main();
}

#[divan::bench(args = DEPTHS, min_time = 0.25)]
fn buy(bencher: Bencher, depth: usize) {
    bench_selection(bencher, Side::Buy, depth);
}

#[divan::bench(args = DEPTHS, min_time = 0.25)]
fn sell(bencher: Bencher, depth: usize) {
    bench_selection(bencher, Side::Sell, depth);
}

fn bench_selection(bencher: Bencher, side: Side, depth: usize) {
    // Cycle through 11 price levels around the limit, with every eighth order exhausted.
    let book: Vec<_> = (0..depth)
        .map(|i| RestingOrder {
            id: OrderId(i as u64),
            price_micros: PriceMicros(LIMIT.0 - 5 + (i * 7 % 11) as u64),
            remaining: if i % 8 == 0 { 0 } else { 100 },
        })
        .collect();

    bencher.bench_local(|| {
        select_resting_order(
            black_box(side),
            black_box(LIMIT),
            black_box(book.as_slice()),
        )
    });
}
