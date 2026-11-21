//! # Checks that can't drift apart
//!
//! Every `unwrap` and `expect` below sits right after a check that should
//! make it safe. Each one is a promise the compiler can't verify, and one
//! of them is wrong: a test makes it panic. Restructure so the check *is*
//! the pattern match: `let .. else`, `if let`, `match`, `split_once`,
//! `strip_prefix`, `unwrap_or`, `map_or`, `?`.
//!
//! Rules: no `unwrap`/`expect` outside the tests.

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum Command {
    Join(String),
    Msg { to: String, text: String },
    Away(Option<String>),
    Slow(u32),
}

#[derive(Debug, PartialEq)]
enum CommandError {
    NotACommand,
    Unknown(String),
    MissingArgument(&'static str),
    BadChannel(String),
    BadNumber(String),
}

/// Parses a chat command. The name runs from the '/' to the first space;
/// the rest, trimmed, is the argument.
/// - `/join #<channel>`: the channel is '#' plus at least one character,
///   else `BadChannel(<argument>)`.
/// - `/msg <nick> <text>`: `MissingArgument("nick")` if there's no
///   argument, `MissingArgument("text")` if there's only a nick.
/// - `/away` or `/away <message>`.
/// - `/slow <seconds>`: a `u32`. `MissingArgument("seconds")` without an
///   argument, `BadNumber(<argument>)` if it isn't a `u32`.
///
/// A line without the leading '/' is `NotACommand`; any other name is
/// `Unknown(<name>)`.
fn parse_command(line: &str) -> Result<Command, CommandError> {
    if !line.starts_with('/') {
        return Err(CommandError::NotACommand);
    }
    let body = line.strip_prefix('/').unwrap();
    let (name, rest) = if body.contains(' ') {
        let (name, rest) = body.split_once(' ').unwrap();
        (name, rest.trim())
    } else {
        (body, "")
    };
    match name {
        "join" => {
            if rest.len() < 2 || !rest.starts_with('#') {
                return Err(CommandError::BadChannel(rest.to_string()));
            }
            Ok(Command::Join(rest.to_string()))
        }
        "msg" => {
            if rest.is_empty() {
                return Err(CommandError::MissingArgument("nick"));
            }
            if !rest.contains(' ') {
                return Err(CommandError::MissingArgument("text"));
            }
            let (to, text) = rest.split_once(' ').unwrap();
            Ok(Command::Msg {
                to: to.to_string(),
                text: text.trim().to_string(),
            })
        }
        "away" => {
            if rest.is_empty() {
                Ok(Command::Away(None))
            } else {
                Ok(Command::Away(Some(rest.to_string())))
            }
        }
        "slow" => {
            if rest.chars().all(|c| c.is_ascii_digit()) {
                Ok(Command::Slow(rest.parse().expect("checked: only digits")))
            } else {
                Err(CommandError::BadNumber(rest.to_string()))
            }
        }
        _ => Err(CommandError::Unknown(name.to_string())),
    }
}

/// The nicks mentioned in a message: words that start with '@', without
/// the '@'. A lone "@" isn't a mention.
fn mentions(text: &str) -> Vec<&str> {
    let mut nicks = Vec::new();
    for word in text.split_whitespace() {
        if word.len() > 1 && word.starts_with('@') {
            nicks.push(word.strip_prefix('@').unwrap());
        }
    }
    nicks
}

/// The letter on a user's avatar: the first character of the nick,
/// uppercased (ASCII only), or '?' for an empty nick.
fn avatar(nick: &str) -> char {
    if nick.is_empty() {
        return '?';
    }
    nick.chars().next().unwrap().to_ascii_uppercase()
}

/// What to tell someone who messages `nick`.
fn status(away: &HashMap<String, String>, nick: &str) -> String {
    if away.contains_key(nick) {
        format!("{nick} is away: {}", away.get(nick).unwrap())
    } else {
        format!("{nick} is here")
    }
}

fn main() {
    for line in ["/join #rust", "/msg ana see you at 5", "/slow 30", "hi"] {
        println!("{:?}", parse_command(line));
    }
    println!("{:?}", mentions("thanks @ana and @bo!"));
    println!("{}", avatar("zoë"));
    let away = HashMap::from([("ana".to_string(), "lunch".to_string())]);
    println!("{}", status(&away, "ana"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(line: &str) -> Result<Command, CommandError> {
        parse_command(line)
    }

    #[test]
    fn commands() {
        assert_eq!(cmd("/join #rust"), Ok(Command::Join("#rust".to_string())));
        assert_eq!(
            cmd("/msg ana  see you at 5 "),
            Ok(Command::Msg {
                to: "ana".to_string(),
                text: "see you at 5".to_string()
            })
        );
        assert_eq!(cmd("/away"), Ok(Command::Away(None)));
        assert_eq!(
            cmd("/away  back soon "),
            Ok(Command::Away(Some("back soon".to_string())))
        );
        assert_eq!(cmd("/slow 30"), Ok(Command::Slow(30)));
    }

    #[test]
    fn command_errors() {
        assert_eq!(cmd("hello"), Err(CommandError::NotACommand));
        assert_eq!(
            cmd("/dance"),
            Err(CommandError::Unknown("dance".to_string()))
        );
        assert_eq!(
            cmd("/join rust"),
            Err(CommandError::BadChannel("rust".to_string()))
        );
        assert_eq!(
            cmd("/join #"),
            Err(CommandError::BadChannel("#".to_string()))
        );
        assert_eq!(cmd("/msg"), Err(CommandError::MissingArgument("nick")));
        assert_eq!(cmd("/msg ana"), Err(CommandError::MissingArgument("text")));
        assert_eq!(
            cmd("/slow 5s"),
            Err(CommandError::BadNumber("5s".to_string()))
        );
    }

    #[test]
    fn slow_mode_needs_a_number_that_fits() {
        assert_eq!(cmd("/slow"), Err(CommandError::MissingArgument("seconds")));
        assert_eq!(cmd("/slow "), Err(CommandError::MissingArgument("seconds")));
        assert_eq!(
            cmd("/slow 99999999999"),
            Err(CommandError::BadNumber("99999999999".to_string()))
        );
    }

    #[test]
    fn finds_mentions() {
        assert_eq!(mentions("hi @ana @ and @bo"), ["ana", "bo"]);
        assert!(mentions("no one here").is_empty());
    }

    #[test]
    fn avatars() {
        assert_eq!(avatar("ana"), 'A');
        assert_eq!(avatar("émile"), 'é');
        assert_eq!(avatar(""), '?');
    }

    #[test]
    fn statuses() {
        let away = HashMap::from([("ana".to_string(), "lunch".to_string())]);
        assert_eq!(status(&away, "ana"), "ana is away: lunch");
        assert_eq!(status(&away, "bo"), "bo is here");
    }
}
