//! # From loops to chains (reference solution)
//!
//! - Each loop was a named method in disguise: a sum (`needless_range_loop`
//!   pointed at it), a counter (`explicit_counter_loop`: `enumerate`, or
//!   `zip` with `1..` to count from 1), `find`, `filter_map`.
//! - `map(..).flatten()` is `flat_map`, `filter(..).next()` is `find`,
//!   `skip(n).next()` is `nth(n)`. Same laziness, fewer moving parts.
//! - A `map` that hands back its input unchanged is really `inspect`
//!   (`manual_inspect`). Better still, keep side effects out of chains: the
//!   audit could be built separately, or in a plain `for` loop.
//! - Option's `as_deref()` turns `&Option<String>` into `Option<&str>`,
//!   which is exactly what `filter_map` wants.

#[derive(Debug, PartialEq)]
struct Line {
    sku: String,
    qty: u32,
    unit_cents: u64,
}

#[derive(Debug, PartialEq)]
struct Order {
    id: u32,
    lines: Vec<Line>,
    coupon: Option<String>,
    shipped: bool,
}

/// The order's total in cents.
fn total_cents(order: &Order) -> u64 {
    order
        .lines
        .iter()
        .map(|line| u64::from(line.qty) * line.unit_cents)
        .sum()
}

/// Receipt lines like "1. 2 x mug", numbered from 1.
fn receipt(order: &Order) -> Vec<String> {
    (1..)
        .zip(&order.lines)
        .map(|(n, line)| format!("{n}. {} x {}", line.qty, line.sku))
        .collect()
}

/// The first order that hasn't shipped yet.
fn first_unshipped(orders: &[Order]) -> Option<&Order> {
    orders.iter().find(|o| !o.shipped)
}

/// Every coupon code used, in order.
fn coupons(orders: &[Order]) -> Vec<&str> {
    orders.iter().filter_map(|o| o.coupon.as_deref()).collect()
}

/// Every SKU on every order, in order, repeats included.
fn all_skus(orders: &[Order]) -> Vec<&str> {
    orders
        .iter()
        .flat_map(|o| &o.lines)
        .map(|line| line.sku.as_str())
        .collect()
}

/// The `n`-th order (counting from 0) that used a coupon.
fn nth_with_coupon(orders: &[Order], n: usize) -> Option<&Order> {
    orders.iter().filter(|o| o.coupon.is_some()).nth(n)
}

/// The total of all orders. Records "order <id>" in `audit` for each order,
/// in order.
fn grand_total(orders: &[Order], audit: &mut Vec<String>) -> u64 {
    orders
        .iter()
        .inspect(|o| audit.push(format!("order {}", o.id)))
        .map(total_cents)
        .sum()
}

fn line(sku: &str, qty: u32, unit_cents: u64) -> Line {
    Line {
        sku: sku.to_string(),
        qty,
        unit_cents,
    }
}

fn sample_orders() -> Vec<Order> {
    vec![
        Order {
            id: 1,
            lines: vec![line("mug", 2, 850), line("tea", 1, 1200)],
            coupon: Some("WELCOME".to_string()),
            shipped: true,
        },
        Order {
            id: 2,
            lines: vec![line("kettle", 1, 4500)],
            coupon: None,
            shipped: false,
        },
        Order {
            id: 3,
            lines: vec![line("tea", 3, 1200), line("mug", 1, 850)],
            coupon: Some("SPRING".to_string()),
            shipped: false,
        },
    ]
}

fn main() {
    let orders = sample_orders();
    let mut audit = Vec::new();
    println!("grand total: {}", grand_total(&orders, &mut audit));
    println!("audit: {audit:?}");
    println!("receipt: {:?}", receipt(&orders[0]));
    println!(
        "first unshipped: {:?}",
        first_unshipped(&orders).map(|o| o.id)
    );
    println!("coupons: {:?}", coupons(&orders));
    println!("skus: {:?}", all_skus(&orders));
    println!(
        "second coupon order: {:?}",
        nth_with_coupon(&orders, 1).map(|o| o.id)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn totals() {
        let orders = sample_orders();
        assert_eq!(total_cents(&orders[0]), 2900);
        assert_eq!(total_cents(&orders[2]), 4450);
        let mut audit = Vec::new();
        assert_eq!(grand_total(&orders, &mut audit), 11850);
        assert_eq!(audit, ["order 1", "order 2", "order 3"]);
        assert_eq!(grand_total(&[], &mut audit), 0);
        assert_eq!(audit.len(), 3);
    }

    #[test]
    fn numbers_receipt_lines() {
        let orders = sample_orders();
        assert_eq!(receipt(&orders[2]), ["1. 3 x tea", "2. 1 x mug"]);
    }

    #[test]
    fn finds_orders() {
        let orders = sample_orders();
        assert_eq!(first_unshipped(&orders).map(|o| o.id), Some(2));
        assert_eq!(first_unshipped(&orders[..1]), None);
        assert_eq!(nth_with_coupon(&orders, 0).map(|o| o.id), Some(1));
        assert_eq!(nth_with_coupon(&orders, 1).map(|o| o.id), Some(3));
        assert_eq!(nth_with_coupon(&orders, 2), None);
    }

    #[test]
    fn lists_coupons_and_skus() {
        let orders = sample_orders();
        assert_eq!(coupons(&orders), ["WELCOME", "SPRING"]);
        assert_eq!(all_skus(&orders), ["mug", "tea", "kettle", "tea", "mug"]);
        assert!(coupons(&[]).is_empty());
    }
}
