use rustweb_core::{Attr, VNode};
use rustweb_testing::{assert_json_snapshot, assert_snapshot};

fn app_tree() -> VNode {
    VNode::element(
        "main",
        vec![Attr::new("class", "app"), Attr::new("aria-label", "Todo app")],
        vec![
            VNode::element("h1", vec![], vec![VNode::text("Todos")]),
            VNode::element(
                "ul",
                vec![],
                vec![
                    {
                        let mut e = rustweb_core::Element::new("li", vec![VNode::text("buy milk")]);
                        e.key = Some("1".into());
                        e.attrs.push(Attr::new("aria-selected", "false"));
                        VNode::Element(e)
                    },
                    {
                        let mut e = rustweb_core::Element::new("li", vec![VNode::text("write docs")]);
                        e.key = Some("2".into());
                        VNode::Element(e)
                    },
                ],
            ),
            VNode::element(
                "button",
                vec![Attr::new("type", "button"), Attr::new("aria-pressed", "false")],
                vec![VNode::text("Add")],
            ),
        ],
    )
}

#[test]
fn app_sexpr_snapshot() {
    assert_snapshot(
        "app",
        &app_tree(),
        r#"(main class="app" aria-label="Todo app" (h1 "Todos") (ul (li #1 aria-selected="false" "buy milk") (li #2 "write docs")) (button type="button" aria-pressed="false" "Add"))"#,
    );
}

#[test]
fn empty_and_fragment_snapshots() {
    assert_snapshot("empty", &VNode::Empty, "()");
    assert_snapshot("frag", &VNode::fragment(vec![VNode::text("a"), VNode::text("b")]), r#"(<> "a" "b")"#);
}

#[test]
fn json_snapshot_shape() {
    let n = VNode::element("button", vec![Attr::new("disabled", true)], vec![VNode::text("Save")]);
    let json = n.to_snapshot_json();
    assert_json_snapshot(&n, &json);
    assert!(json.contains("\"disabled\""), "{json}");
}
