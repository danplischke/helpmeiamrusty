//! # Checkpoint: an order's lifecycle (reference solution)
//!
//! - Matching on the pair `(state, event)` makes the transition table the
//!   code: one arm per allowed transition, and a final catch-all for the
//!   rest. A new state is E0004 in `State::name`, `lines` and `tracking`,
//!   a new event in `Event::name`. The catch-all in `apply` stays quiet, so
//!   review it by hand.
//! - In the tuple, `&mut self.state` binds by `&mut` (match ergonomics)
//!   while `event` is matched by value, so `tracking` is moved out of
//!   `Event::Ship` and `mem::take` moves `lines` out of the old state.
//! - Arms that only edit the draft `return Ok(())` early; arms that move on
//!   evaluate to the new state, which is assigned once at the end. Every
//!   error returns before that assignment, so a rejected event can't leave
//!   a half-updated order.
//! - The guard on the first `Place` arm handles the empty draft; the
//!   second `Place` arm takes the rest.
//! - `State::Draft { lines } | State::Placed { lines, .. } | ..` binds the
//!   same name from several variants. `summary` reuses `lines()` with a
//!   slice pattern. Neither needs a `_` arm.

use std::mem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cents(u64);

#[derive(Debug, PartialEq)]
struct Line {
    sku: String,
    qty: u32,
    unit_price: Cents,
}

#[derive(Debug, PartialEq)]
enum State {
    Draft {
        lines: Vec<Line>,
    },
    Placed {
        lines: Vec<Line>,
        total: Cents,
    },
    Shipped {
        lines: Vec<Line>,
        total: Cents,
        tracking: String,
    },
    Delivered {
        total: Cents,
        tracking: String,
    },
    Cancelled {
        refund: Cents,
    },
}

#[derive(Debug)]
enum Event {
    AddLine(Line),
    Place,
    Ship { tracking: String },
    Deliver,
    Cancel,
}

#[derive(Debug, PartialEq)]
enum OrderError {
    /// Placing a draft that has no lines.
    EmptyOrder,
    /// The event doesn't apply in the current state.
    InvalidTransition {
        state: &'static str,
        event: &'static str,
    },
}

impl State {
    fn name(&self) -> &'static str {
        match self {
            State::Draft { .. } => "draft",
            State::Placed { .. } => "placed",
            State::Shipped { .. } => "shipped",
            State::Delivered { .. } => "delivered",
            State::Cancelled { .. } => "cancelled",
        }
    }
}

impl Event {
    fn name(&self) -> &'static str {
        match self {
            Event::AddLine(_) => "add line",
            Event::Place => "place",
            Event::Ship { .. } => "ship",
            Event::Deliver => "deliver",
            Event::Cancel => "cancel",
        }
    }
}

/// The sum of quantity × unit price over all lines.
fn total(lines: &[Line]) -> Cents {
    Cents(
        lines
            .iter()
            .map(|line| u64::from(line.qty) * line.unit_price.0)
            .sum(),
    )
}

#[derive(Debug)]
struct Order {
    id: u32,
    state: State,
}

impl Order {
    fn new(id: u32) -> Self {
        Order {
            id,
            state: State::Draft { lines: Vec::new() },
        }
    }

    /// Applies an event:
    ///
    /// - `AddLine` (draft): appends the line, or adds its quantity to an
    ///   existing line with the same sku.
    /// - `Place` (draft): records the total. A draft without lines fails
    ///   with `EmptyOrder`.
    /// - `Ship` (placed): records the tracking number.
    /// - `Deliver` (shipped): keeps the total and tracking number, drops
    ///   the lines.
    /// - `Cancel` (draft or placed): refunds the total if it was placed,
    ///   nothing for a draft.
    ///
    /// Anything else is `InvalidTransition` with the names of the current
    /// state and the event. On any error the order is left unchanged.
    fn apply(&mut self, event: Event) -> Result<(), OrderError> {
        self.state = match (&mut self.state, event) {
            (State::Draft { lines }, Event::AddLine(line)) => {
                match lines.iter_mut().find(|existing| existing.sku == line.sku) {
                    Some(existing) => existing.qty += line.qty,
                    None => lines.push(line),
                }
                return Ok(());
            }
            (State::Draft { lines }, Event::Place) if lines.is_empty() => {
                return Err(OrderError::EmptyOrder);
            }
            (State::Draft { lines }, Event::Place) => State::Placed {
                total: total(lines),
                lines: mem::take(lines),
            },
            (State::Placed { lines, total }, Event::Ship { tracking }) => State::Shipped {
                lines: mem::take(lines),
                total: *total,
                tracking,
            },
            (
                State::Shipped {
                    total, tracking, ..
                },
                Event::Deliver,
            ) => State::Delivered {
                total: *total,
                tracking: mem::take(tracking),
            },
            (State::Draft { .. }, Event::Cancel) => State::Cancelled { refund: Cents(0) },
            (State::Placed { total, .. }, Event::Cancel) => State::Cancelled { refund: *total },
            (state, event) => {
                return Err(OrderError::InvalidTransition {
                    state: state.name(),
                    event: event.name(),
                });
            }
        };
        Ok(())
    }

    /// The lines of a draft, placed or shipped order; empty otherwise.
    fn lines(&self) -> &[Line] {
        match &self.state {
            State::Draft { lines } | State::Placed { lines, .. } | State::Shipped { lines, .. } => {
                lines
            }
            State::Delivered { .. } | State::Cancelled { .. } => &[],
        }
    }

    /// The tracking number once shipped, including after delivery.
    fn tracking(&self) -> Option<&str> {
        match &self.state {
            State::Shipped { tracking, .. } | State::Delivered { tracking, .. } => Some(tracking),
            State::Draft { .. } | State::Placed { .. } | State::Cancelled { .. } => None,
        }
    }

    /// Whether `Cancel` would be accepted right now.
    fn is_cancellable(&self) -> bool {
        matches!(self.state, State::Draft { .. } | State::Placed { .. })
    }

    /// The lines in brief: "no items", "3x bolt" or "3x bolt and 2 more".
    fn summary(&self) -> String {
        match self.lines() {
            [] => "no items".to_string(),
            [only] => format!("{}x {}", only.qty, only.sku),
            [first, rest @ ..] => {
                format!("{}x {} and {} more", first.qty, first.sku, rest.len())
            }
        }
    }
}

fn line(sku: &str, qty: u32, cents: u64) -> Line {
    Line {
        sku: sku.to_string(),
        qty,
        unit_price: Cents(cents),
    }
}

fn main() {
    let mut order = Order::new(1001);
    let events = [
        Event::Ship {
            tracking: "TRK-1".to_string(),
        },
        Event::AddLine(line("bolt", 3, 25)),
        Event::AddLine(line("nut", 2, 10)),
        Event::Place,
        Event::Ship {
            tracking: "TRK-1".to_string(),
        },
        Event::Cancel,
        Event::Deliver,
    ];
    for event in events {
        let name = event.name();
        match order.apply(event) {
            Ok(()) => println!("{name}: now {} ({})", order.state.name(), order.summary()),
            Err(err) => println!("{name}: {err:?}"),
        }
    }
    println!("{:?}", order.state);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> Order {
        let mut order = Order::new(7);
        order.apply(Event::AddLine(line("bolt", 3, 25))).unwrap();
        order.apply(Event::AddLine(line("nut", 2, 10))).unwrap();
        order
    }

    fn placed() -> Order {
        let mut order = draft();
        order.apply(Event::Place).unwrap();
        order
    }

    fn shipped() -> Order {
        let mut order = placed();
        let tracking = "TRK-77".to_string();
        order.apply(Event::Ship { tracking }).unwrap();
        order
    }

    fn invalid(state: &'static str, event: &'static str) -> Result<(), OrderError> {
        Err(OrderError::InvalidTransition { state, event })
    }

    #[test]
    fn drafts_collect_lines() {
        let mut order = draft();
        order.apply(Event::AddLine(line("bolt", 2, 25))).unwrap();
        assert_eq!(order.lines(), [line("bolt", 5, 25), line("nut", 2, 10)]);
        assert_eq!(order.tracking(), None);
    }

    #[test]
    fn placing_records_the_total() {
        let order = placed();
        assert_eq!(
            order.state,
            State::Placed {
                lines: vec![line("bolt", 3, 25), line("nut", 2, 10)],
                total: Cents(95),
            }
        );
    }

    #[test]
    fn empty_orders_cannot_be_placed() {
        let mut order = Order::new(1);
        assert_eq!(order.apply(Event::Place), Err(OrderError::EmptyOrder));
        assert_eq!(order.state, State::Draft { lines: vec![] });
    }

    #[test]
    fn ships_and_delivers() {
        let mut order = shipped();
        assert_eq!(order.tracking(), Some("TRK-77"));
        assert_eq!(order.lines().len(), 2);
        order.apply(Event::Deliver).unwrap();
        assert_eq!(
            order.state,
            State::Delivered {
                total: Cents(95),
                tracking: "TRK-77".to_string(),
            }
        );
        assert_eq!(order.tracking(), Some("TRK-77"));
        assert!(order.lines().is_empty());
    }

    #[test]
    fn cancelling_refunds_what_was_placed() {
        let mut order = draft();
        order.apply(Event::Cancel).unwrap();
        assert_eq!(order.state, State::Cancelled { refund: Cents(0) });

        let mut order = placed();
        order.apply(Event::Cancel).unwrap();
        assert_eq!(order.state, State::Cancelled { refund: Cents(95) });
    }

    #[test]
    fn invalid_events_change_nothing() {
        let mut order = Order::new(1);
        let ship = || Event::Ship {
            tracking: "TRK-1".to_string(),
        };
        assert_eq!(order.apply(ship()), invalid("draft", "ship"));
        assert_eq!(order.apply(Event::Deliver), invalid("draft", "deliver"));

        let mut order = placed();
        let extra = Event::AddLine(line("washer", 1, 5));
        assert_eq!(order.apply(extra), invalid("placed", "add line"));
        assert_eq!(order.apply(Event::Place), invalid("placed", "place"));
        assert_eq!(order.lines().len(), 2);

        let mut order = shipped();
        assert_eq!(order.apply(Event::Cancel), invalid("shipped", "cancel"));
        assert_eq!(order.apply(ship()), invalid("shipped", "ship"));
        assert_eq!(order.tracking(), Some("TRK-77"));

        order.apply(Event::Deliver).unwrap();
        assert_eq!(order.apply(Event::Deliver), invalid("delivered", "deliver"));

        let mut order = draft();
        order.apply(Event::Cancel).unwrap();
        assert_eq!(order.apply(Event::Place), invalid("cancelled", "place"));
    }

    #[test]
    fn cancellable_until_shipped() {
        assert!(Order::new(1).is_cancellable());
        assert!(placed().is_cancellable());
        assert!(!shipped().is_cancellable());
        let mut order = placed();
        order.apply(Event::Cancel).unwrap();
        assert!(!order.is_cancellable());
    }

    #[test]
    fn summaries() {
        assert_eq!(Order::new(1).summary(), "no items");
        let mut order = Order::new(1);
        order.apply(Event::AddLine(line("bolt", 3, 25))).unwrap();
        assert_eq!(order.summary(), "3x bolt");
        assert_eq!(draft().summary(), "3x bolt and 1 more");
        let mut order = shipped();
        order.apply(Event::Deliver).unwrap();
        assert_eq!(order.summary(), "no items");
    }
}
