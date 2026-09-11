use rustweb_core::{Attr, VNode};
use rustweb_dom::{Renderer, diff};
use rustweb_ssr::{render_to_string, verify_hydration};

fn list(items: &[&str]) -> VNode {
    VNode::element(
        "ul",
        vec![Attr::new("aria-label", "items")],
        items
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let mut e = rustweb_core::Element::new("li", vec![VNode::text(*t)]);
                e.key = Some(i.to_string());
                VNode::Element(e)
            })
            .collect(),
    )
}

#[test]
fn ssr_then_hydrate_ok() {
    let tree = list(&["a", "b", "c"]);
    let html = render_to_string(&tree);
    verify_hydration(&html, &tree).expect("identical tree must hydrate");
}

#[test]
fn ssr_then_client_update_patches_cleanly() {
    let old = list(&["a", "b", "c"]);
    let new = list(&["c", "a", "b", "d"]);
    let html = render_to_string(&old);
    verify_hydration(&html, &old).unwrap();

    let patches = diff(&old, &new);
    let committed = rustweb_dom::renderer::apply_patches_to_tree(old, &patches).unwrap();
    assert_eq!(committed, new);

    assert_eq!(render_to_string(&committed), render_to_string(&new));
}

#[test]
fn hydration_mismatch_is_structured() {
    let server = VNode::element("div", vec![], vec![VNode::text("server")]);
    let client = VNode::element("div", vec![], vec![VNode::text("client")]);
    let html = render_to_string(&server);
    let err = verify_hydration(&html, &client).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("byte") || msg.contains("mismatch"), "{msg}");
}

#[test]
fn renderer_commits_without_mismatch() {
    let mut r = Renderer::new(list(&["x"]));
    let n = r.update("Test", list(&["x", "y"])).unwrap();
    assert!(n > 0);
    assert_eq!(r.current().child_count(), 2);
}
