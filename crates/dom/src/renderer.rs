use crate::patch::Patch;
use rustweb_core::{ComponentError, VNode};
use std::rc::Rc;

pub fn apply_patches_to_tree(mut root: VNode, patches: &[Patch]) -> Result<VNode, ComponentError> {
    let mut removes: Vec<&Patch> = patches
        .iter()
        .filter(|p| matches!(p, Patch::Remove { .. }))
        .collect();
    removes.sort_by(|a, b| b.path().cmp(a.path()));
    for p in removes {
        if let Patch::Remove { path } = p {
            remove_at(&mut root, path)?;
        }
    }

    for p in patches {
        match p {
            Patch::Remove { .. } => {}
            Patch::Replace { path, new } => replace_at(&mut root, path, new.clone())?,
            Patch::Create { path, index, node } => {
                insert_at(&mut root, path, *index, node.clone())?
            }
            Patch::SetText { path, text } => set_text_at(&mut root, path, text)?,
            Patch::SetAttr { path, name, value } => {
                set_attr_at(&mut root, path, name, value.clone())?
            }
            Patch::RemoveAttr { path, name } => remove_attr_at(&mut root, path, name)?,
            Patch::Move { path, to, key, .. } => move_keyed_at(&mut root, path, key, *to)?,
            Patch::SetListener {
                path,
                event,
                handler_id,
            } => set_listener_at(&mut root, path, event, handler_id)?,
            Patch::RemoveListener { path, event } => remove_listener_at(&mut root, path, event)?,
        }
    }
    Ok(root)
}

fn err(component: &str, msg: String) -> ComponentError {
    ComponentError::Lifecycle {
        component: component.into(),
        hook: "patch".into(),
        message: msg,
    }
}

#[allow(dead_code)]
fn children_mut(node: &mut VNode) -> Option<&mut Vec<VNode>> {
    let mut cur = node;
    loop {
        match cur {
            VNode::Element(el) => return Some(&mut el.children),
            VNode::Fragment(c) => return Some(c),
            VNode::Component(c) => cur = &mut c.rendered,
            VNode::Text(_) | VNode::Empty => return None,
        }
    }
}

fn resolve_mut(mut cur: &mut VNode) -> &mut VNode {
    loop {
        match cur {
            VNode::Component(c) => cur = &mut c.rendered,
            _ => return cur,
        }
    }
}

fn get_mut<'a>(root: &'a mut VNode, path: &[usize]) -> Result<&'a mut VNode, ComponentError> {
    let mut cur = root;
    for &i in path {
        cur = resolve_mut(cur);
        let kids = match cur {
            VNode::Element(el) => &mut el.children,
            VNode::Fragment(c) => c,
            VNode::Component(_) => unreachable!("resolve_mut strips components"),
            VNode::Text(_) | VNode::Empty => {
                return Err(err("Renderer", format!("path {path:?} descends into leaf")));
            }
        };
        cur = kids.get_mut(i).ok_or_else(|| {
            err(
                "Renderer",
                format!("path index {i} out of bounds at {path:?}"),
            )
        })?;
    }
    Ok(cur)
}

fn parent_mut<'a>(root: &'a mut VNode, path: &[usize]) -> Result<&'a mut VNode, ComponentError> {
    if path.is_empty() {
        return Err(err("Renderer", "parent of root requested".into()));
    }
    get_mut(root, &path[..path.len() - 1])
}

fn remove_at(root: &mut VNode, path: &[usize]) -> Result<(), ComponentError> {
    if path.is_empty() {
        *root = VNode::Empty;
        return Ok(());
    }
    let parent = parent_mut(root, path)?;

    let kids = match parent {
        VNode::Component(c) => match &mut *c.rendered {
            VNode::Element(el) => &mut el.children,
            VNode::Fragment(f) => f,
            other => {
                if path[path.len() - 1] == 0 {
                    *other = VNode::Empty;
                    return Ok(());
                }
                return Err(err(
                    "Renderer",
                    format!("remove: bad index for component inner at {path:?}"),
                ));
            }
        },
        VNode::Element(el) => &mut el.children,
        VNode::Fragment(f) => f,
        VNode::Text(_) | VNode::Empty => {
            return Err(err(
                "Renderer",
                format!("remove: parent is leaf at {path:?}"),
            ))
        }
    };
    let idx = path[path.len() - 1];
    if idx >= kids.len() {
        return Err(err(
            "Renderer",
            format!("remove: index {idx} OOB (len {})", kids.len()),
        ));
    }
    kids.remove(idx);
    Ok(())
}

fn replace_at(root: &mut VNode, path: &[usize], new: VNode) -> Result<(), ComponentError> {
    if path.is_empty() {
        *root = new;
        return Ok(());
    }
    let slot = get_mut(root, path)?;
    *slot = new;
    Ok(())
}

fn insert_at(
    root: &mut VNode,
    parent_path: &[usize],
    index: usize,
    node: VNode,
) -> Result<(), ComponentError> {
    let parent = if parent_path.is_empty() {
        root
    } else {
        get_mut(root, parent_path)?
    };

    let kids = match parent {
        VNode::Element(el) => &mut el.children,
        VNode::Fragment(f) => f,
        VNode::Component(c) => match &mut *c.rendered {
            VNode::Element(el) => &mut el.children,
            VNode::Fragment(f) => f,
            inner => {
                if index == 0 {
                    *inner = VNode::Fragment(vec![node]);
                } else {
                    return Err(err("Renderer", "insert into component scalar inner".into()));
                }
                return Ok(());
            }
        },
        VNode::Text(_) | VNode::Empty => {
            let old = std::mem::replace(parent, VNode::Fragment(vec![]));
            if let VNode::Fragment(f) = parent {
                if !matches!(old, VNode::Empty) {
                    f.push(old);
                }
                if index <= f.len() {
                    f.insert(index.min(f.len()), node);
                } else {
                    f.push(node);
                }
                return Ok(());
            }
            unreachable!()
        }
    };
    let idx = index.min(kids.len());
    kids.insert(idx, node);
    Ok(())
}

fn set_text_at(root: &mut VNode, path: &[usize], text: &str) -> Result<(), ComponentError> {
    let slot = get_mut(root, path)?;

    let target = match slot {
        VNode::Component(c) => &mut *c.rendered,
        other => other,
    };
    match target {
        VNode::Text(t) => {
            *t = text.to_owned();
            Ok(())
        }
        _ => Err(err(
            "Renderer",
            format!("SetText targeted non-text at {path:?}"),
        )),
    }
}

fn set_attr_at(
    root: &mut VNode,
    path: &[usize],
    name: &str,
    value: rustweb_core::AttrValue,
) -> Result<(), ComponentError> {
    let slot = get_mut(root, path)?;
    let el = match slot {
        VNode::Element(el) => el,
        VNode::Component(c) => match &mut *c.rendered {
            VNode::Element(el) => el,
            _ => {
                return Err(err(
                    "Renderer",
                    format!("SetAttr targeted non-element at {path:?}"),
                ))
            }
        },
        _ => {
            return Err(err(
                "Renderer",
                format!("SetAttr targeted non-element at {path:?}"),
            ))
        }
    };
    match el.attrs.iter_mut().find(|a| a.name == name) {
        Some(a) => a.value = value,
        None => el.attrs.push(rustweb_core::Attr {
            name: name.to_owned(),
            value,
        }),
    }
    Ok(())
}

fn remove_attr_at(root: &mut VNode, path: &[usize], name: &str) -> Result<(), ComponentError> {
    let slot = get_mut(root, path)?;
    let el = match slot {
        VNode::Element(el) => el,
        VNode::Component(c) => match &mut *c.rendered {
            VNode::Element(el) => el,
            _ => return Ok(()),
        },
        _ => return Ok(()),
    };
    el.attrs.retain(|a| a.name != name);
    Ok(())
}

fn set_listener_at(
    root: &mut VNode,
    path: &[usize],
    event: &str,
    id: &str,
) -> Result<(), ComponentError> {
    let slot = get_mut(root, path)?;
    let el = match slot {
        VNode::Element(el) => el,
        VNode::Component(c) => match &mut *c.rendered {
            VNode::Element(el) => el,
            _ => return Ok(()),
        },
        _ => return Ok(()),
    };
    match el.listeners.iter_mut().find(|(e, _)| e == event) {
        Some(slot) => slot.1 = id.to_owned(),
        None => el.listeners.push((event.to_owned(), id.to_owned())),
    }
    Ok(())
}

fn remove_listener_at(root: &mut VNode, path: &[usize], event: &str) -> Result<(), ComponentError> {
    let slot = get_mut(root, path)?;
    let el = match slot {
        VNode::Element(el) => el,
        VNode::Component(c) => match &mut *c.rendered {
            VNode::Element(el) => el,
            _ => return Ok(()),
        },
        _ => return Ok(()),
    };
    el.listeners.retain(|(e, _)| e != event);
    Ok(())
}

fn move_keyed_at(
    root: &mut VNode,
    parent_path: &[usize],
    key: &str,
    to: usize,
) -> Result<(), ComponentError> {
    let parent = if parent_path.is_empty() {
        &mut *root
    } else {
        get_mut(root, parent_path)?
    };
    let kids = match parent {
        VNode::Element(el) => &mut el.children,
        VNode::Fragment(f) => f,
        VNode::Component(c) => match &mut *c.rendered {
            VNode::Element(el) => &mut el.children,
            VNode::Fragment(f) => f,
            _ => return Err(err("Renderer", "Move inside scalar component".into())),
        },
        _ => return Err(err("Renderer", "Move inside leaf".into())),
    };
    let from = kids
        .iter()
        .position(|n| n.key() == Some(key))
        .ok_or_else(|| err("Renderer", format!("Move: key `{key}` not found")))?;
    if from == to.min(kids.len().saturating_sub(1)) {
        return Ok(());
    }
    let node = kids.remove(from);
    let idx = to.min(kids.len());
    kids.insert(idx, node);
    Ok(())
}

fn guard<T>(
    component: &str,
    stage: &str,
    f: impl FnOnce() -> Result<T, ComponentError>,
) -> Result<T, ComponentError> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(r) => r,
        Err(payload) => {
            let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "<non-string panic>".to_owned()
            };
            Err(ComponentError::Panicked {
                component: component.into(),
                stage: stage.into(),
                location: "renderer".into(),
                payload: msg,
            })
        }
    }
}

pub struct Renderer {
    current: VNode,
    _delegator_note: (),
}

impl Renderer {
    pub fn new(initial: VNode) -> Self {
        Self {
            current: initial,
            _delegator_note: (),
        }
    }

    pub fn current(&self) -> &VNode {
        &self.current
    }

    pub fn update(
        &mut self,
        component: &'static str,
        next: VNode,
    ) -> Result<usize, ComponentError> {
        let old = self.current.clone();
        let patches = guard(component, "diff", || {
            Ok::<_, ComponentError>(crate::diff::diff(&old, &next))
        })?;
        let n = patches.len();
        let committed = guard(component, "patch", || apply_patches_to_tree(old, &patches))?;

        debug_assert_eq!(committed, next, "patch applier diverged from diff target");
        self.current = next;
        let _ = (component, Rc::new(()));
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustweb_core::VNode;

    fn li(key: &str, t: &str) -> VNode {
        let mut e = rustweb_core::Element::new("li", vec![VNode::text(t)]);
        e.key = Some(key.to_owned());
        VNode::Element(e)
    }

    #[test]
    fn apply_roundtrip_reorder() {
        let old = VNode::element("ul", vec![], vec![li("a", "a"), li("b", "b"), li("c", "c")]);
        let new = VNode::element("ul", vec![], vec![li("c", "c"), li("a", "a"), li("b", "b")]);
        let patches = crate::diff::diff(&old, &new);
        let out = apply_patches_to_tree(old, &patches).unwrap();
        assert_eq!(out, new);
    }

    #[test]
    fn applier_never_panics_on_bad_path() {
        let root = VNode::element("div", vec![], vec![]);
        let bad = vec![Patch::Remove {
            path: vec![9, 9, 9],
        }];
        assert!(apply_patches_to_tree(root, &bad).is_err());
    }
}
