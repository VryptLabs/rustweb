use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RouterError {
    #[error("no route matched for path `{0}`")]
    NotFound(String),

    #[error("navigation to `{path}` denied: {reason}")]
    Forbidden { path: String, reason: String },

    #[error("redirect loop resolving `{0}`")]
    RedirectLoop(String),

    #[error("failed to load lazy route `{route}`: {message}")]
    LazyLoad { route: String, message: String },
}

#[derive(Debug, Clone, Default)]
pub struct RouteContext {
    pub path: String,

    pub params: HashMap<String, String>,

    pub query: HashMap<String, String>,

    pub principal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardResult {
    Allow,

    Redirect(String),

    Deny(String),
}

pub type Guard = Rc<dyn Fn(&RouteContext) -> GuardResult>;

#[derive(Clone)]
pub struct Route {
    pub segment: String,

    pub name: String,

    pub view: String,

    pub guards: Vec<Guard>,

    pub children: Vec<Route>,

    pub lazy: Option<Rc<LazyRoute>>,
}

impl Debug for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Route")
            .field("segment", &self.segment)
            .field("name", &self.name)
            .field("view", &self.view)
            .field("guards", &format!("{} guard(s)", self.guards.len()))
            .field("children", &self.children)
            .field("lazy", &self.lazy.is_some())
            .finish()
    }
}

impl Route {
    pub fn new(segment: impl Into<String>, view: impl Into<String>) -> Self {
        let seg = segment.into();
        let v = view.into();
        let name = format!("{seg}->{v}");
        Self {
            segment: seg,
            name,
            view: v,
            guards: Vec::new(),
            children: Vec::new(),
            lazy: None,
        }
    }

    pub fn guard(mut self, g: impl Fn(&RouteContext) -> GuardResult + 'static) -> Self {
        self.guards.push(Rc::new(g));
        self
    }

    pub fn child(mut self, c: Route) -> Self {
        self.children.push(c);
        self
    }

    pub fn lazy_children(mut self, loader: LazyRoute) -> Self {
        self.lazy = Some(Rc::new(loader));
        self
    }
}

type LazyFactory = Box<dyn FnOnce() -> Result<Vec<Route>, String>>;

pub struct LazyRoute {
    cell: OnceCell<Vec<Route>>,
    factory: RefCell<Option<LazyFactory>>,
}

impl Clone for LazyRoute {
    fn clone(&self) -> Self {
        Self {
            cell: OnceCell::new(),
            factory: RefCell::new(None),
        }
    }
}

impl Debug for LazyRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyRoute")
            .field("loaded", &self.cell.get().is_some())
            .finish()
    }
}

impl LazyRoute {
    pub fn new(factory: impl FnOnce() -> Result<Vec<Route>, String> + 'static) -> Self {
        Self {
            cell: OnceCell::new(),
            factory: RefCell::new(Some(Box::new(factory))),
        }
    }

    pub fn load(&self) -> Result<&Vec<Route>, RouterError> {
        if self.cell.get().is_none() {
            let f = self
                .factory
                .borrow_mut()
                .take()
                .ok_or_else(|| RouterError::LazyLoad {
                    route: "<lazy>".into(),
                    message: "factory already consumed (route cloned after load?)".into(),
                })?;
            match f() {
                Ok(routes) => {
                    let _ = self.cell.set(routes);
                }
                Err(message) => {
                    return Err(RouterError::LazyLoad {
                        route: "<lazy>".into(),
                        message,
                    })
                }
            }
        }
        Ok(self.cell.get().expect("just set"))
    }

    pub fn is_loaded(&self) -> bool {
        self.cell.get().is_some()
    }
}

#[derive(Debug, Clone)]
pub struct Resolved {
    pub view: String,

    pub chain: Vec<String>,

    pub params: HashMap<String, String>,

    pub query: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn route(mut self, r: Route) -> Self {
        self.routes.push(r);
        self
    }

    pub fn resolve(&self, path: &str) -> Result<Resolved, RouterError> {
        let mut current = path.to_string();
        for _ in 0..8 {
            match self.resolve_once(&current)? {
                ResolveStep::Hit(r) => return Ok(r),
                ResolveStep::Redirect(to) => current = to,
            }
        }
        Err(RouterError::RedirectLoop(path.to_string()))
    }

    fn resolve_once(&self, path: &str) -> Result<ResolveStep, RouterError> {
        let (path_part, query) = split_query(path);
        let segs: Vec<&str> = path_part.split('/').filter(|s| !s.is_empty()).collect();
        let mut ctx = RouteContext {
            path: path.to_string(),
            params: HashMap::new(),
            query,
            principal: None,
        };
        let mut chain: Vec<(Guard, String)> = Vec::new();
        let view = self.match_tree(&self.routes, &segs, 0, &mut ctx, &mut chain)?;
        for (guard, _name) in &chain {
            match guard(&ctx) {
                GuardResult::Allow => {}
                GuardResult::Redirect(to) => return Ok(ResolveStep::Redirect(to)),
                GuardResult::Deny(reason) => {
                    return Err(RouterError::Forbidden {
                        path: path.to_string(),
                        reason,
                    });
                }
            }
        }
        let names = chain.iter().map(|(_, n)| n.clone()).collect();
        Ok(ResolveStep::Hit(Resolved {
            view,
            chain: names,
            params: ctx.params,
            query: ctx.query,
        }))
    }

    fn match_tree(
        &self,
        routes: &[Route],
        segs: &[&str],
        pos: usize,
        ctx: &mut RouteContext,
        chain: &mut Vec<(Guard, String)>,
    ) -> Result<String, RouterError> {
        for r in routes {
            let saved_params = ctx.params.clone();
            let saved_chain = chain.len();
            let adv = match advance(&r.segment, segs, pos) {
                Some(a) => a,
                None => continue,
            };
            if r.segment == "*" {
                ctx.params.insert("wildcard".into(), segs[pos..].join("/"));
            } else {
                capture_param(&r.segment, segs.get(pos).copied(), &mut ctx.params);
            }
            for g in &r.guards {
                chain.push((g.clone(), r.name.clone()));
            }
            let next = pos + adv;
            let mut kids = r.children.clone();
            if let Some(lazy) = &r.lazy {
                kids.extend(lazy.load()?.clone());
            }
            if next == segs.len() {
                let index_hit = kids
                    .iter()
                    .find(|k| k.segment.is_empty() || k.segment == "/")
                    .cloned();
                if let Some(idx) = index_hit {
                    let saved_chain_idx = chain.len();
                    for g in &idx.guards {
                        chain.push((g.clone(), idx.name.clone()));
                    }
                    let mut sub_kids = idx.children.clone();
                    if let Some(lazy) = &idx.lazy {
                        sub_kids.extend(lazy.load()?.clone());
                    }
                    if sub_kids.is_empty() {
                        return Ok(idx.view.clone());
                    }
                    match self.match_tree(&sub_kids, segs, next, ctx, chain) {
                        Ok(v) => return Ok(v),
                        Err(RouterError::NotFound(_)) => {
                            chain.truncate(saved_chain_idx);
                        }
                        Err(e) => return Err(e),
                    }
                }
                return Ok(r.view.clone());
            }
            match self.match_tree(&kids, segs, next, ctx, chain) {
                Ok(view) => return Ok(view),
                Err(RouterError::NotFound(_)) => {
                    ctx.params = saved_params;
                    chain.truncate(saved_chain);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        Err(RouterError::NotFound(format!("/{}", segs.join("/"))))
    }
}

enum ResolveStep {
    Hit(Resolved),
    Redirect(String),
}

fn advance(pattern: &str, segs: &[&str], pos: usize) -> Option<usize> {
    let at_end = pos >= segs.len();
    if pattern.is_empty() || pattern == "/" {
        return Some(0);
    }
    if pattern == "*" {
        return Some(segs.len() - pos);
    }
    if let Some(name) = pattern.strip_prefix(':') {
        if name.is_empty() || at_end {
            return None;
        }
        return Some(1);
    }
    match segs.get(pos) {
        Some(c) if *c == pattern => Some(1),
        _ => None,
    }
}

fn capture_param(pattern: &str, current: Option<&str>, params: &mut HashMap<String, String>) {
    if let Some(name) = pattern.strip_prefix(':') {
        if !name.is_empty() {
            if let Some(c) = current {
                params.insert(name.to_string(), url_decode(c));
            }
        }
    }
}

fn split_query(path: &str) -> (String, HashMap<String, String>) {
    let mut q = HashMap::new();
    match path.split_once('?') {
        Some((p, qs)) => {
            for pair in qs.split('&') {
                if pair.is_empty() {
                    continue;
                }
                match pair.split_once('=') {
                    Some((k, v)) => {
                        q.insert(url_decode(k), url_decode(v));
                    }
                    None => {
                        q.insert(url_decode(pair), String::new());
                    }
                }
            }
            (p.to_string(), q)
        }
        None => (path.to_string(), q),
    }
}

fn url_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h: String = chars.by_ref().take(2).collect();
            if let Ok(b) = u8::from_str_radix(&h, 16) {
                out.push(b as char);
            } else {
                out.push('%');
                out.push_str(&h);
            }
        } else if c == '+' {
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn app_router() -> Router {
        Router::new()
            .route(Route::new("", "Home"))
            .route(
                Route::new("users", "Users")
                    .guard(|ctx| {
                        if ctx.principal.is_some() {
                            GuardResult::Allow
                        } else {
                            GuardResult::Redirect("/login".into())
                        }
                    })
                    .child(Route::new(":id", "UserDetail"))
                    .child(Route::new(":id/edit", "UserEdit")),
            )
            .route(Route::new("login", "Login"))
    }

    #[test]
    fn nested_match_with_params() {
        let r = Router::new().route(
            Route::new("users", "Users")
                .child(Route::new(":id", "UserDetail").child(Route::new("edit", "UserEdit"))),
        );
        let hit = r.resolve("/users/42/edit").unwrap();
        assert_eq!(hit.view, "UserEdit");
        assert_eq!(hit.params.get("id").map(|s| s.as_str()), Some("42"));
    }

    #[test]
    fn guard_redirect() {
        let r = Router::new()
            .route(Route::new("admin", "Admin").guard(|_| GuardResult::Redirect("/login".into())))
            .route(Route::new("login", "Login"));
        let hit = r.resolve("/admin").unwrap();
        assert_eq!(hit.view, "Login");
    }

    #[test]
    fn guard_deny_is_forbidden() {
        let r = Router::new()
            .route(Route::new("secret", "S").guard(|_| GuardResult::Deny("nope".into())));
        assert!(matches!(
            r.resolve("/secret"),
            Err(RouterError::Forbidden { .. })
        ));
    }

    #[test]
    fn not_found() {
        let r = app_router();
        assert!(matches!(
            r.resolve("/missing"),
            Err(RouterError::NotFound(_))
        ));
    }

    #[test]
    fn lazy_loads_once() {
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        let lazy = LazyRoute::new(|| {
            CALLS.fetch_add(1, Ordering::SeqCst);
            Ok(vec![Route::new("profile", "Profile")])
        });
        let r = Router::new().route(Route::new("settings", "Settings").lazy_children(lazy));
        assert_eq!(r.resolve("/settings/profile").unwrap().view, "Profile");
        assert_eq!(r.resolve("/settings/profile").unwrap().view, "Profile");
        assert_eq!(
            CALLS.load(Ordering::SeqCst),
            1,
            "factory must run exactly once"
        );
    }

    #[test]
    fn wildcard() {
        let r = Router::new().route(Route::new("files", "F").child(Route::new("*", "FileView")));
        let hit = r.resolve("/files/a/b/c").unwrap();
        assert_eq!(hit.view, "FileView");
        assert_eq!(
            hit.params.get("wildcard").map(|s| s.as_str()),
            Some("a/b/c")
        );
    }

    #[test]
    fn nested_index_recurses_into_grand_children() {
        let r =
            Router::new()
                .route(Route::new("app", "App").child(
                    Route::new("", "AppIndex").child(Route::new("settings", "AppSettings")),
                ));
        let hit = r.resolve("/app/settings").unwrap();
        assert_eq!(
            hit.view, "AppSettings",
            "grand index child must be reachable"
        );
    }

    #[test]
    fn index_without_children_returns_own_view() {
        let r = Router::new().route(Route::new("app", "App").child(Route::new("", "AppIndex")));
        let hit = r.resolve("/app").unwrap();
        assert_eq!(hit.view, "AppIndex");
    }
}
