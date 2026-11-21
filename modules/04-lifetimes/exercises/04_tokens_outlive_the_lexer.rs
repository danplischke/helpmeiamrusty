//! # Tokens that outlive the lexer
//!
//! A `Lexer<'a>` hands out slices of its input. Those slices point into the
//! *input*, which lives for `'a`, so they may outlive the lexer itself. The
//! signatures below tie them to `&self` and `&mut self` instead: the most
//! common lifetime mistake in parsers. Fix the signatures; the bodies are
//! fine as they are.

/// Whether `c` belongs in a word token.
fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Splits `s`, after skipping leading whitespace, into its first token and
/// the rest. A token is a run of word characters (alphanumeric or `_`), or
/// any other single character. `None` if only whitespace is left.
fn split_token(s: &str) -> Option<(&str, &str)> {
    let s = s.trim_start();
    let first = s.chars().next()?;
    let end = if is_word(first) {
        s.find(|c: char| !is_word(c)).unwrap_or(s.len())
    } else {
        first.len_utf8()
    };
    Some(s.split_at(end))
}

struct Lexer<'a> {
    rest: &'a str,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Lexer { rest: input }
    }

    /// The next token, without consuming it.
    fn peek(&self) -> Option<&str> {
        split_token(self.rest).map(|(token, _)| token)
    }

    /// Consumes and returns the next token.
    fn next_token(&mut self) -> Option<&str> {
        let (token, rest) = split_token(self.rest)?;
        self.rest = rest;
        Some(token)
    }

    /// Consumes the next token if it equals `expected`.
    fn eat(&mut self, expected: &str) -> bool {
        if self.peek() == Some(expected) {
            self.next_token();
            true
        } else {
            false
        }
    }

    /// The input that hasn't been consumed yet.
    fn rest(&self) -> &str {
        self.rest
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        self.next_token()
    }
}

fn main() {
    let source = String::from("let total = price * 2;");
    let tokens: Vec<&str> = Lexer::new(&source).collect();
    println!("{tokens:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_tokens() {
        assert_eq!(split_token("  let x"), Some(("let", " x")));
        assert_eq!(split_token("+= 1"), Some(("+", "= 1")));
        assert_eq!(split_token("naïve_name2!"), Some(("naïve_name2", "!")));
        assert_eq!(split_token(" \n "), None);
    }

    #[test]
    fn tokens_outlive_the_lexer() {
        let source = String::from("let total = price * 2;");
        let tokens: Vec<&str> = Lexer::new(&source).collect();
        assert_eq!(tokens, ["let", "total", "=", "price", "*", "2", ";"]);
    }

    #[test]
    fn collecting_by_hand() {
        let source = String::from("a+b");
        let mut lexer = Lexer::new(&source);
        let mut tokens = Vec::new();
        while let Some(token) = lexer.next_token() {
            tokens.push(token);
        }
        assert_eq!(tokens, ["a", "+", "b"]);
    }

    #[test]
    fn peeked_token_survives_consuming_it() {
        let mut lexer = Lexer::new("x = 1");
        let peeked = lexer.peek();
        let taken = lexer.next_token();
        assert_eq!(peeked, Some("x"));
        assert_eq!(peeked, taken);
        assert!(lexer.eat("="));
        assert!(!lexer.eat("="));
        assert_eq!(lexer.next_token(), Some("1"));
        assert_eq!(lexer.next_token(), None);
        assert_eq!(lexer.peek(), None);
    }

    #[test]
    fn rest_outlives_the_lexer() {
        let source = String::from("fn main() {}");
        let rest = {
            let mut lexer = Lexer::new(&source);
            lexer.next_token();
            lexer.rest()
        };
        assert_eq!(rest, " main() {}");
    }
}
