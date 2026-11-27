use crate::{Order, OrderId, PriceMicros, RestingOrder, Side, select_resting_order};
use std::collections::HashSet;
use vstd::prelude::*;

verus! {

#[derive(Debug, PartialEq, Eq)]
pub struct Trade {
    pub resting_order_id: OrderId,
    pub incoming_order_id: OrderId,
    pub price_micros: PriceMicros,
    pub quantity: u64,
}

pub struct OrderBook {
    bids: Vec<RestingOrder>,
    asks: Vec<RestingOrder>,
    accepted_ids: HashSet<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DuplicateOrderId(pub OrderId);

impl Default for OrderBook {
    fn default() -> (book: Self)
        ensures book.well_formed(), book.accepted_ids() == Set::<u64>::empty(),
    {
        Self { bids: Vec::new(), asks: Vec::new(), accepted_ids: HashSet::new() }
    }
}

impl OrderBook {
    pub closed spec fn well_formed(&self) -> bool {
        vstd::std_specs::hash::hash_set_wf(&self.accepted_ids)
    }

    pub closed spec fn accepted_ids(&self) -> Set<u64> {
        self.accepted_ids@
    }

    /// Executes against resting orders at their prices, then books any unfilled quantity.
    /// Each ID can be submitted once during the lifetime of this book.
    pub fn submit_limit(&mut self, order: Order) -> (result: Result<Vec<Trade>, DuplicateOrderId>)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            result.is_err() == old(self).accepted_ids().contains(order.id.0),
            final(self).accepted_ids() == old(self).accepted_ids().insert(order.id.0),
    {
        broadcast use vstd::std_specs::hash::group_hash_axioms;
        if !self.accepted_ids.insert(order.id.0) {
            return Err(DuplicateOrderId(order.id));
        }
        let (own, opposing) = match order.side {
            Side::Buy => (&mut self.bids, &mut self.asks),
            Side::Sell => (&mut self.asks, &mut self.bids),
        };
        let mut trades = Vec::new();
        let mut remaining = order.quantity;
        while remaining > 0
            decreases remaining,
        {
            let Some(index) = select_resting_order(order.side, order.limit, opposing.as_slice()) else {
                own.push(RestingOrder { id: order.id, price_micros: order.limit, remaining });
                break;
            };
            let mut resting = opposing[index];
            let filled = remaining.min(resting.remaining);
            trades.push(Trade {
                resting_order_id: resting.id,
                incoming_order_id: order.id,
                price_micros: resting.price_micros,
                quantity: filled,
            });
            remaining -= filled;
            resting.remaining -= filled;
            if resting.is_empty() {
                opposing.remove(index);
            } else {
                opposing.set(index, resting);
            }
        }
        Ok(trades)
    }
}

}
