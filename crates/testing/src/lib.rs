use rustweb_core::{Component, ComponentError, Context, ContextMap, Link, VNode};
use std::fmt::Debug;

pub mod a11y;

pub use a11y::{check as check_a11y, A11yReport, A11yViolation, Severity};

pub fn render_to_sexpr(node: &VNode) -> String {
    node.to_sexpr()
}

pub fn assert_snapshot(name: &str, node: &VNode, expected_sexpr: &str) {
    let actual = node.to_sexpr();
    if std::env::var("UPDATE_SNAPSHOTS").as_deref() == Ok("1") {
        let dir = std::path::Path::new("snapshots");
        let _ = std::fs::create_dir_all(dir);
        let _ = std::fs::write(dir.join(format!("{name}.snap")), &actual);
        return;
    }
    assert_eq!(actual, expected_sexpr, "snapshot `{name}` mismatch.\n  actual:   {actual}\n  expected: {expected_sexpr}\n  hint: run with UPDATE_SNAPSHOTS=1 to bless, then `git diff snapshots/`");
}

pub fn assert_json_snapshot(node: &VNode, expected_json: &str) {
    let actual: serde_json::Value = serde_json::from_str(&node.to_snapshot_json()).unwrap();
    let expected: serde_json::Value = serde_json::from_str(expected_json).unwrap();
    assert_eq!(
        actual,
        expected,
        "JSON snapshot mismatch:\nactual:\n{}\n",
        node.to_snapshot_json()
    );
}

pub struct TestHarness<C: Component> {
    ctx: Context<C>,
    inbox: std::rc::Rc<std::cell::RefCell<Vec<C::Msg>>>,
}

impl<C: Component> TestHarness<C> {
    pub fn new(props: C::Props) -> Self {
        Self::with_contexts(props, ContextMap::new())
    }

    pub fn with_contexts(props: C::Props, contexts: ContextMap) -> Self {
        let inbox = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let inbox2 = inbox.clone();
        let link = Link::new(move |msg| inbox2.borrow_mut().push(msg));
        let ctx = Context::new(props, link, contexts);
        Self { ctx, inbox }
    }

    pub fn ctx(&self) -> &Context<C> {
        &self.ctx
    }

    pub fn sent(&self) -> Vec<String>
    where
        C::Msg: Debug,
    {
        self.inbox
            .borrow()
            .iter()
            .map(|m| format!("{m:?}"))
            .collect()
    }

    pub fn create(&self) -> Result<C, ComponentError> {
        let loc = std::panic::Location::caller();
        rustweb_core::runtime::run_guarded::<C, C>(C::name(), "create", loc, || {
            C::create(&self.ctx)
        })
    }

    pub fn view(&self, state: &C) -> Result<VNode, ComponentError> {
        let loc = std::panic::Location::caller();
        match rustweb_core::runtime::run_render_guarded(loc, || state.view(&self.ctx)) {
            Ok(v) => Ok(v),
            Err(e) => Err(ComponentError::Render(e)),
        }
    }

    pub fn update(
        &self,
        state: &mut C,
        msg: C::Msg,
    ) -> Result<rustweb_core::Cmd<C>, ComponentError> {
        let loc = std::panic::Location::caller();
        let msg_dbg = format!("{:?}", &msg as &dyn Debug);
        rustweb_core::runtime::run_guarded::<C, _>(C::name(), "update", loc, || {
            state.update(&self.ctx, msg).map_err(|e| match e {
                ComponentError::Update { .. } => e,
                other => ComponentError::Update {
                    component: C::name().into(),
                    msg: msg_dbg.clone(),
                    message: other.to_string(),
                },
            })
        })
    }

    pub fn view_to_string(&self, state: &C) -> Result<String, ComponentError> {
        Ok(rustweb_ssr::render_to_string(&self.view(state)?))
    }
}

pub mod headless {

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Browser {
        Chrome,

        Firefox,

        Safari,
    }

    #[derive(Debug, Clone)]
    pub struct HeadlessConfig {
        pub browser: Browser,

        pub webdriver_url: String,

        pub headless: bool,
    }

    impl Default for HeadlessConfig {
        fn default() -> Self {
            let browser = match std::env::var("HEADLESS_BROWSER").as_deref() {
                Ok("firefox") => Browser::Firefox,
                Ok("safari") => Browser::Safari,
                _ => Browser::Chrome,
            };
            Self {
                browser,
                webdriver_url: std::env::var("WEBDRIVER_URL")
                    .unwrap_or_else(|_| "http://localhost:4444".into()),
                headless: std::env::var("HEADLESS").as_deref() != Ok("0"),
            }
        }
    }

    pub fn require_headless() -> HeadlessConfig {
        if std::env::var("HEADLESS").as_deref() != Ok("1") {
            eprintln!("skipping browser test (set HEADLESS=1 with a WebDriver at $WEBDRIVER_URL)");
        }
        HeadlessConfig::default()
    }
}
