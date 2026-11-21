//! # Who does the result borrow from?
//!
//! In a method, elision ties every returned reference to `&self`. Often
//! that's right. Here each method's result comes from somewhere specific:
//! the `path` argument, `self` alone, or string literals (`'static`). Fix
//! the signatures so each one says where its result points. The tests drop
//! the router or the path early to check.

struct Router {
    base: String,
    routes: Vec<(String, String)>,
}

impl Router {
    fn new(base: &str) -> Self {
        Router {
            base: base.to_string(),
            routes: Vec::new(),
        }
    }

    /// Adds a route from `path` (relative to the base) to a handler name.
    fn route(mut self, path: &str, handler: &str) -> Self {
        self.routes.push((path.to_string(), handler.to_string()));
        self
    }

    /// The part of `path` after the base, as a slice of `path`:
    /// "/app/users" gives "/users" and "/app" gives "/". `None` for paths
    /// outside the base, such as "/other" or "/application".
    fn local_path(&self, path: &str) -> Option<&str> {
        match path.strip_prefix(self.base.as_str())? {
            "" => Some("/"),
            rest if rest.starts_with('/') => Some(rest),
            _ => None,
        }
    }

    /// The handler name for a full `path`, if a route matches it exactly.
    fn handler<'a>(&'a self, path: &'a str) -> Option<&'a str> {
        let local = self.local_path(path)?;
        self.routes
            .iter()
            .find(|(route, _)| route == local)
            .map(|(_, handler)| handler.as_str())
    }

    /// "api" for local paths under "/api/", "static" under "/static/",
    /// "page" for other local paths and "external" outside the base.
    fn section(&self, path: &str) -> &str {
        match self.local_path(path) {
            Some(local) if local.starts_with("/api/") => "api",
            Some(local) if local.starts_with("/static/") => "static",
            Some(_) => "page",
            None => "external",
        }
    }
}

fn main() {
    let router = Router::new("/app")
        .route("/", "home")
        .route("/users", "list_users");
    for path in ["/app/users", "/app/api/users", "/elsewhere"] {
        println!(
            "{path}: {:?} {:?} {}",
            router.local_path(path),
            router.handler(path),
            router.section(path)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn router() -> Router {
        Router::new("/app")
            .route("/", "home")
            .route("/users", "list_users")
            .route("/api/users", "users_json")
    }

    #[test]
    fn local_paths() {
        let router = router();
        assert_eq!(router.local_path("/app/users/7"), Some("/users/7"));
        assert_eq!(router.local_path("/app"), Some("/"));
        assert_eq!(router.local_path("/application"), None);
        assert_eq!(router.local_path("/other/app"), None);
    }

    #[test]
    fn local_path_outlives_the_router() {
        let path = String::from("/app/users/7");
        let local = {
            let router = router();
            router.local_path(&path)
        };
        assert_eq!(local, Some("/users/7"));
    }

    #[test]
    fn handler_outlives_the_path() {
        let router = router();
        let handler;
        {
            let path = format!("/app/{}", "users");
            handler = router.handler(&path);
        }
        assert_eq!(handler, Some("list_users"));
        assert_eq!(router.handler("/app"), Some("home"));
        assert_eq!(router.handler("/app/nope"), None);
        assert_eq!(router.handler("/users"), None);
    }

    #[test]
    fn section_names_outlive_the_router() {
        let paths = [
            "/app/api/users",
            "/app/static/logo.png",
            "/app/users",
            "/api/x",
        ];
        let sections: Vec<&str> = {
            let router = router();
            paths.iter().map(|path| router.section(path)).collect()
        };
        assert_eq!(sections, ["api", "static", "page", "external"]);
    }
}
