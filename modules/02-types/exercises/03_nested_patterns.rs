//! # Nested patterns, `@` and `|`
//!
//! A terminal editor turns input events into actions, one match arm per
//! rule. Patterns nest as deep as the data: destructure an input, its key
//! and its modifiers in one go. `c @ '1'..='9'` tests a range and keeps the
//! value. `|` works deep inside a pattern too (`key: Key::Esc | Key::F(10)`).
//! Named constants such as `CTRL` work as patterns. Use a guard (`if ..`)
//! only for what a pattern can't say.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Key {
    Char(char),
    Enter,
    Esc,
    F(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mods {
    ctrl: bool,
    alt: bool,
}

const PLAIN: Mods = Mods {
    ctrl: false,
    alt: false,
};
const CTRL: Mods = Mods {
    ctrl: true,
    alt: false,
};
const ALT: Mods = Mods {
    ctrl: false,
    alt: true,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Input {
    Key { key: Key, mods: Mods },
    Click { x: u16, y: u16 },
    Scroll(i16),
    Resize { width: u16, height: u16 },
}

#[derive(Debug, PartialEq)]
enum Action {
    Save,
    Quit,
    Help,
    SwitchTab(u8),
    Insert(char),
    Newline,
    OpenMenu { x: u16 },
    MoveCursor { x: u16, y: u16 },
    ScrollBy(i16),
    Redraw { width: u16, height: u16 },
    Ignore,
}

/// The keymap. Modifiers must match exactly: Ctrl+Alt+S is not Save.
///
/// - Ctrl+S is `Save`. Ctrl+Q, plain Esc and plain F10 are `Quit`.
/// - F1 is `Help`, whatever the modifiers.
/// - Alt+1 to Alt+9 are `SwitchTab(1)` to `SwitchTab(9)`.
/// - A plain character is `Insert`, and a plain Enter is `Newline`.
/// - A click on the top row (`y == 0`) is `OpenMenu`; anywhere else it's
///   `MoveCursor`.
/// - A scroll is `ScrollBy`, unless it scrolls by 0.
/// - A resize to at least 20 columns and 5 rows is `Redraw`.
/// - Everything else is `Ignore`.
fn action(input: Input) -> Action {
    todo!("map {input:?} to an action")
}

/// Whether the input is a key pressed with Ctrl or Alt (or both) held.
/// One `matches!` will do.
fn is_chord(input: Input) -> bool {
    todo!("is {input:?} a chord?")
}

fn main() {
    let session = [
        Input::Key {
            key: Key::Char('h'),
            mods: PLAIN,
        },
        Input::Key {
            key: Key::Char('2'),
            mods: ALT,
        },
        Input::Click { x: 12, y: 0 },
        Input::Resize {
            width: 10,
            height: 3,
        },
        Input::Key {
            key: Key::Char('s'),
            mods: CTRL,
        },
    ];
    for input in session {
        println!(
            "{input:?} -> {:?} (chord: {})",
            action(input),
            is_chord(input)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(key: Key, mods: Mods) -> Input {
        Input::Key { key, mods }
    }

    const CTRL_ALT: Mods = Mods {
        ctrl: true,
        alt: true,
    };

    #[test]
    fn shortcuts() {
        assert_eq!(action(press(Key::Char('s'), CTRL)), Action::Save);
        assert_eq!(action(press(Key::Char('s'), CTRL_ALT)), Action::Ignore);
        assert_eq!(action(press(Key::Char('q'), CTRL)), Action::Quit);
        assert_eq!(action(press(Key::Esc, PLAIN)), Action::Quit);
        assert_eq!(action(press(Key::F(10), PLAIN)), Action::Quit);
        assert_eq!(action(press(Key::Esc, ALT)), Action::Ignore);
        assert_eq!(action(press(Key::F(10), CTRL)), Action::Ignore);
        assert_eq!(action(press(Key::F(1), PLAIN)), Action::Help);
        assert_eq!(action(press(Key::F(1), CTRL_ALT)), Action::Help);
        assert_eq!(action(press(Key::F(2), PLAIN)), Action::Ignore);
    }

    #[test]
    fn tabs() {
        assert_eq!(action(press(Key::Char('1'), ALT)), Action::SwitchTab(1));
        assert_eq!(action(press(Key::Char('9'), ALT)), Action::SwitchTab(9));
        assert_eq!(action(press(Key::Char('0'), ALT)), Action::Ignore);
        assert_eq!(action(press(Key::Char('a'), ALT)), Action::Ignore);
        assert_eq!(action(press(Key::Char('1'), CTRL)), Action::Ignore);
    }

    #[test]
    fn typing() {
        assert_eq!(action(press(Key::Char('x'), PLAIN)), Action::Insert('x'));
        assert_eq!(action(press(Key::Char('7'), PLAIN)), Action::Insert('7'));
        assert_eq!(action(press(Key::Char('ß'), PLAIN)), Action::Insert('ß'));
        assert_eq!(action(press(Key::Char('x'), CTRL)), Action::Ignore);
        assert_eq!(action(press(Key::Enter, PLAIN)), Action::Newline);
        assert_eq!(action(press(Key::Enter, CTRL)), Action::Ignore);
    }

    #[test]
    fn mouse() {
        assert_eq!(
            action(Input::Click { x: 5, y: 0 }),
            Action::OpenMenu { x: 5 }
        );
        assert_eq!(
            action(Input::Click { x: 5, y: 3 }),
            Action::MoveCursor { x: 5, y: 3 }
        );
        assert_eq!(action(Input::Scroll(-3)), Action::ScrollBy(-3));
        assert_eq!(action(Input::Scroll(0)), Action::Ignore);
    }

    #[test]
    fn resizing() {
        let resize = |width, height| action(Input::Resize { width, height });
        assert_eq!(
            resize(80, 24),
            Action::Redraw {
                width: 80,
                height: 24
            }
        );
        assert_eq!(
            resize(20, 5),
            Action::Redraw {
                width: 20,
                height: 5
            }
        );
        assert_eq!(resize(19, 50), Action::Ignore);
        assert_eq!(resize(200, 4), Action::Ignore);
    }

    #[test]
    fn chords() {
        assert!(is_chord(press(Key::Char('s'), CTRL)));
        assert!(is_chord(press(Key::Enter, ALT)));
        assert!(is_chord(press(Key::F(1), CTRL_ALT)));
        assert!(!is_chord(press(Key::Char('s'), PLAIN)));
        assert!(!is_chord(Input::Scroll(1)));
    }
}
