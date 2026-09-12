use crate::diff;
use crate::renderer::apply_patches_to_tree;
use rustweb_core::{Attr, VNode};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn now_ms() -> f64 {
    js_sys::Date::now()
}

fn keyed_list(n: usize) -> VNode {
    let children = (0..n)
        .map(|i| {
            let mut e = rustweb_core::Element::new("li", vec![VNode::text(format!("row {i}"))]);
            e.key = Some(i.to_string());
            e.attrs.push(Attr::new("class", "row"));
            VNode::Element(e)
        })
        .collect();
    VNode::element("ul", vec![Attr::new("aria-label", "rows")], children)
}

fn rotated(list: &VNode, by: usize) -> VNode {
    match list {
        VNode::Element(el) => {
            let mut kids = el.children.clone();
            kids.rotate_right(by);
            VNode::element(el.tag.clone(), el.attrs.clone(), kids)
        }
        other => other.clone(),
    }
}

#[wasm_bindgen_test]
fn browser_diff_1000_under_budget() {
    let old = keyed_list(1000);
    let new = rotated(&old, 500);
    let start = now_ms();
    let mut total = 0usize;
    for _ in 0..20 {
        total += diff(&old, &new).len();
    }
    let per = (now_ms() - start) / 20.0;
    web_sys::console::log_1(&format!("diff/1000 = {per:.3} ms ({total} patches)").into());
    assert!(per < 50.0, "browser diff regression: {per:.3} ms > 50 ms");
}

#[wasm_bindgen_test]
fn browser_apply_1000_under_budget() {
    let old = keyed_list(1000);
    let new = rotated(&old, 500);
    let patches = diff(&old, &new);
    let start = now_ms();
    let mut ok = 0usize;
    for _ in 0..20 {
        if apply_patches_to_tree(old.clone(), &patches).is_ok() {
            ok += 1;
        }
    }
    let per = (now_ms() - start) / 20.0;
    web_sys::console::log_1(&format!("apply/1000 = {per:.3} ms ({ok}/20 ok)").into());
    assert_eq!(ok, 20);
    assert!(per < 50.0, "browser apply regression: {per:.3} ms > 50 ms");
}

#[wasm_bindgen_test]
fn browser_ssr_1000_under_budget() {
    let tree = keyed_list(1000);
    let start = now_ms();
    let mut len = 0usize;
    for _ in 0..20 {
        len = rustweb_ssr::render_to_string(&tree).len();
    }
    let per = (now_ms() - start) / 20.0;
    web_sys::console::log_1(&format!("ssr/1000 = {per:.3} ms ({len} bytes)").into());
    assert!(len > 10_000);
    assert!(per < 80.0, "browser ssr regression: {per:.3} ms > 80 ms");
}
