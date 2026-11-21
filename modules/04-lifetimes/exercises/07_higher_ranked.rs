//! # Closures that work for any lifetime
//!
//! `F: Fn(&'a str) -> &'a str` with `'a` declared on the function means one
//! lifetime, chosen by the *caller*. A function that calls `f` on its own
//! local strings needs `F` to work for every lifetime:
//! `for<'x> Fn(&'x str) -> &'x str`, which is what the elided
//! `Fn(&str) -> &str` means. The same goes for `dyn Fn` types. Fix the
//! errors without changing what the code does.

/// Lowercases each tag, applies `key` to it and returns the distinct keys
/// in first-seen order.
fn distinct_keys<'a, F>(tags: &[&str], key: F) -> Vec<String>
where
    F: Fn(&'a str) -> &'a str,
{
    let mut keys: Vec<String> = Vec::new();
    for tag in tags {
        let lower = tag.to_lowercase();
        let k = key(&lower);
        if !keys.iter().any(|seen| seen == k) {
            keys.push(k.to_string());
        }
    }
    keys
}

/// One cleanup step: returns a slice of its input.
type Step<'a> = Box<dyn Fn(&'a str) -> &'a str>;

/// Cleanup steps, applied in order.
struct Pipeline<'a> {
    steps: Vec<Step<'a>>,
}

impl<'a> Pipeline<'a> {
    fn new() -> Self {
        Pipeline { steps: Vec::new() }
    }

    /// Adds a step at the end.
    fn step(mut self, f: impl Fn(&'a str) -> &'a str + 'static) -> Self {
        self.steps.push(Box::new(f));
        self
    }

    /// Runs every step in order; with no steps, returns `tag` unchanged.
    fn run(&self, tag: &'a str) -> &'a str {
        self.steps.iter().fold(tag, |tag, step| step(tag))
    }
}

/// "lifetimes" -> "lifetime"; anything else unchanged.
fn strip_plural(tag: &str) -> &str {
    tag.strip_suffix('s').unwrap_or(tag)
}

/// Trims whitespace, then leading '#'s, then one trailing 's'.
fn standard_pipeline() -> Pipeline<'static> {
    let trim = |tag: &str| tag.trim();
    let strip_hash = |tag: &str| tag.trim_start_matches('#');
    Pipeline::new()
        .step(trim)
        .step(strip_hash)
        .step(strip_plural)
}

fn main() {
    let pipeline = standard_pipeline();
    let tags = ["#Rust", " rust", "#Lifetimes", "lifetime "];
    println!("{:?}", distinct_keys(&tags, |tag| pipeline.run(tag)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinct_keys_in_first_seen_order() {
        let tags = ["#Rust", "rust", "#Lifetimes", "LIFETIME"];
        let keys = distinct_keys(&tags, |tag| tag.trim_start_matches('#'));
        assert_eq!(keys, ["rust", "lifetimes", "lifetime"]);
        assert!(distinct_keys(&[], |tag| tag).is_empty());
    }

    #[test]
    fn custom_steps() {
        let pipeline = Pipeline::new()
            .step(|tag| tag.trim_end_matches('!'))
            .step(|tag| tag.trim_start_matches('@'));
        assert_eq!(pipeline.run("@ferris!!"), "ferris");
        assert_eq!(Pipeline::new().run(" as is "), " as is ");
    }

    #[test]
    fn standard_cleanup() {
        let pipeline = standard_pipeline();
        assert_eq!(pipeline.run("  ##lifetimes "), "lifetime");
        assert_eq!(pipeline.run("rust"), "rust");
        assert_eq!(pipeline.run("#"), "");
    }

    #[test]
    fn pipeline_runs_on_local_strings() {
        let pipeline = standard_pipeline();
        let mut cleaned = Vec::new();
        for raw in ["  #Rust ", "#TAGS"] {
            let lower = raw.to_lowercase();
            cleaned.push(pipeline.run(&lower).to_string());
        }
        assert_eq!(cleaned, ["rust", "tag"]);
    }

    #[test]
    fn pipeline_as_a_key() {
        let pipeline = standard_pipeline();
        let keys = distinct_keys(&["#Tags", " tag", "TAG ", "#borrow"], |tag| {
            pipeline.run(tag)
        });
        assert_eq!(keys, ["tag", "borrow"]);
    }
}
