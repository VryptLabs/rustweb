use rustweb_core::VNode;
use std::collections::HashSet;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next() % n as u64) as usize
    }
    fn one_in(&mut self, n: usize) -> bool {
        self.below(n) == 0
    }
    fn pick<'a, T>(&mut self, s: &'a [T]) -> &'a T {
        &s[self.below(s.len())]
    }
}

const TAGS: &[&str] = &["div", "span", "ul", "li", "button", "p", "section"];
const WORDS: &[&str] = &["a", "todo", "buy milk", "<x>&\"'", "", "lorem ipsum", "42"];
const KEYS: &[&str] = &["a", "b", "c", "d", "e", "f"];
const ATTRS: &[&str] = &["class", "id", "aria-label", "title"];

fn gen_attr(rng: &mut Rng) -> rustweb_core::Attr {
    let name = rng.pick(ATTRS).to_string();
    match rng.below(3) {
        0 => rustweb_core::Attr::new(name, rng.pick(WORDS).to_string()),
        1 => rustweb_core::Attr::new(name, rng.below(2) == 0),
        _ => rustweb_core::Attr::new(name, rng.below(5) as i64 - 2),
    }
}

fn gen_children(rng: &mut Rng, depth: usize, keyed: bool) -> Vec<VNode> {
    let n = rng.below(5);
    (0..n).map(|_| gen_node_inner(rng, depth, keyed)).collect()
}

fn gen_element_inner(rng: &mut Rng, depth: usize, keyed: bool) -> VNode {
    let mut el =
        rustweb_core::Element::new(rng.pick(TAGS).to_string(), gen_children(rng, depth, keyed));
    if keyed {
        el.key = Some(rng.pick(KEYS).to_string());
    }
    let mut seen = HashSet::new();
    for _ in 0..rng.below(3) {
        let a = gen_attr(rng);
        if seen.insert(a.name.clone()) {
            el.attrs.push(a);
        }
    }
    VNode::Element(el)
}

fn gen_node_inner(rng: &mut Rng, depth: usize, keyed: bool) -> VNode {
    if depth == 0 {
        return if rng.one_in(2) {
            VNode::text(rng.pick(WORDS).to_string())
        } else {
            VNode::Empty
        };
    }
    match rng.below(10) {
        0..=4 => gen_element_inner(rng, depth - 1, keyed),
        5..=7 => VNode::text(rng.pick(WORDS).to_string()),
        8 => VNode::fragment(gen_children(rng, depth - 1, keyed)),
        _ => VNode::Empty,
    }
}

fn mutate_children(rng: &mut Rng, kids: &mut Vec<VNode>, keyed: bool) {
    if kids.is_empty() {
        if rng.one_in(2) {
            kids.push(gen_node_inner(rng, 2, keyed));
        }
        return;
    }
    match rng.below(6) {
        0 => {
            for i in (1..kids.len()).rev() {
                let j = rng.below(i + 1);
                kids.swap(i, j);
            }
        }
        1 => {
            kids.remove(rng.below(kids.len()));
        }
        2 => {
            kids.insert(rng.below(kids.len() + 1), gen_node_inner(rng, 2, keyed));
        }
        3 => {
            let i = rng.below(kids.len());
            mutate_inner(rng, &mut kids[i], keyed);
        }
        4 => {
            let i = rng.below(kids.len());
            kids[i] = gen_node_inner(rng, 2, keyed);
        }
        _ => {
            for k in kids.iter_mut() {
                if rng.one_in(3) {
                    mutate_inner(rng, k, keyed);
                }
            }
        }
    }
}

fn mutate_inner(rng: &mut Rng, node: &mut VNode, keyed: bool) {
    match node {
        VNode::Text(t) => {
            if rng.one_in(2) {
                *t = rng.pick(WORDS).to_string();
            }
        }
        VNode::Empty => {
            if rng.one_in(2) {
                *node = gen_node_inner(rng, 1, keyed);
            }
        }
        VNode::Fragment(c) => mutate_children(rng, c, keyed),
        VNode::Element(el) => match rng.below(5) {
            0 => el.tag = rng.pick(TAGS).to_string(),
            1 | 4 => mutate_children(rng, &mut el.children, keyed),
            2 => {
                if rng.one_in(2) || el.attrs.is_empty() {
                    let a = gen_attr(rng);
                    match el.attrs.iter_mut().find(|x| x.name == a.name) {
                        Some(slot) => slot.value = a.value,
                        None => el.attrs.push(a),
                    }
                } else {
                    el.attrs.remove(rng.below(el.attrs.len()));
                }
            }
            3 => {
                if keyed {
                    el.key = if rng.one_in(3) {
                        None
                    } else {
                        Some(rng.pick(KEYS).to_string())
                    };
                }
            }
            _ => *node = gen_node_inner(rng, 2, keyed),
        },
        VNode::Component(c) => mutate_inner(rng, &mut c.rendered, keyed),
    }
}

fn gen_tree(rng: &mut Rng) -> VNode {
    let keyed = rng.one_in(2);
    gen_node_inner(rng, 2, keyed)
}

fn mutate_tree(rng: &mut Rng, node: &mut VNode) {
    let keyed = match node {
        VNode::Element(el) => el.key.is_some() || el.children.iter().any(|c| c.key().is_some()),
        VNode::Fragment(c) => c.iter().any(|n| n.key().is_some()),
        _ => false,
    };
    mutate_inner(rng, node, keyed);
}

#[test]
fn fuzz_diff_apply_roundtrip() {
    let iters: usize = std::env::var("FUZZ_ITERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);
    let seed: u64 = std::env::var("FUZZ_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x9E3779B97F4A7C15);
    let mut rng = Rng(seed);
    let mut ssr_matches = 0u64;
    let mut apply_ok = 0u64;
    let mut first_failure: Option<(usize, String)> = None;
    for i in 0..iters {
        let old = gen_tree(&mut rng);
        let mut new = old.clone();
        mutate_tree(&mut rng, &mut new);
        let patches = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rustweb_dom::diff(&old, &new)
        }));
        let patches = match patches {
            Ok(p) => p,
            Err(_) => {
                first_failure.get_or_insert((i, "diff panicked".into()));
                continue;
            }
        };
        let out = match rustweb_dom::renderer::apply_patches_to_tree(old.clone(), &patches) {
            Ok(t) => t,
            Err(e) => {
                first_failure.get_or_insert((i, format!("apply error: {e}")));
                continue;
            }
        };
        apply_ok += 1;
        let new_ssr = rustweb_ssr::render_to_string(&new);
        let out_ssr = rustweb_ssr::render_to_string(&out);
        if out_ssr == new_ssr {
            ssr_matches += 1;
        } else {
            first_failure
                .get_or_insert((i, format!("ssr mismatch\nout: {out_ssr}\nnew: {new_ssr}")));
        }
    }
    let apply_rate = apply_ok as f64 / iters as f64;
    let ssr_rate = ssr_matches as f64 / iters as f64;
    eprintln!(
        "fuzz: {iters} iters, apply_ok={apply_ok} ({:.1}%), ssr_match={ssr_matches} ({:.1}%)",
        apply_rate * 100.0,
        ssr_rate * 100.0
    );
    assert!(
        apply_rate >= 0.95,
        "apply success rate {apply_rate:.3} below 95% — applier regression (first failure: {first_failure:?})"
    );
    assert!(
        ssr_rate >= 0.95,
        "SSR match rate {ssr_rate:.3} below 95% threshold — diff/applier regression (first failure: {first_failure:?})"
    );
}
