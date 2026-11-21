//! # Dyn compatibility and trait upcasting (reference solution)
//!
//! - What breaks dyn compatibility: generic methods, `Self` in argument or
//!   return position (other than as the receiver), associated functions
//!   without `self`, associated consts, `-> impl Trait`, and a `Sized`
//!   supertrait (so `trait Shape: Clone` is out).
//! - `where Self: Sized` on a method opts it out of the vtable instead of
//!   ruling out `dyn Filter`. `Iterator` does this for `map`, `filter` and
//!   every other adapter, which is why `Box<dyn Iterator>` works.
//! - On a `Box<dyn Filter>`, `apply_all` and `then` aren't callable. If you
//!   need them there, write a dyn-compatible version (`&[&str]` instead of
//!   a generic), or implement `Named` and `Filter` for `Box<dyn Filter>` by
//!   forwarding to `**self`: the box is `Sized`.
//! - `label(filter)` failed because a `&Box<dyn Filter>` isn't a `&dyn
//!   Filter`. `filter.as_ref()` (or `&**filter`) gets the `&dyn Filter`, and
//!   upcasting turns it into `&dyn Named`. Before Rust 1.86 that needed an
//!   `as_named(&self) -> &dyn Named` method in every impl.

/// Anything with a name.
trait Named {
    fn name(&self) -> String;
}

/// One step in a text pipeline.
trait Filter: Named {
    /// Transforms one line, or drops it by returning `None`.
    fn apply(&self, line: &str) -> Option<String>;

    /// Applies the filter to each line, keeping the ones it doesn't drop.
    fn apply_all<S: AsRef<str>>(&self, lines: &[S]) -> Vec<String>
    where
        Self: Sized,
    {
        lines
            .iter()
            .filter_map(|line| self.apply(line.as_ref()))
            .collect()
    }

    /// This filter followed by `next`, as one filter.
    fn then<F: Filter>(self, next: F) -> Chain<Self, F>
    where
        Self: Sized,
    {
        Chain {
            first: self,
            second: next,
        }
    }
}

/// Removes leading and trailing whitespace.
struct Trim;

/// Drops lines that start with '#'.
struct SkipComments;

/// Uppercases the line.
struct Upper;

/// Two filters in a row. Named "<first> > <second>".
struct Chain<A, B> {
    first: A,
    second: B,
}

impl Named for Trim {
    fn name(&self) -> String {
        "trim".to_string()
    }
}

impl Named for SkipComments {
    fn name(&self) -> String {
        "skip-comments".to_string()
    }
}

impl Named for Upper {
    fn name(&self) -> String {
        "upper".to_string()
    }
}

impl<A: Named, B: Named> Named for Chain<A, B> {
    fn name(&self) -> String {
        format!("{} > {}", self.first.name(), self.second.name())
    }
}

impl Filter for Trim {
    fn apply(&self, line: &str) -> Option<String> {
        Some(line.trim().to_string())
    }
}

impl Filter for SkipComments {
    fn apply(&self, line: &str) -> Option<String> {
        (!line.starts_with('#')).then(|| line.to_string())
    }
}

impl Filter for Upper {
    fn apply(&self, line: &str) -> Option<String> {
        Some(line.to_uppercase())
    }
}

impl<A: Filter, B: Filter> Filter for Chain<A, B> {
    fn apply(&self, line: &str) -> Option<String> {
        let halfway = self.first.apply(line)?;
        self.second.apply(&halfway)
    }
}

/// "<name>", for anything named.
fn label(item: &dyn Named) -> String {
    format!("<{}>", item.name())
}

/// Filters chosen at runtime, applied in order.
struct Pipeline {
    filters: Vec<Box<dyn Filter>>,
}

impl Pipeline {
    /// Runs each line through every filter in turn. A line that any filter
    /// drops is left out.
    fn run(&self, lines: &[&str]) -> Vec<String> {
        lines
            .iter()
            .filter_map(|line| {
                self.filters
                    .iter()
                    .try_fold(line.to_string(), |text, filter| filter.apply(&text))
            })
            .collect()
    }

    /// Each filter's label, in order.
    fn labels(&self) -> Vec<String> {
        self.filters
            .iter()
            .map(|filter| label(filter.as_ref()))
            .collect()
    }
}

fn main() {
    let pipeline = Pipeline {
        filters: vec![Box::new(Trim), Box::new(SkipComments), Box::new(Upper)],
    };
    println!("{:?}", pipeline.labels());
    println!("{:?}", pipeline.run(&["  hello ", "# note", " bye"]));
    let chain = Trim.then(Upper);
    println!("{}: {:?}", chain.name(), chain.apply_all(&[" a ", " b "]));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pipeline() -> Pipeline {
        Pipeline {
            filters: vec![Box::new(Trim), Box::new(SkipComments), Box::new(Upper)],
        }
    }

    #[test]
    fn runs_trait_objects_in_order() {
        let lines = ["  hello ", "# note", "   # indented note", " bye"];
        assert_eq!(pipeline().run(&lines), ["HELLO", "BYE"]);
        assert!(pipeline().run(&[]).is_empty());
    }

    #[test]
    fn labels_through_the_supertrait() {
        assert_eq!(
            pipeline().labels(),
            ["<trim>", "<skip-comments>", "<upper>"]
        );
        let filter: &dyn Filter = &Upper;
        assert_eq!(label(filter), "<upper>");
    }

    #[test]
    fn generic_methods_still_work_on_concrete_filters() {
        let owned = vec![String::from(" a "), String::from("# b")];
        assert_eq!(Trim.apply_all(&owned), ["a", "# b"]);
        assert_eq!(SkipComments.apply_all(&["x", "#y"]), ["x"]);
        let chain = Trim.then(SkipComments).then(Upper);
        assert_eq!(chain.name(), "trim > skip-comments > upper");
        assert_eq!(chain.apply("  ok "), Some("OK".to_string()));
        assert_eq!(chain.apply(" # no"), None);
    }

    #[test]
    fn a_chain_can_join_a_pipeline() {
        let pipeline = Pipeline {
            filters: vec![Box::new(Trim.then(Upper)), Box::new(SkipComments)],
        };
        assert_eq!(pipeline.labels(), ["<trim > upper>", "<skip-comments>"]);
        assert_eq!(pipeline.run(&[" # x ", " y"]), ["Y"]);
    }
}
