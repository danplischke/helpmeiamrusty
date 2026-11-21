//! # Closures that work for any lifetime (reference solution)
//!
//! - A lifetime parameter on a function or a struct is picked once, by the
//!   caller. `distinct_keys` calls `key` on a `String` that dies inside the
//!   loop, a lifetime no caller can name, so `key` has to accept *any*
//!   lifetime: `F: for<'x> Fn(&'x str) -> &'x str`, spelled `Fn(&str) -> &str`.
//! - The same holds for trait objects: `Box<dyn Fn(&str) -> &str>` is
//!   implicitly higher-ranked. `Pipeline<'a>` pinned every step to one
//!   lifetime, `'static` in practice, so it couldn't clean a local `String`.
//!   `run` still needs a name, because the result comes from `tag`, not
//!   from `self`.
//! - A closure in a `let` has no expected type to learn from, so its return
//!   type isn't tied to its argument ("returning this value requires that
//!   `'1` must outlive `'2`"). Annotating `-> &str` doesn't help. Pass it
//!   straight to `.step(..)`, or use a `fn` item: `str::trim` and
//!   `strip_plural` are higher-ranked already.
//! - The same limit explains why `v.sort_by_key(|p| p.name.as_str())`
//!   fails: `sort_by_key<K, F: FnMut(&T) -> K>` fixes one `K` for every
//!   call, so `K` can't borrow from the argument. Use `sort_by` there.

/// Lowercases each tag, applies `key` to it and returns the distinct keys
/// in first-seen order.
fn distinct_keys<F>(tags: &[&str], key: F) -> Vec<String>
where
    F: Fn(&str) -> &str,
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
type Step = Box<dyn Fn(&str) -> &str>;

/// Cleanup steps, applied in order.
struct Pipeline {
    steps: Vec<Step>,
}

impl Pipeline {
    fn new() -> Self {
        Pipeline { steps: Vec::new() }
    }

    /// Adds a step at the end.
    fn step(mut self, f: impl Fn(&str) -> &str + 'static) -> Self {
        self.steps.push(Box::new(f));
        self
    }

    /// Runs every step in order; with no steps, returns `tag` unchanged.
    fn run<'t>(&self, tag: &'t str) -> &'t str {
        self.steps.iter().fold(tag, |tag, step| step(tag))
    }
}

/// "lifetimes" -> "lifetime"; anything else unchanged.
fn strip_plural(tag: &str) -> &str {
    tag.strip_suffix('s').unwrap_or(tag)
}

/// Trims whitespace, then leading '#'s, then one trailing 's'.
fn standard_pipeline() -> Pipeline {
    Pipeline::new()
        .step(str::trim)
        .step(|tag| tag.trim_start_matches('#'))
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
