//! Limit order matching for a single instrument.

mod book;
pub use book::{DuplicateOrderId, OrderBook, Trade};

use vstd::prelude::*;

verus! {

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OrderId(pub u64);

/// A USD price in millionths of a dollar. One dollar is `PriceMicros(1_000_000)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PriceMicros(pub u64);

#[derive(Debug)]
pub struct Order {
    pub id: OrderId,
    pub side: Side,
    pub limit: PriceMicros,
    pub quantity: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct RestingOrder {
    pub id: OrderId,
    pub price_micros: PriceMicros,
    pub remaining: u64,
}

impl RestingOrder {
    pub open spec fn spec_is_empty(&self) -> bool {
        self.remaining == 0
    }

    /// Returns true when no quantity remains to execute.
    #[verifier::when_used_as_spec(spec_is_empty)]
    pub fn is_empty(&self) -> bool
        returns self.spec_is_empty(),
    {
        self.remaining == 0
    }
}

/// Compares prices from the incoming order's perspective.
pub open spec fn price_at_least_as_good(side: Side, price: PriceMicros, reference: PriceMicros) -> bool {
    match side {
        Side::Buy => price.0 <= reference.0,
        Side::Sell => price.0 >= reference.0,
    }
}

impl Side {
    fn price_at_least_as_good(self, price: PriceMicros, reference: PriceMicros) -> bool
        returns price_at_least_as_good(self, price, reference),
    {
        match self {
            Self::Buy => price.0 <= reference.0,
            Self::Sell => price.0 >= reference.0,
        }
    }
}

pub open spec fn eligible(side: Side, limit: PriceMicros, order: RestingOrder) -> bool {
    !order.is_empty() && price_at_least_as_good(side, order.price_micros, limit)
}

/// Selects the earliest eligible index at the best price in the first `end` orders,
/// or requires `None` if none is eligible.
pub open spec fn best_in_prefix(
    side: Side, limit: PriceMicros, book: Seq<RestingOrder>, end: int, best: Option<usize>,
) -> bool {
    match best {
        None => forall|j: int| 0 <= j < end ==> !eligible(side, limit, #[trigger] book[j]),
        Some(index) => {
            let selected = book[index as int];
            &&& index < end
            &&& eligible(side, limit, selected)
            &&& forall|j: int| 0 <= j < end && eligible(side, limit, #[trigger] book[j])
                ==> price_at_least_as_good(side, selected.price_micros, book[j].price_micros)
            &&& forall|j: int| 0 <= j < index && eligible(side, limit, #[trigger] book[j])
                ==> book[j].price_micros != selected.price_micros
        },
    }
}

/// Returns the first index at the best eligible price, or `None` if no live order crosses.
/// The caller supplies the full opposing side of one instrument's book.
pub fn select_resting_order(side: Side, limit: PriceMicros, book: &[RestingOrder]) -> (best: Option<usize>)
    ensures best_in_prefix(side, limit, book@, book.len() as int, best),
{
    let mut best: Option<usize> = None;
    for i in 0..book.len()
        invariant best_in_prefix(side, limit, book@, i as int, best),
    {
        let order = book[i];
        if order.is_empty() {
            continue;
        }

        // Accept the limit price, but preserve the first order on a price tie.
        let select = match best {
            None => side.price_at_least_as_good(order.price_micros, limit),
            Some(previous) => {
                let previous_price = book[previous].price_micros;
                order.price_micros.0 != previous_price.0
                    && side.price_at_least_as_good(order.price_micros, previous_price)
            },
        };
        if select {
            best = Some(i);
        }
    }
    best
}

}
