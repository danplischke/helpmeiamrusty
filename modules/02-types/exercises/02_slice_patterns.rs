//! # Slice patterns
//!
//! A `match` can test the *shape* of a slice: `[]`, `[x]`,
//! `[first, .., last]`, `[head, rest @ ..]`. Literals work inside
//! (`["users", id]`), and `name @ ..` binds the skipped part as a
//! sub-slice. Patterns check the length for you, so you never index.
//! Note that `[first, .., last]` needs at least two elements.
//! Implement each function with a `match` on a slice.
//!
//! Rule: no indexing or slicing (`items[0]`, `&items[1..]`).

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
    todo!("route {segments:?}")
}

/// Joins items the way you'd write them in English: "", "tea",
/// "tea and cake", "tea, cake and scones" (no comma before "and").
fn english_list(items: &[&str]) -> String {
    todo!("join {items:?}")
}

/// Whether `items` reads the same forwards and backwards. Empty and
/// one-element slices do. Compare the two ends, then recurse on the middle.
fn is_palindrome<T: PartialEq>(items: &[T]) -> bool {
    let _ = items; // only here to silence "unused variable"
    todo!("check whether the slice is a palindrome")
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
