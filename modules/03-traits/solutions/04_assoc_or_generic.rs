//! # Associated type or generic parameter? (reference solution)
//!
//! - A generic parameter makes a family of traits: `Logger` implements both
//!   `Handler<Click>` and `Handler<Key>`, and the argument type picks the
//!   impl. Std examples: `From<T>`, `PartialEq<Rhs>`, `AsRef<T>`.
//! - An associated type means one impl per type, and the type determines
//!   it: `Iterator::Item`, `Deref::Target`, `FromStr::Err`. Generic code can
//!   name it (`S::Event`) without an extra parameter, which is why
//!   `Recorder<S>` needs only one.
//! - `H: Handler<S::Event>` uses both: the source decides the event type,
//!   and that selects which `Handler` impl runs. `Add<Rhs> { type Output }`
//!   mixes them the same way: many right-hand sides, one output for each.
//! - A struct whose *fields* mention `S::Event` needs the `S: Source` bound
//!   on the struct itself; methods can add more (`where S::Event: Clone`).

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Click {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Key(char);

/// Reacts to events.
trait Handler<E> {
    fn handle(&mut self, event: &E);
}

/// Produces events until it runs dry.
trait Source {
    type Event;
    fn next_event(&mut self) -> Option<Self::Event>;
}

/// Writes one line per event: "click 3,4" for clicks, "key q" for keys.
#[derive(Debug, Default)]
struct Logger {
    lines: Vec<String>,
}

impl Handler<Click> for Logger {
    fn handle(&mut self, click: &Click) {
        self.lines.push(format!("click {},{}", click.x, click.y));
    }
}

impl Handler<Key> for Logger {
    fn handle(&mut self, key: &Key) {
        self.lines.push(format!("key {}", key.0));
    }
}

/// Counts keystrokes.
#[derive(Debug, Default)]
struct KeyCounter {
    count: usize,
}

impl Handler<Key> for KeyCounter {
    fn handle(&mut self, _key: &Key) {
        self.count += 1;
    }
}

/// A scripted mouse.
struct Clicks(VecDeque<Click>);

/// A scripted keyboard.
struct Keys(VecDeque<Key>);

impl Source for Clicks {
    type Event = Click;

    fn next_event(&mut self) -> Option<Click> {
        self.0.pop_front()
    }
}

impl Source for Keys {
    type Event = Key;

    fn next_event(&mut self) -> Option<Key> {
        self.0.pop_front()
    }
}

/// Feeds every event from `source` to `handler`. Returns how many there
/// were.
fn pump<S, H>(source: &mut S, handler: &mut H) -> usize
where
    S: Source,
    H: Handler<S::Event>,
{
    let mut count = 0;
    while let Some(event) = source.next_event() {
        handler.handle(&event);
        count += 1;
    }
    count
}

/// Pulls events from a source one at a time, keeping a copy of each.
struct Recorder<S: Source> {
    source: S,
    seen: Vec<S::Event>,
}

impl<S: Source> Recorder<S>
where
    S::Event: Clone,
{
    fn new(source: S) -> Self {
        Self {
            source,
            seen: Vec::new(),
        }
    }

    /// Pulls the next event, records it and returns it.
    fn pull(&mut self) -> Option<S::Event> {
        let event = self.source.next_event()?;
        self.seen.push(event.clone());
        Some(event)
    }
}

fn main() {
    let mut logger = Logger::default();
    let mut clicks = Clicks(VecDeque::from([Click { x: 1, y: 2 }]));
    println!("{} events", pump(&mut clicks, &mut logger));
    let mut counter = KeyCounter::default();
    let mut keys = Keys("hello".chars().map(Key).collect());
    println!("{} events", pump(&mut keys, &mut counter));
    let mut recorder = Recorder::new(Clicks(VecDeque::from([Click { x: 5, y: 5 }])));
    println!("{:?} {:?}", recorder.pull(), logger.lines);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clicks() -> Clicks {
        Clicks(VecDeque::from([Click { x: 1, y: 2 }, Click { x: 3, y: 4 }]))
    }

    fn keys(text: &str) -> Keys {
        Keys(text.chars().map(Key).collect())
    }

    #[test]
    fn logger_handles_clicks_and_keys() {
        let mut logger = Logger::default();
        logger.handle(&Click { x: 0, y: 7 });
        logger.handle(&Key('q'));
        assert_eq!(logger.lines, ["click 0,7", "key q"]);
    }

    #[test]
    fn pump_feeds_any_matching_handler() {
        let mut logger = Logger::default();
        assert_eq!(pump(&mut clicks(), &mut logger), 2);
        assert_eq!(pump(&mut keys("hi"), &mut logger), 2);
        assert_eq!(logger.lines, ["click 1,2", "click 3,4", "key h", "key i"]);
        let mut counter = KeyCounter::default();
        assert_eq!(pump(&mut keys("hello"), &mut counter), 5);
        assert_eq!(counter.count, 5);
    }

    #[test]
    fn recorder_needs_one_type_parameter() {
        let mut recorder: Recorder<Clicks> = Recorder::new(clicks());
        assert_eq!(recorder.pull(), Some(Click { x: 1, y: 2 }));
        assert_eq!(recorder.pull(), Some(Click { x: 3, y: 4 }));
        assert_eq!(recorder.pull(), None);
        assert_eq!(recorder.seen.len(), 2);
    }
}
