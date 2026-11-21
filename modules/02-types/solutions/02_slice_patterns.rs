//! # Slice patterns (reference solution)
//!
//! - Match a `Vec` through `.as_slice()`: slice patterns match slices and
//!   arrays, not `Vec` itself.
//! - `rest @ ..` is the one place `..` can be bound, and it binds a
//!   sub-slice. To insist it isn't empty, add a guard (as here) or put a
//!   `["static"] => Route::NotFound` arm before it. Careful:
//!   `rest @ [_, ..]` in that position would be a single *element* matched
//!   against a nested slice pattern (E0529 on `&str`).
//! - Slice patterns are checked for exhaustiveness by length: `[] | [_]`
//!   plus `[first, middle @ .., last]` covers every slice, so
//!   `is_palindrome` needs no `_` arm.
//! - In `english_list`, the `..` goes first: `[init @ .., last]` splits off
//!   the last element, and two items need no arm of their own. Arm order
//!   matters: it also matches a single item (with an empty `init`), so
//!   `[only]` has to come before it.

#[derive(Debug, PartialEq)]
enum Route {
    Home,
    User(u32),
    UserPosts(u32),
    Asset(String),
    NotFound,
}

/// Routes a URL path. Empty segments are ignored, so "/users//7/" is the
/// same as "/users/7".
///
/// - "/" is `Home`.
/// - "/users/<id>" is `User(id)` and "/users/<id>/posts" is `UserPosts(id)`,
///   if `<id>` parses as a `u32`. Otherwise they're `NotFound`.
/// - "/static/<path>" is `Asset`, with the segments after "static" joined
///   by "/": "/static/css/site.css" is `Asset("css/site.css")`. Plain
///   "/static" is `NotFound`.
/// - Everything else is `NotFound`.
fn route(path: &str) -> Route {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    match segments.as_slice() {
        [] => Route::Home,
        ["users", id] => id.parse().map_or(Route::NotFound, Route::User),
        ["users", id, "posts"] => id.parse().map_or(Route::NotFound, Route::UserPosts),
        ["static", rest @ ..] if !rest.is_empty() => Route::Asset(rest.join("/")),
        _ => Route::NotFound,
    }
}

/// Joins items the way you'd write them in English: "", "tea",
/// "tea and cake", "tea, cake and scones" (no comma before "and").
fn english_list(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [only] => only.to_string(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

/// Whether `items` reads the same forwards and backwards. Empty and
/// one-element slices do. Compare the two ends, then recurse on the middle.
fn is_palindrome<T: PartialEq>(items: &[T]) -> bool {
    match items {
        [] | [_] => true,
        [first, middle @ .., last] => first == last && is_palindrome(middle),
    }
}

fn main() {
    for path in ["/", "/users/42/posts", "/static/img/logo.png", "/users/me"] {
        println!("{path} -> {:?}", route(path));
    }
    println!("{}", english_list(&["borrowck", "traits", "patterns"]));
    let word: Vec<char> = "racecar".chars().collect();
    println!("palindrome: {}", is_palindrome(&word));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_home_and_users() {
        assert_eq!(route("/"), Route::Home);
        assert_eq!(route(""), Route::Home);
        assert_eq!(route("/users/7"), Route::User(7));
        assert_eq!(route("/users//7/"), Route::User(7));
        assert_eq!(route("/users/7/posts"), Route::UserPosts(7));
    }

    #[test]
    fn routes_assets() {
        assert_eq!(
            route("/static/css/site.css"),
            Route::Asset("css/site.css".to_string())
        );
        assert_eq!(
            route("/static/robots.txt"),
            Route::Asset("robots.txt".to_string())
        );
        assert_eq!(route("/static"), Route::NotFound);
        assert_eq!(route("/static/"), Route::NotFound);
    }

    #[test]
    fn everything_else_is_not_found() {
        for path in [
            "/users",
            "/users/me",
            "/users/-1",
            "/users/7/comments",
            "/users/7/posts/1",
            "/admin",
            "/css/static",
        ] {
            assert_eq!(route(path), Route::NotFound, "{path}");
        }
    }

    #[test]
    fn english_lists() {
        assert_eq!(english_list(&[]), "");
        assert_eq!(english_list(&["tea"]), "tea");
        assert_eq!(english_list(&["tea", "cake"]), "tea and cake");
        assert_eq!(
            english_list(&["tea", "cake", "scones"]),
            "tea, cake and scones"
        );
        assert_eq!(english_list(&["a", "b", "c", "d"]), "a, b, c and d");
    }

    #[test]
    fn palindromes() {
        let chars = |s: &str| s.chars().collect::<Vec<_>>();
        assert!(is_palindrome(&chars("racecar")));
        assert!(is_palindrome(&chars("abba")));
        assert!(is_palindrome(&chars("x")));
        assert!(is_palindrome::<char>(&[]));
        assert!(!is_palindrome(&chars("abca")));
        assert!(!is_palindrome(&chars("ab")));
        assert!(is_palindrome(&[1, 2, 3, 2, 1]));
        assert!(!is_palindrome(&[1, 2, 3, 1, 1]));
    }
}
