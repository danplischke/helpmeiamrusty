//! # Dyn compatibility and trait upcasting
//!
//! `dyn Filter` only works if every method can be called through a vtable,
//! without knowing the concrete type ("dyn compatible", formerly "object
//! safe"). A generic method would need a vtable entry per type argument; a
//! method that takes or returns `Self` by value needs to know its size.
//!
//! You don't have to give such methods up. `where Self: Sized` on a method
//! leaves it out of the vtable: concrete types keep it, `dyn Filter`
//! doesn't get it. Separately, since Rust 1.86 a `&dyn Filter` converts to
//! a `&dyn Named` (trait upcasting), because `Named` is a supertrait.
//!
//! Make this compile without changing what it does.

/// Anything with a name.
trait Named {
    fn name(&self) -> String;
}

/// One step in a text pipeline.
trait Filter: Named {
    /// Transforms one line, or drops it by returning `None`.
    fn apply(&self, line: &str) -> Option<String>;

    /// Applies the filter to each line, keeping the ones it doesn't drop.
    fn apply_all<S: AsRef<str>>(&self, lines: &[S]) -> Vec<String> {
        lines
            .iter()
            .filter_map(|line| self.apply(line.as_ref()))
            .collect()
    }

    /// This filter followed by `next`, as one filter.
    fn then<F: Filter>(self, next: F) -> Chain<Self, F> {
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
        self.filters.iter().map(|filter| label(filter)).collect()
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
