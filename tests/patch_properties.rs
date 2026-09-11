use rustweb_core::VNode;
use rustweb_dom::{diff, renderer::apply_patches_to_tree};

fn li(key: &str, text: &str) -> VNode {
    let mut e = rustweb_core::Element::new("li", vec![VNode::text(text)]);
    e.key = Some(key.to_owned());
    VNode::Element(e)
}

fn ul(keys: &[&str]) -> VNode {
    VNode::element("ul", vec![], keys.iter().map(|k| li(k, k)).collect())
}

#[test]
fn reverse_is_exact() {
    let old = ul(&["a", "b", "c", "d", "e"]);
    let new = ul(&["e", "d", "c", "b", "a"]);
    let p = diff(&old, &new);
    assert_eq!(apply_patches_to_tree(old, &p).unwrap(), new);
}

#[test]
fn interleave_insert_remove_move() {
    let cases: Vec<(Vec<&str>, Vec<&str>)> = vec![
        (vec![], vec!["a"]),
        (vec!["a"], vec![]),
        (vec!["a", "b"], vec!["b", "a"]),
        (vec!["a", "b", "c"], vec!["c"]),
        (vec!["a"], vec!["b", "a", "c"]),
        (vec!["a", "b", "c", "d"], vec!["d", "b", "a"]),
    ];
    for (o, n) in cases {
        let old = ul(&o);
        let new = ul(&n);
        let p = diff(&old, &new);
        assert_eq!(
            apply_patches_to_tree(old, &p).unwrap(),
            new,
            "case {o:?} -> {n:?}"
        );
    }
}

#[test]
fn text_and_attr_edits_compose() {
    use rustweb_core::Attr;
    let old = VNode::element("div", vec![Attr::new("class", "a")], vec![VNode::text("x")]);
    let new = VNode::element(
        "div",
        vec![Attr::new("class", "b"), Attr::new("id", "r")],
        vec![VNode::text("y")],
    );
    let p = diff(&old, &new);
    assert_eq!(apply_patches_to_tree(old, &p).unwrap(), new);
}
