//! Venue-local price selection for a single instrument.

use vstd::prelude::*;

verus! {

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Clone, Copy, Debug)]
pub struct RestingOrder {
    pub price_ticks: u64,
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
pub open spec fn price_at_least_as_good(side: Side, price: u64, reference: u64) -> bool {
    match side {
        Side::Buy => price <= reference,
        Side::Sell => price >= reference,
    }
}

impl Side {
    fn price_at_least_as_good(self, price: u64, reference: u64) -> bool
        returns price_at_least_as_good(self, price, reference),
    {
        match self {
            Self::Buy => price <= reference,
            Self::Sell => price >= reference,
        }
    }
}

pub open spec fn eligible(side: Side, limit: u64, order: RestingOrder) -> bool {
    !order.is_empty() && price_at_least_as_good(side, order.price_ticks, limit)
}

/// Selects the earliest eligible index at the best price in the first `end` orders,
/// or requires `None` if none is eligible.
pub open spec fn best_in_prefix(
    side: Side, limit: u64, book: Seq<RestingOrder>, end: int, best: Option<usize>,
) -> bool {
    match best {
        None => forall|j: int| 0 <= j < end ==> !eligible(side, limit, #[trigger] book[j]),
        Some(index) => {
            let selected = book[index as int];
            &&& index < end
            &&& eligible(side, limit, selected)
            &&& forall|j: int| 0 <= j < end && eligible(side, limit, #[trigger] book[j])
                ==> price_at_least_as_good(side, selected.price_ticks, book[j].price_ticks)
            &&& forall|j: int| 0 <= j < index && eligible(side, limit, #[trigger] book[j])
                ==> book[j].price_ticks != selected.price_ticks
        },
    }
}

/// Returns the first index at the best eligible price, or `None` if no live order crosses.
/// The caller supplies the full opposing side of one instrument's book.
pub fn select_resting_order(side: Side, limit: u64, book: &[RestingOrder]) -> (best: Option<usize>)
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
            None => side.price_at_least_as_good(order.price_ticks, limit),
            Some(previous) => {
                let previous_price = book[previous].price_ticks;
                order.price_ticks != previous_price
                    && side.price_at_least_as_good(order.price_ticks, previous_price)
            },
        };
        if select {
            best = Some(i);
        }
    }
    best
}

}
