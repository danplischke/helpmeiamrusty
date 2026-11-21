//! # Associated type or generic parameter?
//!
//! With `trait Handler { type Event; .. }` a type can implement `Handler`
//! once, and that impl fixes its event type. With `trait Handler<E>` it can
//! implement it many times: `Handler<Click>` and `Handler<Key>` are
//! different traits. For each trait, ask: can one type implement it more
//! than once?
//!
//! `Logger` should handle keys as well as clicks, and the tests write
//! `Recorder<Clicks>`, with one type parameter. Neither fits the traits as
//! they're designed. Redesign them; the method bodies barely change.

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Click {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Key(char);

/// Reacts to events.
trait Handler {
    type Event;
    fn handle(&mut self, event: &Self::Event);
}

/// Produces events until it runs dry.
trait Source<E> {
    fn next_event(&mut self) -> Option<E>;
}

/// Writes one line per event: "click 3,4" for clicks, "key q" for keys.
#[derive(Debug, Default)]
struct Logger {
    lines: Vec<String>,
}

impl Handler for Logger {
    type Event = Click;

    fn handle(&mut self, click: &Click) {
        self.lines.push(format!("click {},{}", click.x, click.y));
    }
}

/// Counts keystrokes.
#[derive(Debug, Default)]
struct KeyCounter {
    count: usize,
}

impl Handler for KeyCounter {
    type Event = Key;

    fn handle(&mut self, _key: &Key) {
        self.count += 1;
    }
}

/// A scripted mouse.
struct Clicks(VecDeque<Click>);

/// A scripted keyboard.
struct Keys(VecDeque<Key>);

impl Source<Click> for Clicks {
    fn next_event(&mut self) -> Option<Click> {
        self.0.pop_front()
    }
}

impl Source<Key> for Keys {
    fn next_event(&mut self) -> Option<Key> {
        self.0.pop_front()
    }
}

/// Feeds every event from `source` to `handler`. Returns how many there
/// were.
fn pump<S, H, E>(source: &mut S, handler: &mut H) -> usize
where
    S: Source<E>,
    H: Handler<Event = E>,
{
    let mut count = 0;
    while let Some(event) = source.next_event() {
        handler.handle(&event);
        count += 1;
    }
    count
}

/// Pulls events from a source one at a time, keeping a copy of each.
struct Recorder<S, E> {
    source: S,
    seen: Vec<E>,
}

impl<S: Source<E>, E: Clone> Recorder<S, E> {
    fn new(source: S) -> Self {
        Self {
            source,
            seen: Vec::new(),
        }
    }

    /// Pulls the next event, records it and returns it.
    fn pull(&mut self) -> Option<E> {
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
