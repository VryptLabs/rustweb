use rustweb_core::{Attr, Cmd, Component, Context, Props, RenderError, VNode};
use rustweb_macro::html;
use rustweb_router::{GuardResult, LazyRoute, Route, Router};

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
        Ok(Self { todos: ctx.props.initial.clone(), next_id })
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Msg) -> Result<Cmd<Self>, rustweb_core::ComponentError> {
        match msg {
            Msg::Add(text) => {
                self.todos.push(Todo { id: self.next_id, text, done: false });
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
        Ok(html! {
            <main aria-label="Todo app">
                <h1>{ "Todos" }</h1>
                <ul>{ rows }</ul>
                <button aria-pressed="false">{ "Add" }</button>
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
                    if ctx.principal.is_some() { GuardResult::Allow } else { GuardResult::Redirect("/login".into()) }
                })
                .child(Route::new(":id", "TodoDetail")),
        )
        .route(Route::new("login", "Login"))
        .route(Route::new("settings", "Settings").lazy_children(LazyRoute::new(|| Ok(vec![Route::new("profile", "Profile")]))))
}

fn main() {
    let link = rustweb_core::Link::new(|_: Msg| {});
    let ctx = Context::new(
        AppProps {
            initial: vec![
                Todo { id: 0, text: "buy milk".into(), done: false },
                Todo { id: 1, text: "write docs".into(), done: true },
            ],
            children: vec![],
        },
        link,
        Default::default(),
    );
    let state = App::create(&ctx).expect("create");
    let tree = state.view(&ctx).expect("view");

    let html = rustweb_ssr::render_to_string(&tree);
    println!("{html}");

    rustweb_ssr::verify_hydration(&html, &tree).expect("hydrate");

    let hit = router().resolve("/todos/1").unwrap_or_else(|_| router().resolve("/login").unwrap());
    eprintln!("route → {hit:?}");
    let _ = Attr::new("demo", "attr");
}
