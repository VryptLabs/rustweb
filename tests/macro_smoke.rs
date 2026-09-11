use rustweb_core::{Attr, Component, Context, Cmd, Props, RenderError, VNode};
use rustweb_macro::html;

#[test]
fn element_with_aria_and_text() {
    let title = "Hello";
    let node = html! {
        <main class="app" aria-label="Todo app">
            <h1>{ title }</h1>
            <button type="button" aria-pressed="false" disabled={ false }>"Add"</button>
        </main>
    };
    assert_eq!(node.tag(), Some("main"));
    if let VNode::Element(el) = &node {
        assert!(el.attrs.iter().any(|a| a.name == "aria-label"));
        assert_eq!(el.children.len(), 2);
    } else {
        panic!("expected element, got {node:?}");
    }
}

#[test]
fn conditional_and_list_children() {
    let logged_in = true;
    let items = vec!["a", "b"];
    let node = html! {
        <div>
            { if logged_in { html!{ <span>"hi"</span> } } else { html!{ <a href="/login">"login"</a> } } }
            { items.iter().map(|t| html!{ <li>{ t }</li> }).collect::<Vec<_>>() }
        </div>
    };
    if let VNode::Element(el) = &node {

        assert_eq!(el.children.len(), 3, "{}", node.to_sexpr());
    } else {
        panic!("expected div");
    }
}

#[test]
fn fragment_and_key() {
    let node = html! {
        <>
            <li key={ 1 }>"one"</li>
            <li key={ 2 }>"two"</li>
        </>
    };
    assert!(matches!(node, VNode::Fragment(_)));
    if let VNode::Fragment(c) = &node {
        assert_eq!(c[0].key(), Some("1"));
    }
}

#[derive(Clone, PartialEq, Debug)]
struct ButtonProps {
    label: String,
    disabled: bool,
}
impl Props for ButtonProps {
    fn validate(&self) -> Result<(), rustweb_core::PropsError> {
        if self.label.is_empty() {
            return Err(rustweb_core::PropsError::Invalid {
                component: "Button".into(),
                prop: "label".into(),
                hint: "label must be non-empty".into(),
            });
        }
        Ok(())
    }
}
struct Button;
impl Component for Button {
    type Props = ButtonProps;
    type Msg = ();
    fn create(_ctx: &Context<Self>) -> Result<Self, rustweb_core::ComponentError> {
        Ok(Button)
    }
    fn update(&mut self, _ctx: &Context<Self>, _msg: ()) -> Result<Cmd<Self>, rustweb_core::ComponentError> {
        Ok(Cmd::None)
    }
    fn view(&self, ctx: &Context<Self>) -> Result<VNode, RenderError> {
        Ok(html! { <button disabled={ ctx.props.disabled }>{ &ctx.props.label }</button> })
    }
}

#[test]
fn component_tag_builds_typed_props() {
    let node = html! { <Button label="Save" disabled={ false } /> };
    let (name, _props_json, inner) = node.as_component().expect("component node");
    assert_eq!(name, "Button");
    let sexpr = inner.to_sexpr();
    assert!(sexpr.contains("(button"), "{sexpr}");
    assert!(sexpr.contains("\"Save\""), "{sexpr}");
}
