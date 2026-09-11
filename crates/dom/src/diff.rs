use crate::patch::{Patch, PatchPath};
use rustweb_core::{VNode};
use std::collections::{HashMap, HashSet};

pub fn diff(old: &VNode, new: &VNode) -> Vec<Patch> {
    let mut patches = Vec::new();
    diff_node(old, new, &mut Vec::new(), &mut patches);
    patches
}

fn child_list(n: &VNode) -> Option<&[VNode]> {
    match n {
        VNode::Element(el) => Some(&el.children),
        VNode::Fragment(c) => Some(c),
        VNode::Component(c) => Some(std::slice::from_ref(&c.rendered)),
        VNode::Text(_) | VNode::Empty => None,
    }
}

fn diff_node(old: &VNode, new: &VNode, path: &mut PatchPath, out: &mut Vec<Patch>) {

    match (old, new) {
        (VNode::Component(o), VNode::Component(n)) => {
            if o.name != n.name || o.props_json != n.props_json || o.key != n.key {
                out.push(Patch::Replace { path: path.clone(), new: new.clone() });
                return;
            }
            diff_node(&o.rendered, &n.rendered, path, out);
            return;
        }
        (VNode::Component(o), _) => {

            let mut tmp = Vec::new();
            let mut sub_path = path.clone();
            diff_node(&o.rendered, new, &mut sub_path, &mut tmp);

            out.extend(tmp);
            return;
        }
        (_, VNode::Component(n)) => {
            let mut tmp = Vec::new();
            let mut sub_path = path.clone();
            diff_node(old, &n.rendered, &mut sub_path, &mut tmp);
            out.extend(tmp);
            return;
        }
        _ => {}
    }

    match (old, new) {
        (VNode::Empty, VNode::Empty) => {}
        (VNode::Text(a), VNode::Text(b)) => {
            if a != b {
                out.push(Patch::SetText { path: path.clone(), text: b.clone() });
            }
        }
        (VNode::Element(a), VNode::Element(b)) => {
            if a.tag != b.tag || a.namespace != b.namespace {
                out.push(Patch::Replace { path: path.clone(), new: new.clone() });
                return;
            }
            diff_attrs(a, b, path, out);
            diff_listeners(a, b, path, out);

            diff_children(&a.children, &b.children, path, out);
        }
        (VNode::Fragment(a), VNode::Fragment(b)) => {
            diff_children(a, b, path, out);
        }

        _ => {
            if old != new {
                out.push(Patch::Replace { path: path.clone(), new: new.clone() });
            }
        }
    }
    let _ = child_list;
}

fn diff_attrs(
    old: &rustweb_core::Element,
    new: &rustweb_core::Element,
    path: &[usize],
    out: &mut Vec<Patch>,
) {
    let old_map: HashMap<&str, _> = old.attrs.iter().map(|a| (a.name.as_str(), &a.value)).collect();
    let new_map: HashMap<&str, _> = new.attrs.iter().map(|a| (a.name.as_str(), &a.value)).collect();
    for (name, val) in &new_map {
        match old_map.get(name) {
            Some(old_val) if *old_val == *val => {}
            _ => out.push(Patch::SetAttr { path: path.to_vec(), name: name.to_string(), value: (*val).clone() }),
        }
    }
    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            out.push(Patch::RemoveAttr { path: path.to_vec(), name: name.to_string() });
        }
    }
}

fn diff_listeners(
    old: &rustweb_core::Element,
    new: &rustweb_core::Element,
    path: &[usize],
    out: &mut Vec<Patch>,
) {
    let old_map: HashMap<&str, _> = old.listeners.iter().map(|(e, id)| (e.as_str(), id.as_str())).collect();
    let new_map: HashMap<&str, _> = new.listeners.iter().map(|(e, id)| (e.as_str(), id.as_str())).collect();
    for (ev, id) in &new_map {
        if old_map.get(ev) != Some(id) {
            out.push(Patch::SetListener { path: path.to_vec(), event: ev.to_string(), handler_id: id.to_string() });
        }
    }
    for ev in old_map.keys() {
        if !new_map.contains_key(ev) {
            out.push(Patch::RemoveListener { path: path.to_vec(), event: ev.to_string() });
        }
    }
}

fn diff_children(old: &[VNode], new: &[VNode], parent_path: &mut Vec<usize>, out: &mut Vec<Patch>) {

    if old.is_empty() && new.is_empty() {
        return;
    }

    let old_all_unkeyed = old.iter().all(|n| n.key().is_none());
    let new_all_unkeyed = new.iter().all(|n| n.key().is_none());
    if old_all_unkeyed && new_all_unkeyed {
        let common = old.len().min(new.len());
        for i in 0..common {
            parent_path.push(i);
            diff_node(&old[i], &new[i], parent_path, out);
            parent_path.pop();
        }

        for i in (new.len()..old.len()).rev() {
            let mut p = parent_path.clone();
            p.push(i);
            out.push(Patch::Remove { path: p });
        }
        for (i, node) in new.iter().enumerate().skip(old.len()) {
            out.push(Patch::Create { path: parent_path.clone(), index: i, node: node.clone() });
        }
        return;
    }

    let mut old_key_idx: HashMap<&str, usize> = HashMap::new();
    let mut dup_old_keys: HashSet<&str> = HashSet::new();
    for (i, n) in old.iter().enumerate() {
        if let Some(k) = n.key() {
            if old_key_idx.insert(k, i).is_some() {
                dup_old_keys.insert(k);
            }
        }
    }
    let mut new_keys: HashSet<&str> = HashSet::new();
    for n in new.iter() {
        if let Some(k) = n.key() {
            new_keys.insert(k);
        }
    }

    let mut removals: Vec<usize> = Vec::new();
    for (i, n) in old.iter().enumerate() {
        if let Some(k) = n.key() {
            if dup_old_keys.contains(k) {
                continue;
            }
            if !new_keys.contains(k) {
                removals.push(i);
            }
        }
    }
    removals.sort_unstable_by(|a, b| b.cmp(a));
    for i in removals {
        let mut p = parent_path.clone();
        p.push(i);
        out.push(Patch::Remove { path: p });
    }

    let mut old_pos_seq: Vec<usize> = Vec::new();
    let mut new_idx_for_seq: Vec<usize> = Vec::new();

    let mut old_unkeyed: Vec<usize> = old
        .iter()
        .enumerate()
        .filter(|(_, n)| n.key().is_none())
        .map(|(i, _)| i)
        .collect();
    let mut used_old = vec![false; old.len()];
    for &r in &{
        let mut kept: Vec<usize> = Vec::new();
        for (i, n) in old.iter().enumerate() {
            let remove = if let Some(k) = n.key() {
                !dup_old_keys.contains(k) && !new_keys.contains(k)
            } else {
                false
            };
            if !remove {
                kept.push(i);
            }
        }
        kept
    } {
        let _ = r;
    }

    for (new_i, new_node) in new.iter().enumerate() {
        if let Some(k) = new_node.key() {
            if dup_old_keys.contains(k) {

                parent_path.push(new_i);
                let old_node = old.get(new_i);
                match old_node {
                    Some(o) => diff_node(o, new_node, parent_path, out),
                    None => out.push(Patch::Create { path: parent_path.clone(), index: new_i, node: new_node.clone() }),
                }
                parent_path.pop();
                continue;
            }
            match old_key_idx.get(k) {
                Some(&old_i) => {
                    used_old[old_i] = true;
                    parent_path.push(new_i);
                    diff_node(&old[old_i], new_node, parent_path, out);
                    parent_path.pop();
                    old_pos_seq.push(old_i);
                    new_idx_for_seq.push(new_i);
                }
                None => {
                    out.push(Patch::Create { path: parent_path.clone(), index: new_i, node: new_node.clone() });
                }
            }
        } else {

            let slot = old_unkeyed.iter().find(|&&oi| !used_old[oi]).copied();
            match slot {
                Some(old_i) => {
                    used_old[old_i] = true;

                    old_unkeyed.retain(|&x| x != old_i);
                    parent_path.push(new_i);
                    diff_node(&old[old_i], new_node, parent_path, out);
                    parent_path.pop();
                }
                None => {
                    out.push(Patch::Create { path: parent_path.clone(), index: new_i, node: new_node.clone() });
                }
            }
        }
    }

    let mut leftover_unkeyed: Vec<usize> = Vec::new();
    for (i, n) in old.iter().enumerate() {
        if !used_old[i] && n.key().is_none() {
            leftover_unkeyed.push(i);
        }

        if !used_old[i] && n.key().is_some_and(|k| dup_old_keys.contains(k)) && i >= new.len() {
            leftover_unkeyed.push(i);
        }
    }
    leftover_unkeyed.sort_unstable_by(|a, b| b.cmp(a));
    for i in leftover_unkeyed {
        let mut p = parent_path.clone();
        p.push(i);
        out.push(Patch::Remove { path: p });
    }

    if old_pos_seq.len() > 1 {
        let lis = lis_indices(&old_pos_seq);
        let in_lis: HashSet<usize> = lis.into_iter().collect();
        for (seq_pos, &new_i) in new_idx_for_seq.iter().enumerate() {
            if in_lis.contains(&seq_pos) {
                continue;
            }
            let old_i = old_pos_seq[seq_pos];
            let key = new[new_i].key().unwrap_or_default().to_string();
            out.push(Patch::Move { path: parent_path.clone(), from: old_i, to: new_i, key });
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
    use rustweb_core::{Attr, VNode};

    fn el(tag: &str, key: Option<&str>, children: Vec<VNode>) -> VNode {
        let mut e = rustweb_core::Element::new(tag, children);
        e.key = key.map(|s| s.to_owned());
        VNode::Element(e)
    }

    #[test]
    fn text_update() {
        let p = diff(&VNode::text("a"), &VNode::text("b"));
        assert_eq!(p, vec![Patch::SetText { path: vec![], text: "b".into() }]);
    }

    #[test]
    fn tag_change_replaces() {
        let p = diff(&el("div", None, vec![]), &el("span", None, vec![]));
        assert!(matches!(p[0], Patch::Replace { .. }));
    }

    #[test]
    fn attr_diff() {
        let a = VNode::element("div", vec![Attr::new("class", "a")], vec![]);
        let b = VNode::element("div", vec![Attr::new("class", "b"), Attr::new("id", "x")], vec![]);
        let p = diff(&a, &b);
        assert!(p.iter().any(|x| matches!(x, Patch::SetAttr { name, .. } if name == "class")));
        assert!(p.iter().any(|x| matches!(x, Patch::SetAttr { name, .. } if name == "id")));
    }

    #[test]
    fn keyed_reorder_uses_move_not_replace() {
        let old = VNode::element(
            "ul",
            vec![],
            vec![el("li", Some("a"), vec![VNode::text("a")]), el("li", Some("b"), vec![VNode::text("b")]), el("li", Some("c"), vec![VNode::text("c")])],
        );
        let new = VNode::element(
            "ul",
            vec![],
            vec![el("li", Some("c"), vec![VNode::text("c")]), el("li", Some("a"), vec![VNode::text("a")]), el("li", Some("b"), vec![VNode::text("b")])],
        );
        let p = diff(&old, &new);
        assert!(p.iter().any(|x| x.is_move()), "expected a Move, got {p:?}");
        assert!(!p.iter().any(|x| matches!(x, Patch::Replace { .. })), "reorder must not replace: {p:?}");
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
        let new = VNode::element("div", vec![], vec![VNode::text("a"), VNode::text("c"), VNode::text("d")]);
        let p = diff(&old, &new);
        assert!(p.iter().any(|x| matches!(x, Patch::SetText { .. })));
        assert!(p.iter().any(|x| matches!(x, Patch::Create { .. })));
    }
}
