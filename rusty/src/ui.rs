//! Terminal output helpers: colors, rules, banners, wrapping, line input.

use std::io::{self, BufRead, IsTerminal, Write};
use std::sync::OnceLock;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::Duration;

pub const WIDTH: usize = 72;

pub fn color_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("NO_COLOR").is_none() && io::stdout().is_terminal())
}

/// Value for `--color` flags passed to cargo and rustc.
pub fn color_arg() -> &'static str {
    if color_enabled() { "always" } else { "never" }
}

fn paint(code: &str, s: &str) -> String {
    if color_enabled() {
        format!("\x1b[{code}m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

pub fn bold(s: &str) -> String {
    paint("1", s)
}
pub fn dim(s: &str) -> String {
    paint("2", s)
}
pub fn red(s: &str) -> String {
    paint("31", s)
}
pub fn green(s: &str) -> String {
    paint("32", s)
}
pub fn yellow(s: &str) -> String {
    paint("33", s)
}
pub fn cyan(s: &str) -> String {
    paint("36", s)
}

pub fn clear_screen() {
    if color_enabled() {
        print!("\x1b[2J\x1b[3J\x1b[H");
    } else {
        println!();
    }
}

pub fn rule() -> String {
    dim(&"─".repeat(WIDTH))
}

/// `━━ left ━━━━━━━━━━━━━ right ━━`
pub fn banner(left: &str, right: &str) -> String {
    let used = left.chars().count() + right.chars().count() + 8;
    let fill = WIDTH.saturating_sub(used).max(2);
    format!(
        "{} {} {} {} {}",
        dim("━━"),
        bold(left),
        dim(&"━".repeat(fill)),
        right,
        dim("━━")
    )
}

pub fn progress_bar(done: usize, total: usize, width: usize) -> String {
    let filled = (done * width).checked_div(total).unwrap_or(0);
    format!(
        "{}{}",
        green(&"▓".repeat(filled)),
        dim(&"░".repeat(width - filled))
    )
}

/// Word-wraps `text` to the screen width; blank lines separate paragraphs.
pub fn wrap(text: &str, indent: &str) -> String {
    let width = WIDTH.saturating_sub(indent.chars().count()).max(20);
    let mut out = Vec::new();
    for paragraph in text.trim().split("\n\n") {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
                out.push(format!("{indent}{line}"));
                line.clear();
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        out.push(format!("{indent}{line}"));
        out.push(String::new());
    }
    out.pop();
    out.join("\n")
}

pub fn indent(text: &str, prefix: &str) -> String {
    text.lines()
        .map(|l| format!("{prefix}{l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub enum Poll {
    Line(String),
    Timeout,
    Closed,
}

/// Lines typed by the user, read on a background thread so the watcher can poll files meanwhile.
pub struct Input {
    rx: Receiver<String>,
}

impl Input {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            for line in io::stdin().lock().lines() {
                let Ok(line) = line else { break };
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        Self { rx }
    }

    pub fn poll(&self, timeout: Duration) -> Poll {
        match self.rx.recv_timeout(timeout) {
            Ok(line) => Poll::Line(line),
            Err(RecvTimeoutError::Timeout) => Poll::Timeout,
            Err(RecvTimeoutError::Disconnected) => Poll::Closed,
        }
    }

    /// Prints `prompt` and blocks for one line. `None` once stdin is closed.
    pub fn ask(&self, prompt: &str) -> Option<String> {
        print!("{prompt}");
        io::stdout().flush().ok();
        self.rx.recv().ok()
    }

    /// Drops lines typed while we were busy (e.g. during a long check).
    pub fn discard_pending(&self) {
        while self.rx.try_recv().is_ok() {}
    }
}
