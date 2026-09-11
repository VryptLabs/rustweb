use rustweb_core::{Attr, VNode};
use rustweb_ssr::{checksum, render_to_string};

fn todo_app_tree() -> VNode {
    VNode::element(
        "main",
        vec![
            Attr::new("class", "app"),
            Attr::new("aria-label", "Todo app"),
        ],
        vec![
            VNode::element("h1", vec![], vec![VNode::text("Todos")]),
            VNode::element(
                "ul",
                vec![],
                vec![
                    {
                        let mut e = rustweb_core::Element::new("li", vec![VNode::text("buy milk")]);
                        e.key = Some("0".into());
                        VNode::Element(e)
                    },
                    {
                        let mut e =
                            rustweb_core::Element::new("li", vec![VNode::text("write docs")]);
                        e.key = Some("1".into());
                        e.attrs.push(Attr::new("aria-selected", "true"));
                        VNode::Element(e)
                    },
                    VNode::Empty,
                ],
            ),
            VNode::element(
                "button",
                vec![
                    Attr::new("type", "button"),
                    Attr::new("aria-pressed", "false"),
                ],
                vec![VNode::text("Add")],
            ),
        ],
    )
}

#[test]
fn ssr_markers_present() {
    let html = render_to_string(&todo_app_tree());
    assert!(
        html.contains("data-rwh=\"0\""),
        "root must have hydration id 0: {html}"
    );
    assert!(
        html.contains("data-rwh-checksum=\""),
        "root must have checksum: {html}"
    );
    assert!(
        html.contains("aria-label=\"Todo app\""),
        "ARIA passes through: {html}"
    );
    assert!(
        html.contains("aria-pressed=\"false\""),
        "aria-pressed passes through: {html}"
    );
    assert!(
        html.contains("aria-selected"),
        "aria-selected passes through: {html}"
    );
    assert!(
        html.contains("<!---->"),
        "empty nodes get comment placeholder: {html}"
    );
}

#[test]
fn ssr_ids_are_preorder() {
    let tree = VNode::element(
        "div",
        vec![],
        vec![
            VNode::element("span", vec![], vec![VNode::text("a")]),
            VNode::element("em", vec![], vec![VNode::text("b")]),
        ],
    );
    let html = render_to_string(&tree);
    assert!(html.contains("data-rwh=\"0\""), "root = 0");
    assert!(html.contains("data-rwh=\"1\""), "first child = 1");
    assert!(html.contains("data-rwh=\"2\""), "second child = 2");
}

#[test]
fn ssr_deterministic_across_renders() {
    let tree = todo_app_tree();
    let a = render_to_string(&tree);
    let b = render_to_string(&tree);
    assert_eq!(a, b, "SSR output must be deterministic");
}

#[test]
fn ssr_checksum_changes_on_content_change() {
    let tree_a = VNode::element("div", vec![], vec![VNode::text("hello")]);
    let tree_b = VNode::element("div", vec![], vec![VNode::text("world")]);
    let html_a = render_to_string(&tree_a);
    let html_b = render_to_string(&tree_b);
    let c_a = checksum(&html_a);
    let c_b = checksum(&html_b);
    assert_ne!(c_a, c_b, "checksum must differ for different content");
}

#[test]
fn ssr_escapes_html_entities() {
    let tree = VNode::element(
        "p",
        vec![Attr::new("title", "<script>&\"'")],
        vec![VNode::text("<b>bold</b>")],
    );
    let html = render_to_string(&tree);
    assert!(
        html.contains("&lt;b&gt;bold&lt;/b&gt;"),
        "text escaped: {html}"
    );
    assert!(
        html.contains("title=\"&lt;script&gt;&amp;&quot;&#39;\""),
        "attr escaped: {html}"
    );
}

#[test]
fn ssr_empty_and_fragment() {
    let html = render_to_string(&VNode::Empty);
    assert_eq!(html, "<!---->");

    let frag = VNode::fragment(vec![VNode::text("a"), VNode::text("b")]);
    let html = render_to_string(&frag);
    assert!(html.contains("a"), "fragment text present: {html}");
    assert!(html.contains("b"), "fragment text present: {html}");
    assert!(
        !html.contains("data-rwh"),
        "fragment has no hydration id: {html}"
    );
}

#[test]
fn ssr_void_elements_self_close() {
    let tree = VNode::element(
        "div",
        vec![],
        vec![VNode::element(
            "input",
            vec![Attr::new("type", "text"), Attr::new("aria-label", "Name")],
            vec![],
        )],
    );
    let html = render_to_string(&tree);
    assert!(html.contains("<input"), "input present: {html}");
    assert!(
        !html.contains("</input>"),
        "input must not have closing tag: {html}"
    );
}

#[test]
fn golden_format_stability() {
    let tree = VNode::element(
        "div",
        vec![Attr::new("id", "root")],
        vec![
            VNode::element("span", vec![], vec![VNode::text("hello")]),
            VNode::Empty,
        ],
    );
    let html = render_to_string(&tree);
    assert!(
        html.starts_with(r#"<div data-rwh="0" id="root""#),
        "golden: div must start with data-rwh and id: {html}"
    );
    assert!(
        html.contains(r#"<span data-rwh="1">hello</span>"#),
        "golden: span with id 1 must be present: {html}"
    );
    assert!(
        html.contains("<!---->"),
        "golden: empty must be rendered as comment: {html}"
    );
    assert!(
        html.ends_with("</div>"),
        "golden: must close the root div: {html}"
    );
    assert!(
        html.contains("data-rwh-checksum=\""),
        "golden: checksum must be injected in root tag: {html}"
    );
}

#[test]
fn golden_checksum_stability() {
    let tree = VNode::element("p", vec![], vec![VNode::text("test")]);
    let html = render_to_string(&tree);
    let cs = checksum(&html);
    assert!(!cs.is_empty(), "checksum must be non-empty");
    assert_eq!(cs.len(), 16, "checksum must be 16 hex chars: {cs}");
}
