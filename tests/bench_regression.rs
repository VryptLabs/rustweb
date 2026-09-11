use rustweb_core::VNode;
use std::time::Instant;

fn big_list(n: usize) -> VNode {
    VNode::element(
        "ul",
        vec![],
        (0..n)
            .map(|i| {
                let mut e = rustweb_core::Element::new("li", vec![VNode::text(format!("row {i}"))]);
                e.key = Some(i.to_string());
                VNode::Element(e)
            })
            .collect(),
    )
}

fn mult() -> f64 {
    std::env::var("BENCH_MULT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1.0)
}

#[test]
fn bench_diff_1000() {
    let old = big_list(1000);
    let mut new_children = match &old {
        VNode::Element(el) => el.children.clone(),
        _ => unreachable!(),
    };
    new_children.rotate_right(1);
    let new = VNode::element("ul", vec![], new_children);
    let t0 = Instant::now();
    let patches = rustweb_dom::diff(&old, &new);
    let dt = t0.elapsed();
    assert!(!patches.is_empty());
    let budget = (50.0 * mult()) as u128;
    assert!(
        dt.as_millis() <= budget,
        "diff regression: {:?} > {budget}ms",
        dt
    );
    eprintln!("diff 1000 rows: {dt:?} ({} patches)", patches.len());
}

#[test]
fn bench_patch_apply_1000() {
    let old = big_list(1000);
    let mut new_children = match &old {
        VNode::Element(el) => el.children.clone(),
        _ => unreachable!(),
    };
    new_children.rotate_right(7);
    let new = VNode::element("ul", vec![], new_children);
    let patches = rustweb_dom::diff(&old, &new);
    let t0 = Instant::now();
    let out = rustweb_dom::renderer::apply_patches_to_tree(old, &patches).unwrap();
    let dt = t0.elapsed();
    assert_eq!(out, new);
    let budget = (25.0 * mult()) as u128;
    assert!(
        dt.as_millis() <= budget,
        "patch regression: {:?} > {budget}ms",
        dt
    );
}

#[test]
fn bench_ssr_1000() {
    let tree = big_list(1000);
    let t0 = Instant::now();
    let html = rustweb_ssr::render_to_string(&tree);
    let dt = t0.elapsed();
    assert!(html.len() > 10_000);
    let budget = (50.0 * mult()) as u128;
    assert!(
        dt.as_millis() <= budget,
        "ssr regression: {:?} > {budget}ms",
        dt
    );
}
