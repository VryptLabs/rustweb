use crate::patch::{Patch, PatchPath};
use rustweb_core::VNode;
use std::collections::{HashMap, HashSet};

pub fn diff(old: &VNode, new: &VNode) -> Vec<Patch> {
    let mut patches = Vec::new();
    diff_node(old, new, &mut Vec::new(), &mut Vec::new(), &mut patches);
    patches
}

fn diff_node(
    old: &VNode,
    new: &VNode,
    old_path: &mut PatchPath,
    new_path: &mut PatchPath,
    out: &mut Vec<Patch>,
) {
    match (old, new) {
        (VNode::Component(o), VNode::Component(n)) => {
            if o.name != n.name || o.props_json != n.props_json || o.key != n.key {
                out.push(Patch::Replace {
                    path: new_path.clone(),
                    new: new.clone(),
                });
                return;
            }
            diff_node(&o.rendered, &n.rendered, old_path, new_path, out);
            return;
        }
        (VNode::Component(o), _) => {
            diff_node(&o.rendered, new, old_path, new_path, out);
            return;
        }
        (_, VNode::Component(n)) => {
            diff_node(old, &n.rendered, old_path, new_path, out);
            return;
        }
        _ => {}
    }

    match (old, new) {
        (VNode::Empty, VNode::Empty) => {}
        (VNode::Text(a), VNode::Text(b)) => {
            if a != b {
                out.push(Patch::SetText {
                    path: new_path.clone(),
                    text: b.clone(),
                });
            }
        }
        (VNode::Element(a), VNode::Element(b)) => {
            if a.tag != b.tag || a.namespace != b.namespace || a.key != b.key {
                out.push(Patch::Replace {
                    path: new_path.clone(),
                    new: new.clone(),
                });
                return;
            }
            diff_attrs(a, b, new_path, out);
            diff_listeners(a, b, new_path, out);
            diff_children(&a.children, &b.children, old_path, new_path, out);
        }
        (VNode::Fragment(a), VNode::Fragment(b)) => {
            diff_children(a, b, old_path, new_path, out);
        }
        _ => {
            if old != new {
                out.push(Patch::Replace {
                    path: new_path.clone(),
                    new: new.clone(),
                });
            }
        }
    }
}

fn diff_attrs(
    old: &rustweb_core::Element,
    new: &rustweb_core::Element,
    path: &[usize],
    out: &mut Vec<Patch>,
) {
    if old.attrs == new.attrs {
        return;
    }
    for a in &new.attrs {
        out.push(Patch::SetAttr {
            path: path.to_vec(),
            name: a.name.clone(),
            value: a.value.clone(),
        });
    }
    for a in &old.attrs {
        if !new.attrs.iter().any(|n| n.name == a.name) {
            out.push(Patch::RemoveAttr {
                path: path.to_vec(),
                name: a.name.clone(),
            });
        }
    }
}

fn diff_listeners(
    old: &rustweb_core::Element,
    new: &rustweb_core::Element,
    path: &[usize],
    out: &mut Vec<Patch>,
) {
    let old_map: HashMap<&str, _> = old
        .listeners
        .iter()
        .map(|(e, id)| (e.as_str(), id.as_str()))
        .collect();
    let new_map: HashMap<&str, _> = new
        .listeners
        .iter()
        .map(|(e, id)| (e.as_str(), id.as_str()))
        .collect();
    for (ev, id) in &new_map {
        if old_map.get(ev) != Some(id) {
            out.push(Patch::SetListener {
                path: path.to_vec(),
                event: ev.to_string(),
                handler_id: id.to_string(),
            });
        }
    }
    for ev in old_map.keys() {
        if !new_map.contains_key(ev) {
            out.push(Patch::RemoveListener {
                path: path.to_vec(),
                event: ev.to_string(),
            });
        }
    }
}

fn diff_children(
    old: &[VNode],
    new: &[VNode],
    old_parent: &mut PatchPath,
    new_parent: &mut PatchPath,
    out: &mut Vec<Patch>,
) {
    if old.is_empty() && new.is_empty() {
        return;
    }

    let old_all_unkeyed = old.iter().all(|n| n.key().is_none());
    let new_all_unkeyed = new.iter().all(|n| n.key().is_none());
    if old_all_unkeyed && new_all_unkeyed {
        let common = old.len().min(new.len());
        for i in 0..common {
            old_parent.push(i);
            new_parent.push(i);
            diff_node(&old[i], &new[i], old_parent, new_parent, out);
            old_parent.pop();
            new_parent.pop();
        }
        for i in (new.len()..old.len()).rev() {
            let mut p = old_parent.clone();
            p.push(i);
            out.push(Patch::Remove { path: p });
        }
        for (i, node) in new.iter().enumerate().skip(old.len()) {
            out.push(Patch::Create {
                path: new_parent.clone(),
                index: i,
                node: node.clone(),
            });
        }
        return;
    }

    let old_has_key = old.iter().any(|n| n.key().is_some());
    let new_has_key = new.iter().any(|n| n.key().is_some());
    let all_keyed = old.iter().all(|n| n.key().is_some()) && new.iter().all(|n| n.key().is_some());
    if (old_has_key || new_has_key) && !all_keyed {
        let common = old.len().min(new.len());
        for i in 0..common {
            old_parent.push(i);
            new_parent.push(i);
            diff_node(&old[i], &new[i], old_parent, new_parent, out);
            old_parent.pop();
            new_parent.pop();
        }
        for i in (new.len()..old.len()).rev() {
            let mut p = old_parent.clone();
            p.push(i);
            out.push(Patch::Remove { path: p });
        }
        for (i, node) in new.iter().enumerate().skip(old.len()) {
            out.push(Patch::Create {
                path: new_parent.clone(),
                index: i,
                node: node.clone(),
            });
        }
        return;
    }

    let mut old_key_first: HashMap<&str, usize> = HashMap::new();
    for (i, n) in old.iter().enumerate() {
        if let Some(k) = n.key() {
            old_key_first.entry(k).or_insert(i);
        }
    }

    let mut matched_old = vec![false; old.len()];
    let mut matched_new_keys: HashSet<&str> = HashSet::new();
    let mut keyed_pairs: Vec<(usize, usize)> = Vec::new();

    for (new_i, new_node) in new.iter().enumerate() {
        if let Some(k) = new_node.key() {
            let already = !matched_new_keys.insert(k);
            if let Some(&old_i) = old_key_first.get(k) {
                if !already && !matched_old[old_i] {
                    matched_old[old_i] = true;
                    keyed_pairs.push((old_i, new_i));
                    continue;
                }
            }
        }
        out.push(Patch::Create {
            path: new_parent.clone(),
            index: new_i,
            node: new_node.clone(),
        });
    }

    let mut removals: Vec<usize> = old
        .iter()
        .enumerate()
        .filter(|(i, _)| !matched_old[*i])
        .map(|(i, _)| i)
        .collect();
    removals.sort_unstable_by(|a, b| b.cmp(a));
    for i in removals {
        let mut p = old_parent.clone();
        p.push(i);
        out.push(Patch::Remove { path: p });
    }

    let has_unkeyed_new = new.iter().any(|n| n.key().is_none());
    let emit_all_moves = has_unkeyed_new || keyed_pairs.len() <= 1;

    if !emit_all_moves {
        let old_pos_seq: Vec<usize> = keyed_pairs.iter().map(|p| p.0).collect();
        let lis = lis_indices(&old_pos_seq);
        let in_lis: HashSet<usize> = lis.into_iter().collect();
        for (seq_pos, &(old_i, new_i)) in keyed_pairs.iter().enumerate() {
            old_parent.push(old_i);
            new_parent.push(new_i);
            diff_node(&old[old_i], &new[new_i], old_parent, new_parent, out);
            old_parent.pop();
            new_parent.pop();
            if !in_lis.contains(&seq_pos) {
                let key = new[new_i].key().unwrap_or_default().to_string();
                out.push(Patch::Move {
                    path: new_parent.clone(),
                    from: old_i,
                    to: new_i,
                    key,
                });
            }
        }
    } else {
        for &(old_i, new_i) in &keyed_pairs {
            old_parent.push(old_i);
            new_parent.push(new_i);
            diff_node(&old[old_i], &new[new_i], old_parent, new_parent, out);
            old_parent.pop();
            new_parent.pop();
            if old_i != new_i {
                let key = new[new_i].key().unwrap_or_default().to_string();
                out.push(Patch::Move {
                    path: new_parent.clone(),
                    from: old_i,
                    to: new_i,
                    key,
                });
            }
        }
    }
}

fn lis_indices(seq: &[usize]) -> Vec<usize> {
    let n = seq.len();
    if n == 0 {
        return vec![];
    }
    let mut tails: Vec<usize> = Vec::new();
    let mut prev: Vec<Option<usize>> = vec![None; n];
    for i in 0..n {
        let mut lo = 0usize;
        let mut hi = tails.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            if seq[tails[mid]] < seq[i] {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        if lo == tails.len() {
            tails.push(i);
        } else {
            tails[lo] = i;
        }
        if lo > 0 {
            prev[i] = Some(tails[lo - 1]);
        }
    }
    let mut out = Vec::with_capacity(tails.len());
    let mut cur = Some(*tails.last().unwrap());
    while let Some(c) = cur {
        out.push(c);
        cur = prev[c];
    }
    out.reverse();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustweb_core::Attr;

    fn el(tag: &str, key: Option<&str>, children: Vec<VNode>) -> VNode {
        let mut e = rustweb_core::Element::new(tag, children);
        e.key = key.map(|s| s.to_owned());
        VNode::Element(e)
    }

    #[test]
    fn text_update() {
        let p = diff(&VNode::text("a"), &VNode::text("b"));
        assert_eq!(
            p,
            vec![Patch::SetText {
                path: vec![],
                text: "b".into()
            }]
        );
    }

    #[test]
    fn tag_change_replaces() {
        let p = diff(&el("div", None, vec![]), &el("span", None, vec![]));
        assert!(matches!(p[0], Patch::Replace { .. }));
    }

    #[test]
    fn attr_diff() {
        let a = VNode::element("div", vec![Attr::new("class", "a")], vec![]);
        let b = VNode::element(
            "div",
            vec![Attr::new("class", "b"), Attr::new("id", "x")],
            vec![],
        );
        let p = diff(&a, &b);
        assert!(p
            .iter()
            .any(|x| matches!(x, Patch::SetAttr { name, .. } if name == "class")));
        assert!(p
            .iter()
            .any(|x| matches!(x, Patch::SetAttr { name, .. } if name == "id")));
    }

    #[test]
    fn keyed_reorder_uses_move_not_replace() {
        let old = VNode::element(
            "ul",
            vec![],
            vec![
                el("li", Some("a"), vec![VNode::text("a")]),
                el("li", Some("b"), vec![VNode::text("b")]),
                el("li", Some("c"), vec![VNode::text("c")]),
            ],
        );
        let new = VNode::element(
            "ul",
            vec![],
            vec![
                el("li", Some("c"), vec![VNode::text("c")]),
                el("li", Some("a"), vec![VNode::text("a")]),
                el("li", Some("b"), vec![VNode::text("b")]),
            ],
        );
        let p = diff(&old, &new);
        assert!(p.iter().any(|x| x.is_move()), "expected a Move, got {p:?}");
        assert!(
            !p.iter().any(|x| matches!(x, Patch::Replace { .. })),
            "reorder must not replace: {p:?}"
        );
    }

    #[test]
    fn keyed_insert_and_remove() {
        let old = VNode::element("ul", vec![], vec![el("li", Some("a"), vec![])]);
        let new = VNode::element("ul", vec![], vec![el("li", Some("b"), vec![])]);
        let p = diff(&old, &new);
        assert!(p.iter().any(|x| matches!(x, Patch::Remove { .. })));
        assert!(p.iter().any(|x| matches!(x, Patch::Create { .. })));
    }

    #[test]
    fn unkeyed_positional() {
        let old = VNode::element("div", vec![], vec![VNode::text("a"), VNode::text("b")]);
        let new = VNode::element(
            "div",
            vec![],
            vec![VNode::text("a"), VNode::text("c"), VNode::text("d")],
        );
        let p = diff(&old, &new);
        assert!(p.iter().any(|x| matches!(x, Patch::SetText { .. })));
        assert!(p.iter().any(|x| matches!(x, Patch::Create { .. })));
    }

    #[test]
    fn remove_path_uses_old_navigation() {
        let inner_old = el(
            "span",
            Some("x"),
            vec![el("i", Some("a"), vec![]), el("i", Some("b"), vec![])],
        );
        let inner_new = el("span", Some("x"), vec![el("i", Some("a"), vec![])]);
        let old = VNode::element(
            "main",
            vec![],
            vec![el("div", Some("z"), vec![]), inner_old],
        );
        let new = VNode::element("main", vec![], vec![inner_new]);
        let p = diff(&old, &new);
        let mut removes: Vec<Vec<usize>> = p
            .iter()
            .filter_map(|x| match x {
                Patch::Remove { path } => Some(path.clone()),
                _ => None,
            })
            .collect();
        removes.sort();
        assert_eq!(
            removes,
            vec![vec![0], vec![1, 1]],
            "remove paths must use old navigation, got: {p:?}"
        );
    }
}
