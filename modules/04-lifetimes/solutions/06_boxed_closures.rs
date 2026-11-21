//! # Boxed closures that borrow (reference solution)
//!
//! - `Box<dyn Trait>` in a field or a signature is `Box<dyn Trait + 'static>`.
//!   The compiler's suggestion, `rule: impl Fn(&str) -> bool + 'static`,
//!   makes `with` compile but rejects every closure that borrows a local.
//! - `LineFilter<'a>` holding `Box<dyn Fn(&str) -> bool + 'a>` accepts
//!   both: borrowing closures make it a `LineFilter<'short>`, owning ones
//!   leave it free to be `LineFilter<'static>`. Type aliases take lifetime
//!   parameters too, which keeps clippy's `type_complexity` quiet.
//! - `T: 'static` means "contains no borrows shorter than `'static`", not
//!   "lives forever". A `String`, a `Vec<String>`, a `move` closure over
//!   owned data: all `'static`, all dropped normally. `skip_prefixes` copies
//!   the prefixes into a `Vec<String>` and moves it in.
//! - `dyn Trait + 'a` is covariant in `'a`, so a `LineFilter<'static>` can
//!   still take a borrowing rule later: it shrinks to a shorter lifetime.

/// A rule: does this line pass?
type Rule<'a> = Box<dyn Fn(&str) -> bool + 'a>;

struct LineFilter<'a> {
    rules: Vec<Rule<'a>>,
}

impl<'a> LineFilter<'a> {
    fn new() -> Self {
        LineFilter { rules: Vec::new() }
    }

    /// Adds a rule. A line is kept only if every rule returns true.
    fn with(mut self, rule: impl Fn(&str) -> bool + 'a) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    /// Whether `line` passes every rule. With no rules, every line passes.
    fn keeps(&self, line: &str) -> bool {
        self.rules.iter().all(|rule| rule(line))
    }

    /// The lines of `text` that pass, as slices of `text`.
    fn apply<'t>(&self, text: &'t str) -> Vec<&'t str> {
        text.lines().filter(|line| self.keeps(line)).collect()
    }
}

/// A filter that drops blank lines and lines that start with any of
/// `prefixes`, ignoring leading whitespace. It must not borrow `prefixes`:
/// callers may drop them and keep the filter.
fn skip_prefixes(prefixes: &[&str]) -> LineFilter<'static> {
    let prefixes: Vec<String> = prefixes.iter().map(|p| p.to_string()).collect();
    LineFilter::new()
        .with(|line| !line.trim().is_empty())
        .with(move |line| {
            let line = line.trim_start();
            !prefixes.iter().any(|p| line.starts_with(p.as_str()))
        })
}

const LOG: &str = "\
# server log
GET /index.html 200

password=hunter2 in query
GET /admin 403
   # indented comment";

fn main() {
    let banned = ["password"];
    let filter = skip_prefixes(&["#"]).with(|line| !banned.iter().any(|w| line.contains(w)));
    for line in filter.apply(LOG) {
        println!("{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_rules_keep_everything() {
        let filter = LineFilter::new();
        assert!(filter.keeps(""));
        assert_eq!(filter.apply("a\n\nb"), ["a", "", "b"]);
    }

    #[test]
    fn rules_can_borrow_local_data() {
        let banned = vec![String::from("password"), String::from("token")];
        let filter = LineFilter::new()
            .with(|line| !line.trim().is_empty())
            .with(|line| !banned.iter().any(|word| line.contains(word.as_str())));
        assert!(filter.keeps("GET / 200"));
        assert!(!filter.keeps("token=abc"));
        assert!(!filter.keeps("  "));
        assert_eq!(
            filter.apply(LOG),
            [
                "# server log",
                "GET /index.html 200",
                "GET /admin 403",
                "   # indented comment",
            ]
        );
    }

    #[test]
    fn lines_outlive_the_filter() {
        let text = String::from(LOG);
        let kept = {
            let filter = LineFilter::new().with(|line| line.starts_with("GET"));
            filter.apply(&text)
        };
        assert_eq!(kept, ["GET /index.html 200", "GET /admin 403"]);
    }

    #[test]
    fn prefix_filter_owns_its_data() {
        let filter = {
            let prefixes = vec![String::from("#"), String::from("//")];
            let refs: Vec<&str> = prefixes.iter().map(String::as_str).collect();
            skip_prefixes(&refs)
        }; // `prefixes` is gone; the filter must not borrow it
        assert_eq!(filter.apply("# a\nb\n  // c\n\nd"), ["b", "d"]);
    }

    #[test]
    fn owning_filters_are_static() {
        fn assert_static<T: 'static>(_: &T) {}
        let filter = skip_prefixes(&["#"]);
        assert_static(&filter);
        let min_len = 3;
        let filter = LineFilter::new().with(move |line| line.len() >= min_len);
        assert_static(&filter);
        assert!(!filter.keeps("ab"));
    }

    #[test]
    fn static_filters_accept_borrowing_rules() {
        let banned = String::from("admin");
        let filter = skip_prefixes(&["#"]).with(|line| !line.contains(banned.as_str()));
        assert_eq!(
            filter.apply(LOG),
            ["GET /index.html 200", "password=hunter2 in query"]
        );
    }
}
