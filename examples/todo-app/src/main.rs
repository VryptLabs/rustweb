use rustweb_core::{Attr, Cmd, Component, Context, Props, RenderError, VNode};
use rustweb_dom::{diff, Renderer};
use rustweb_macro::html;
use rustweb_router::{GuardResult, LazyRoute, Route, Router};
use rustweb_ui::{
    render_button, render_dialog, render_input, render_tabs, ButtonProps, ButtonVariant,
    DialogProps, InputProps, TabsProps,
};

#[derive(Clone, PartialEq, Debug)]
struct Todo {
    id: usize,
    text: String,
    done: bool,
}

#[derive(Clone, PartialEq, Debug)]
struct AppProps {
    initial: Vec<Todo>,
    children: Vec<VNode>,
}
impl Props for AppProps {}

struct App {
    todos: Vec<Todo>,
    next_id: usize,
}

#[derive(Debug)]
enum Msg {
    Add(String),
    Toggle(usize),
}

impl Component for App {
    type Props = AppProps;
    type Msg = Msg;

    fn create(ctx: &Context<Self>) -> Result<Self, rustweb_core::ComponentError> {
        let next_id = ctx.props.initial.len();
        Ok(Self {
            todos: ctx.props.initial.clone(),
            next_id,
        })
    }

    fn update(
        &mut self,
        _ctx: &Context<Self>,
        msg: Msg,
    ) -> Result<Cmd<Self>, rustweb_core::ComponentError> {
        match msg {
            Msg::Add(text) => {
                self.todos.push(Todo {
                    id: self.next_id,
                    text,
                    done: false,
                });
                self.next_id += 1;
            }
            Msg::Toggle(id) => {
                if let Some(t) = self.todos.iter_mut().find(|t| t.id == id) {
                    t.done = !t.done;
                }
            }
        }
        Ok(Cmd::Render)
    }

    fn view(&self, _ctx: &Context<Self>) -> Result<VNode, RenderError> {
        let rows: Vec<VNode> = self
            .todos
            .iter()
            .map(|t| {
                html! {
                    <li key={ t.id } aria-selected={ t.done }>
                        { &t.text }
                    </li>
                }
            })
            .collect();
        let input = render_input(InputProps {
            id: "new-todo".into(),
            value: "".into(),
            placeholder: Some("What needs to be done?".into()),
            label: Some("New todo".into()),
            error: None,
            disabled: false,
            children: vec![],
        });
        let tabs = render_tabs(TabsProps {
            tabs: vec!["All".into(), "Active".into(), "Done".into()],
            selected: 0,
            children: vec![
                html! { <div>{ rows.clone() }</div> },
                html! { <div>{ VNode::text("Active filter") }</div> },
                html! { <div>{ VNode::text("Done filter") }</div> },
            ],
        });
        let btn = render_button(ButtonProps {
            label: "Add".into(),
            variant: ButtonVariant::Primary,
            disabled: false,
            loading: false,
            aria_label: None,
            children: vec![],
        });
        let dialog = render_dialog(DialogProps {
            open: false,
            title: "Add todo".into(),
            children: vec![VNode::text("Dialog content")],
        });
        Ok(html! {
            <main aria-label="Todo app">
                <h1>{ "Todos" }</h1>
                { input }
                { tabs }
                { btn }
                { dialog }
            </main>
        })
    }
}

fn router() -> Router {
    Router::new()
        .route(Route::new("", "Home"))
        .route(
            Route::new("todos", "TodoList")
                .guard(|ctx| {
                    if ctx.principal.is_some() {
                        GuardResult::Allow
                    } else {
                        GuardResult::Redirect("/login".into())
                    }
                })
                .child(Route::new(":id", "TodoDetail")),
        )
        .route(Route::new("login", "Login"))
        .route(
            Route::new("settings", "Settings").lazy_children(LazyRoute::new(|| {
                Ok(vec![Route::new("profile", "Profile")])
            })),
        )
}

fn main() {
    let link = rustweb_core::Link::new(|_: Msg| {});
    let ctx = Context::new(
        AppProps {
            initial: vec![
                Todo {
                    id: 0,
                    text: "buy milk".into(),
                    done: false,
                },
                Todo {
                    id: 1,
                    text: "write docs".into(),
                    done: true,
                },
            ],
            children: vec![],
        },
        link,
        Default::default(),
    );
    let mut state = App::create(&ctx).expect("create");
    let mut renderer = Renderer::new(state.view(&ctx).expect("view"));

    let msgs = [Msg::Add("learn rustweb".into()), Msg::Toggle(0)];
    for msg in msgs {
        state.update(&ctx, msg).expect("update");
        let next = state.view(&ctx).expect("view");
        let patches = diff(renderer.current(), &next);
        eprintln!("msg → {} patches", patches.len());
        renderer.update("App", next).expect("patch");
    }

    let html = rustweb_ssr::render_to_string(renderer.current());
    println!("=== final SSR ===\n{html}");
    rustweb_ssr::verify_hydration(&html, renderer.current()).expect("hydrate");

    let hit = router()
        .resolve("/todos/1")
        .unwrap_or_else(|_| router().resolve("/login").unwrap());
    eprintln!("route → {hit:?}");
    let _ = Attr::new("demo", "attr");
}

#[cfg(test)]
mod a11y_tests {
    use super::*;
    use rustweb_testing::{check_a11y, Severity};

    fn initial_props() -> AppProps {
        AppProps {
            initial: vec![
                Todo {
                    id: 0,
                    text: "buy milk".into(),
                    done: false,
                },
                Todo {
                    id: 1,
                    text: "write docs".into(),
                    done: true,
                },
            ],
            children: vec![],
        }
    }

    fn app_view() -> VNode {
        let link = rustweb_core::Link::new(|_: Msg| {});
        let ctx = Context::new(initial_props(), link, Default::default());
        let state = App::create(&ctx).expect("create");
        state.view(&ctx).expect("view")
    }

    #[test]
    fn example_is_a11y_clean() {
        let report = check_a11y(&app_view());
        assert!(
            report.is_clean(),
            "example failed a11y audit:\n{}",
            report.to_text()
        );
    }

    #[test]
    fn empty_button_would_be_caught() {
        let broken = html! { <main aria-label="x"><button></button></main> };
        let report = check_a11y(&broken);
        assert!(report.has_blocking());
        assert!(report
            .violations
            .iter()
            .any(|v| v.rule == "interactive-name"));
        assert_eq!(report.of_severity(Severity::Serious).len(), 1);
    }
}
