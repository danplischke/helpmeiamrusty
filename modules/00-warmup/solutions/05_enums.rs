//! # Enums and pattern matching (reference solution)
//!
//! - Matching on a tuple of `next()` calls checks "exactly two, both
//!   numbers" in one pattern: `(Some(Ok(a)), Some(Ok(b)), None)`.
//! - `split_once(' ')` + `unwrap_or((line, ""))` splits off the verb, and
//!   guards (`"say" if ...`) refine an arm without nesting `if`s.
//! - `Option::map` builds the variant: `parse_pair(rest).map(|(x, y)| ...)`.
//! - `(self.x, self.y) = (x, y)` is destructuring assignment (Rust 1.59).

#[derive(Debug, PartialEq)]
enum Command {
    Move { dx: i32, dy: i32 },
    Say(String),
    Teleport(i32, i32),
    Quit,
}

/// "3 -2" -> Some((3, -2)). Exactly two integers separated by whitespace.
fn parse_pair(s: &str) -> Option<(i32, i32)> {
    let mut numbers = s.split_whitespace().map(str::parse::<i32>);
    match (numbers.next(), numbers.next(), numbers.next()) {
        (Some(Ok(a)), Some(Ok(b)), None) => Some((a, b)),
        _ => None,
    }
}

/// One of "move <dx> <dy>", "tp <x> <y>", "say <text>" (text not empty) or
/// "quit" (nothing after it). Surrounding whitespace is ignored. Anything
/// else is None.
fn parse_command(line: &str) -> Option<Command> {
    let line = line.trim();
    let (verb, rest) = line.split_once(' ').unwrap_or((line, ""));
    let rest = rest.trim();
    match verb {
        "move" => parse_pair(rest).map(|(dx, dy)| Command::Move { dx, dy }),
        "tp" => parse_pair(rest).map(|(x, y)| Command::Teleport(x, y)),
        "say" if !rest.is_empty() => Some(Command::Say(rest.to_string())),
        "quit" if rest.is_empty() => Some(Command::Quit),
        _ => None,
    }
}

#[derive(Debug, Default)]
struct Robot {
    x: i32,
    y: i32,
    said: Vec<String>,
    stopped: bool,
}

impl Robot {
    /// Applies a command. Once stopped, the robot ignores everything.
    fn apply(&mut self, cmd: Command) {
        if self.stopped {
            return;
        }
        match cmd {
            Command::Move { dx, dy } => {
                self.x += dx;
                self.y += dy;
            }
            Command::Teleport(x, y) => (self.x, self.y) = (x, y),
            Command::Say(text) => self.said.push(text),
            Command::Quit => self.stopped = true,
        }
    }
}

fn main() {
    let script = "move 3 -2\nsay hello there\ntp 10 10\nmove -1 0\nquit\nmove 5 5";
    let mut robot = Robot::default();
    for line in script.lines() {
        match parse_command(line) {
            Some(cmd) => robot.apply(cmd),
            None => println!("ignoring {line:?}"),
        }
    }
    println!("{robot:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pairs() {
        assert_eq!(parse_pair("3 -2"), Some((3, -2)));
        assert_eq!(parse_pair("  7   8 "), Some((7, 8)));
        assert_eq!(parse_pair("7"), None);
        assert_eq!(parse_pair("7 8 9"), None);
        assert_eq!(parse_pair("a 8"), None);
    }

    #[test]
    fn parses_commands() {
        assert_eq!(
            parse_command("move 3 -2"),
            Some(Command::Move { dx: 3, dy: -2 })
        );
        assert_eq!(parse_command("tp 10 20"), Some(Command::Teleport(10, 20)));
        assert_eq!(
            parse_command("say hello world"),
            Some(Command::Say("hello world".to_string()))
        );
        assert_eq!(parse_command("  quit "), Some(Command::Quit));
    }

    #[test]
    fn rejects_bad_commands() {
        for bad in [
            "move 3", "move a b", "tp 1 2 3", "say", "say   ", "quit now", "dance", "",
        ] {
            assert_eq!(parse_command(bad), None, "{bad:?} should be rejected");
        }
    }

    #[test]
    fn robot_follows_commands_until_quit() {
        let mut robot = Robot::default();
        robot.apply(Command::Move { dx: 3, dy: -2 });
        robot.apply(Command::Say("hi".to_string()));
        robot.apply(Command::Teleport(10, 10));
        robot.apply(Command::Move { dx: -1, dy: 0 });
        robot.apply(Command::Quit);
        robot.apply(Command::Move { dx: 5, dy: 5 });
        robot.apply(Command::Say("still here?".to_string()));
        assert_eq!((robot.x, robot.y), (9, 10));
        assert_eq!(robot.said, ["hi"]);
        assert!(robot.stopped);
    }
}
