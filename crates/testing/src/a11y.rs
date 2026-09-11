use rustweb_core::{AttrValue, Element, VNode};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Serious,
    Moderate,
    Minor,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Severity::Serious => "serious",
            Severity::Moderate => "moderate",
            Severity::Minor => "minor",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A11yViolation {
    pub path: String,
    pub rule: &'static str,
    pub severity: Severity,
    pub message: String,
    pub fix_hint: String,
}

impl fmt::Display for A11yViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} at `{}`: {} (fix: {})",
            self.severity, self.rule, self.path, self.message, self.fix_hint
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct A11yReport {
    pub violations: Vec<A11yViolation>,
}

impl A11yReport {
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }

    pub fn has_blocking(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity != Severity::Minor)
    }

    pub fn of_severity(&self, severity: Severity) -> Vec<&A11yViolation> {
        self.violations
            .iter()
            .filter(|v| v.severity == severity)
            .collect()
    }

    pub fn to_text(&self) -> String {
        if self.violations.is_empty() {
            return "0 a11y violations".to_owned();
        }
        self.violations
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn non_empty_attr(el: &Element, name: &str) -> bool {
    el.attrs
        .iter()
        .any(|a| a.name == name && matches!(&a.value, AttrValue::String(s) if !s.trim().is_empty()))
}

fn has_attr(el: &Element, name: &str) -> bool {
    el.attrs.iter().any(|a| a.name == name)
}

fn role(el: &Element) -> Option<&str> {
    el.attrs.iter().find_map(|a| {
        if a.name == "role" {
            if let AttrValue::String(s) = &a.value {
                return Some(s.as_str());
            }
        }
        None
    })
}

fn has_accessible_name(el: &Element) -> bool {
    non_empty_attr(el, "aria-label")
        || non_empty_attr(el, "aria-labelledby")
        || non_empty_attr(el, "title")
        || subtree_has_text(el)
}

fn subtree_has_text(el: &Element) -> bool {
    el.children.iter().any(node_has_text)
}

fn node_has_text(node: &VNode) -> bool {
    match node {
        VNode::Text(t) => !t.trim().is_empty(),
        VNode::Element(e) => {
            if !e.children.is_empty() && subtree_has_text(e) {
                return true;
            }
            matches!(e.tag.as_str(), "img" | "input")
                && (non_empty_attr(e, "alt") || non_empty_attr(e, "aria-label"))
        }
        VNode::Fragment(c) => c.iter().any(node_has_text),
        VNode::Component(c) => node_has_text(&c.rendered),
        VNode::Empty => false,
    }
}

fn is_interactive(el: &Element) -> bool {
    let tag = el.tag.as_str();
    if matches!(tag, "button" | "summary") {
        return true;
    }
    if tag == "a" && has_attr(el, "href") {
        return true;
    }
    if tag == "details" {
        return false;
    }
    matches!(
        role(el),
        Some(
            "button"
                | "link"
                | "textbox"
                | "searchbox"
                | "checkbox"
                | "radio"
                | "switch"
                | "tab"
                | "menuitem"
                | "option"
                | "combobox"
                | "slider"
                | "menuitemcheckbox"
                | "menuitemradio"
        )
    )
}

fn is_form_control(el: &Element) -> bool {
    let tag = el.tag.as_str();
    if matches!(tag, "textarea" | "select") {
        return true;
    }
    if tag == "input" {
        if let Some(a) = el.attrs.iter().find(|a| a.name == "type") {
            if let AttrValue::String(t) = &a.value {
                return !matches!(
                    t.as_str(),
                    "hidden" | "button" | "submit" | "reset" | "image"
                );
            }
        }
        return true;
    }
    false
}

fn is_image(el: &Element) -> bool {
    el.tag == "img" || role(el) == Some("img")
}

fn heading_level(tag: &str) -> Option<u8> {
    match tag {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
}

#[derive(Default)]
struct Ctx {
    violations: Vec<A11yViolation>,
    heading_prev: Option<u8>,
    saw_main: bool,
}

impl Ctx {
    fn push(
        &mut self,
        path: &[String],
        rule: &'static str,
        severity: Severity,
        message: String,
        fix_hint: String,
    ) {
        self.violations.push(A11yViolation {
            path: path.join(" > "),
            rule,
            severity,
            message,
            fix_hint,
        });
    }
}

pub fn check(root: &VNode) -> A11yReport {
    let mut ctx = Ctx::default();
    walk(root, &mut Vec::new(), &mut ctx);
    if !ctx.saw_main {
        ctx.violations.push(A11yViolation {
            path: "document".to_owned(),
            rule: "landmark-main",
            severity: Severity::Minor,
            message: "document has no <main> or role=\"main\" landmark".to_owned(),
            fix_hint: "wrap the primary content in <main aria-label=\"…\">".to_owned(),
        });
    }
    A11yReport {
        violations: ctx.violations,
    }
}

fn label_for(el: &Element) -> String {
    if let Some(a) = el.attrs.iter().find(|a| a.name == "aria-label") {
        if let AttrValue::String(s) = &a.value {
            return format!("<{} aria-label=\"{s}\">", el.tag);
        }
    }
    if let Some(k) = &el.key {
        return format!("<{} #{}>", el.tag, k);
    }
    format!("<{}>", el.tag)
}

fn push_path(path: &mut Vec<String>, el: &Element) {
    let mut seg = el.tag.clone();
    if let Some(r) = role(el) {
        seg.push_str("[role=");
        seg.push_str(r);
        seg.push(']');
    }
    path.push(seg);
}

fn walk(node: &VNode, path: &mut Vec<String>, ctx: &mut Ctx) {
    match node {
        VNode::Empty => {}
        VNode::Text(_) => {}
        VNode::Fragment(children) => {
            for c in children {
                walk(c, path, ctx);
            }
        }
        VNode::Component(c) => {
            path.push(format!("<{}>", c.name));
            walk(&c.rendered, path, ctx);
            path.pop();
        }
        VNode::Element(el) => {
            if el.tag == "main" || role(el) == Some("main") {
                ctx.saw_main = true;
            }
            push_path(path, el);
            lint_element(el, path, ctx);
            if matches!(el.tag.as_str(), "ul" | "ol") {
                lint_list(el, path, ctx);
            }
            for child in &el.children {
                walk(child, path, ctx);
            }
            path.pop();
        }
    }
}

fn lint_element(el: &Element, path: &[String], ctx: &mut Ctx) {
    if let Some(level) = heading_level(&el.tag) {
        if let Some(prev) = ctx.heading_prev {
            if level > prev + 1 {
                ctx.push(
                    path,
                    "heading-order",
                    Severity::Moderate,
                    format!("heading skips a level: h{prev} -> h{level}"),
                    "use a contiguous heading level (e.g. h2 after h1, not h3)".to_owned(),
                );
            }
        }
        ctx.heading_prev = Some(level);
    }

    if is_image(el)
        && !has_attr(el, "alt")
        && !non_empty_attr(el, "aria-label")
        && !non_empty_attr(el, "aria-labelledby")
    {
        ctx.push(
            path,
            "image-alt",
            Severity::Serious,
            format!("image `{}` has no alt text", label_for(el)),
            "add alt=\"…\" for meaningful images or alt=\"\" for decorative".to_owned(),
        );
    }

    if is_form_control(el) && !has_accessible_name(el) {
        ctx.push(
            path,
            "form-label",
            Severity::Serious,
            format!("form control `{}` has no accessible name", label_for(el)),
            "associate a <label>, or set aria-label/aria-labelledby".to_owned(),
        );
    } else if is_interactive(el) && !is_image(el) && !has_accessible_name(el) {
        ctx.push(
            path,
            "interactive-name",
            Severity::Serious,
            format!(
                "interactive element `{}` has no accessible name",
                label_for(el)
            ),
            "add content text, aria-label, aria-labelledby, or title".to_owned(),
        );
    }
}

fn lint_list(el: &Element, path: &[String], ctx: &mut Ctx) {
    for child in &el.children {
        match child {
            VNode::Empty => {}
            VNode::Text(t) if t.trim().is_empty() => {}
            VNode::Text(t) => ctx.push(
                path,
                "list-structure",
                Severity::Moderate,
                format!("`<{}>` contains raw text `{t:?}`", el.tag),
                "wrap list text in `<li>`".to_owned(),
            ),
            VNode::Element(child_el) => {
                if child_el.tag != "li" && role(child_el) != Some("listitem") {
                    ctx.push(
                        path,
                        "list-structure",
                        Severity::Moderate,
                        format!("`<{}>` contains `<{}>`, not `<li>`", el.tag, child_el.tag),
                        "children of a list must be `<li>` or role=\"listitem\"".to_owned(),
                    );
                }
            }
            VNode::Fragment(_) | VNode::Component(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustweb_core::Attr;

    fn el(tag: &str, attrs: Vec<Attr>, children: Vec<VNode>) -> VNode {
        let mut e = Element::new(tag, children);
        e.attrs = attrs;
        VNode::Element(e)
    }

    fn app() -> VNode {
        el(
            "main",
            vec![Attr::new("aria-label", "Todo app")],
            vec![
                el("h1", vec![], vec![VNode::text("Todos")]),
                el("ul", vec![], vec![el("li", vec![], vec![VNode::text("a")])]),
                el(
                    "button",
                    vec![Attr::new("aria-pressed", "false")],
                    vec![VNode::text("Add")],
                ),
            ],
        )
    }

    fn rules(report: &A11yReport) -> Vec<&'static str> {
        report.violations.iter().map(|v| v.rule).collect()
    }

    #[test]
    fn clean_app_has_no_violations() {
        let r = check(&app());
        assert!(r.is_clean(), "unexpected violations: {}", r.to_text());
    }

    #[test]
    fn main_landmark_satisfies() {
        let r = check(&app());
        assert!(!rules(&r).contains(&"landmark-main"));
        let nested = el(
            "section",
            vec![],
            vec![el(
                "main",
                vec![],
                vec![el("h1", vec![], vec![VNode::text("x")])],
            )],
        );
        assert!(!rules(&check(&nested)).contains(&"landmark-main"));
    }

    #[test]
    fn missing_main_is_minor_not_blocking() {
        let t = el(
            "div",
            vec![],
            vec![el("h1", vec![], vec![VNode::text("x")])],
        );
        let r = check(&t);
        assert!(!r.has_blocking());
        assert_eq!(rules(&r), vec!["landmark-main"]);
    }

    #[test]
    fn button_without_name_is_serious() {
        let t = el("main", vec![], vec![el("button", vec![], vec![])]);
        let r = check(&t);
        assert!(r.has_blocking());
        assert!(rules(&r).contains(&"interactive-name"));
    }

    #[test]
    fn link_without_href_is_not_checked() {
        let t = el("main", vec![], vec![el("a", vec![], vec![])]);
        assert!(!rules(&check(&t)).contains(&"interactive-name"));
    }

    #[test]
    fn image_requires_alt() {
        let t = el("main", vec![], vec![el("img", vec![], vec![])]);
        let r = check(&t);
        assert!(rules(&r).contains(&"image-alt"));
        let ok = el(
            "main",
            vec![],
            vec![el("img", vec![Attr::new("alt", "dog")], vec![])],
        );
        assert!(!rules(&check(&ok)).contains(&"image-alt"));
        let deco = el(
            "main",
            vec![],
            vec![el("img", vec![Attr::new("alt", "")], vec![])],
        );
        assert!(!rules(&check(&deco)).contains(&"image-alt"));
    }

    #[test]
    fn input_requires_label() {
        let t = el(
            "main",
            vec![],
            vec![el("input", vec![Attr::new("type", "text")], vec![])],
        );
        assert!(rules(&check(&t)).contains(&"form-label"));
        let ok = el(
            "main",
            vec![],
            vec![el("input", vec![Attr::new("aria-label", "Email")], vec![])],
        );
        assert!(!rules(&check(&ok)).contains(&"form-label"));
        let hidden = el(
            "main",
            vec![],
            vec![el("input", vec![Attr::new("type", "hidden")], vec![])],
        );
        assert!(!rules(&check(&hidden)).contains(&"form-label"));
    }

    #[test]
    fn heading_order_flags_skips() {
        let t = el(
            "main",
            vec![],
            vec![
                el("h1", vec![], vec![VNode::text("a")]),
                el("h4", vec![], vec![VNode::text("b")]),
            ],
        );
        assert!(rules(&check(&t)).contains(&"heading-order"));
        let ok = el(
            "main",
            vec![],
            vec![
                el("h1", vec![], vec![VNode::text("a")]),
                el("h2", vec![], vec![VNode::text("b")]),
            ],
        );
        assert!(!rules(&check(&ok)).contains(&"heading-order"));
    }

    #[test]
    fn list_structure_flags_non_li_children() {
        let t = el(
            "main",
            vec![],
            vec![el(
                "ul",
                vec![],
                vec![el("div", vec![], vec![VNode::text("x")])],
            )],
        );
        assert!(rules(&check(&t)).contains(&"list-structure"));
        let ok = el(
            "main",
            vec![],
            vec![el(
                "ul",
                vec![],
                vec![el("li", vec![], vec![VNode::text("x")])],
            )],
        );
        assert!(!rules(&check(&ok)).contains(&"list-structure"));
    }

    #[test]
    fn empty_report_text() {
        let mut r = A11yReport::default();
        assert!(r.is_clean());
        assert_eq!(r.to_text(), "0 a11y violations");
        r.violations.push(A11yViolation {
            path: "main > button".into(),
            rule: "interactive-name",
            severity: Severity::Serious,
            message: "x".into(),
            fix_hint: "y".into(),
        });
        assert!(!r.is_clean());
        assert!(r.to_text().contains("serious"));
    }
}
