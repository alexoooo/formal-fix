use matching::{DuplicateOrderId, Order, OrderBook, OrderId, PriceMicros, Side, Trade};

#[test]
fn crossing_buy_trades_at_the_resting_sell_price() {
    let mut book = OrderBook::default();

    let sell = Order {
        id: OrderId(101),
        side: Side::Sell,
        limit: PriceMicros(10_000_001),
        quantity: 50,
    };
    let buy = Order {
        id: OrderId(102),
        side: Side::Buy,
        limit: PriceMicros(10_005_000),
        quantity: 30,
    };

    assert!(book.submit_limit(sell).unwrap().is_empty());
    let trades = book.submit_limit(buy).unwrap();
    assert_eq!(
        trades,
        [Trade {
            resting_order_id: OrderId(101),
            incoming_order_id: OrderId(102),
            price_micros: PriceMicros(10_000_001),
            quantity: 30,
        }]
    );
}

#[test]
fn reused_ids_are_rejected_before_matching_even_after_a_fill() {
    let mut book = OrderBook::default();
    let order = |id, side| Order {
        id: OrderId(id),
        side,
        limit: PriceMicros(10_000_001),
        quantity: 1,
    };
    book.submit_limit(order(1, Side::Sell)).unwrap();
    assert_eq!(
        book.submit_limit(order(1, Side::Buy)),
        Err(DuplicateOrderId(OrderId(1)))
    );
    let trades = book.submit_limit(order(2, Side::Buy)).unwrap();
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].resting_order_id, OrderId(1));
    assert_eq!(
        book.submit_limit(order(1, Side::Sell)),
        Err(DuplicateOrderId(OrderId(1)))
    );
}
