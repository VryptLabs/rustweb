use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub type Key = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttrValue {
    String(String),

    Bool(bool),

    Int(i64),
}

impl From<&str> for AttrValue {
    fn from(s: &str) -> Self {
        AttrValue::String(s.to_owned())
    }
}
impl From<String> for AttrValue {
    fn from(s: String) -> Self {
        AttrValue::String(s)
    }
}
impl From<bool> for AttrValue {
    fn from(b: bool) -> Self {
        AttrValue::Bool(b)
    }
}
impl From<i32> for AttrValue {
    fn from(i: i32) -> Self {
        AttrValue::Int(i as i64)
    }
}
impl From<i64> for AttrValue {
    fn from(i: i64) -> Self {
        AttrValue::Int(i)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attr {
    pub name: String,

    pub value: AttrValue,
}

impl Attr {
    pub fn new(name: impl Into<String>, value: impl Into<AttrValue>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    pub fn is_internal(name: &str) -> bool {
        matches!(name, "key" | "ref" | "inner_html")
    }

    pub fn is_aria(name: &str) -> bool {
        name == "role" || name.starts_with("aria-")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Element {
    pub tag: String,

    pub key: Option<Key>,

    pub attrs: Vec<Attr>,

    pub listeners: Vec<(String, String)>,

    pub children: Vec<VNode>,

    pub hid: Option<String>,

    pub namespace: Option<String>,
}

impl Element {
    pub fn new(tag: impl Into<String>, children: Vec<VNode>) -> Self {
        Self {
            tag: tag.into(),
            key: None,
            attrs: Vec::new(),
            listeners: Vec::new(),
            children,
            hid: None,
            namespace: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VComp {
    pub name: String,

    pub key: Option<Key>,

    pub props_json: String,

    pub rendered: Box<VNode>,
}

impl VComp {
    pub fn new(name: impl Into<String>, props_json: String, rendered: VNode) -> Self {
        Self {
            name: name.into(),
            key: None,
            props_json,
            rendered: Box::new(rendered),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VNode {
    Element(Element),

    Text(String),

    Fragment(Vec<VNode>),

    Component(VComp),

    Empty,
}

impl VNode {
    pub fn element(tag: impl Into<String>, attrs: Vec<Attr>, children: Vec<VNode>) -> Self {
        VNode::Element(Element {
            tag: tag.into(),
            key: None,
            attrs,
            listeners: Vec::new(),
            children,
            hid: None,
            namespace: None,
        })
    }

    pub fn text(text: impl Into<String>) -> Self {
        VNode::Text(text.into())
    }

    pub fn fragment(children: Vec<VNode>) -> Self {
        VNode::Fragment(children)
    }

    pub fn component(
        name: impl Into<String>,
        key: Option<Key>,
        props_json: String,
        rendered: VNode,
    ) -> Self {
        let mut c = VComp::new(name, props_json, rendered);
        c.key = key;
        VNode::Component(c)
    }

    pub fn key(&self) -> Option<&str> {
        match self {
            VNode::Element(el) => el.key.as_deref(),
            VNode::Component(c) => c.key.as_deref(),
            VNode::Text(_) | VNode::Fragment(_) | VNode::Empty => None,
        }
    }

    pub fn tag(&self) -> Option<&str> {
        match self {
            VNode::Element(el) => Some(&el.tag),
            _ => None,
        }
    }

    pub fn as_component(&self) -> Option<(&str, &str, &VNode)> {
        match self {
            VNode::Component(c) => Some((&c.name, &c.props_json, &c.rendered)),
            _ => None,
        }
    }

    pub fn resolved(&self) -> &VNode {
        match self {
            VNode::Component(c) => c.rendered.resolved(),
            _ => self,
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, VNode::Empty)
    }

    pub fn child_count(&self) -> usize {
        match self {
            VNode::Element(el) => el.children.len(),
            VNode::Fragment(c) => c.len(),
            VNode::Component(c) => c.rendered.child_count(),
            VNode::Text(_) | VNode::Empty => 0,
        }
    }

    pub fn to_sexpr(&self) -> String {
        match self {
            VNode::Text(t) => format!("{:?}", t),
            VNode::Empty => "()".to_owned(),
            VNode::Fragment(c) => {
                let inner: Vec<String> = c.iter().map(|n| n.to_sexpr()).collect();
                format!("(<> {})", inner.join(" "))
            }
            VNode::Component(c) => {
                format!(
                    "({} props={} {})",
                    c.name,
                    c.props_json,
                    c.rendered.to_sexpr()
                )
            }
            VNode::Element(el) => {
                let mut s = format!("({}", el.tag);
                if let Some(k) = &el.key {
                    s.push_str(&format!(" #{}", k));
                }
                for a in &el.attrs {
                    match &a.value {
                        AttrValue::String(v) => s.push_str(&format!(" {}={:?}", a.name, v)),
                        AttrValue::Bool(true) => s.push_str(&format!(" {}", a.name)),
                        AttrValue::Bool(false) => s.push_str(&format!(" !{}", a.name)),
                        AttrValue::Int(i) => s.push_str(&format!(" {}={}", a.name, i)),
                    }
                }
                for (ev, _) in &el.listeners {
                    s.push_str(&format!(" on:{}", ev));
                }
                for c in &el.children {
                    s.push(' ');
                    s.push_str(&c.to_sexpr());
                }
                s.push(')');
                s
            }
        }
    }

    pub fn to_snapshot_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_owned())
    }
}

impl From<String> for VNode {
    fn from(s: String) -> Self {
        VNode::Text(s)
    }
}
impl From<&str> for VNode {
    fn from(s: &str) -> Self {
        VNode::Text(s.to_owned())
    }
}
impl From<Cow<'_, str>> for VNode {
    fn from(s: Cow<'_, str>) -> Self {
        VNode::Text(s.into_owned())
    }
}

impl From<i32> for VNode {
    fn from(i: i32) -> Self {
        VNode::Text(i.to_string())
    }
}
impl From<i64> for VNode {
    fn from(i: i64) -> Self {
        VNode::Text(i.to_string())
    }
}
impl From<bool> for VNode {
    fn from(b: bool) -> Self {
        VNode::Text(b.to_string())
    }
}

pub trait HtmlChild {
    fn into_vnodes(self) -> Vec<VNode>;
}

impl HtmlChild for VNode {
    fn into_vnodes(self) -> Vec<VNode> {
        match self {
            VNode::Empty => vec![],
            other => vec![other],
        }
    }
}
impl HtmlChild for () {
    fn into_vnodes(self) -> Vec<VNode> {
        vec![]
    }
}
impl HtmlChild for String {
    fn into_vnodes(self) -> Vec<VNode> {
        vec![VNode::Text(self)]
    }
}
impl HtmlChild for &str {
    fn into_vnodes(self) -> Vec<VNode> {
        vec![VNode::Text(self.to_owned())]
    }
}
impl HtmlChild for Cow<'_, str> {
    fn into_vnodes(self) -> Vec<VNode> {
        vec![VNode::Text(self.into_owned())]
    }
}
impl HtmlChild for i32 {
    fn into_vnodes(self) -> Vec<VNode> {
        vec![VNode::Text(self.to_string())]
    }
}
impl HtmlChild for i64 {
    fn into_vnodes(self) -> Vec<VNode> {
        vec![VNode::Text(self.to_string())]
    }
}
impl HtmlChild for usize {
    fn into_vnodes(self) -> Vec<VNode> {
        vec![VNode::Text(self.to_string())]
    }
}
impl HtmlChild for bool {
    fn into_vnodes(self) -> Vec<VNode> {
        if self {
            vec![VNode::Text("true".to_owned())]
        } else {
            vec![]
        }
    }
}
impl<T: HtmlChild> HtmlChild for Option<T> {
    fn into_vnodes(self) -> Vec<VNode> {
        self.map(|v| v.into_vnodes()).unwrap_or_default()
    }
}
impl<T: HtmlChild> HtmlChild for Vec<T> {
    fn into_vnodes(self) -> Vec<VNode> {
        self.into_iter().flat_map(|v| v.into_vnodes()).collect()
    }
}
impl<T: HtmlChild, const N: usize> HtmlChild for [T; N] {
    fn into_vnodes(self) -> Vec<VNode> {
        self.into_iter().flat_map(|v| v.into_vnodes()).collect()
    }
}

impl<T: HtmlChild + Clone> HtmlChild for &T {
    fn into_vnodes(self) -> Vec<VNode> {
        self.clone().into_vnodes()
    }
}

pub trait IntoAttrValue {
    fn into_attr_value(self) -> AttrValue;
}

impl IntoAttrValue for AttrValue {
    fn into_attr_value(self) -> AttrValue {
        self
    }
}
impl IntoAttrValue for String {
    fn into_attr_value(self) -> AttrValue {
        AttrValue::String(self)
    }
}
impl IntoAttrValue for &str {
    fn into_attr_value(self) -> AttrValue {
        AttrValue::String(self.to_owned())
    }
}
impl IntoAttrValue for Cow<'_, str> {
    fn into_attr_value(self) -> AttrValue {
        AttrValue::String(self.into_owned())
    }
}
impl IntoAttrValue for bool {
    fn into_attr_value(self) -> AttrValue {
        AttrValue::Bool(self)
    }
}
impl IntoAttrValue for i32 {
    fn into_attr_value(self) -> AttrValue {
        AttrValue::Int(self as i64)
    }
}
impl IntoAttrValue for i64 {
    fn into_attr_value(self) -> AttrValue {
        AttrValue::Int(self)
    }
}
impl IntoAttrValue for usize {
    fn into_attr_value(self) -> AttrValue {
        AttrValue::Int(self as i64)
    }
}
impl IntoAttrValue for u32 {
    fn into_attr_value(self) -> AttrValue {
        AttrValue::Int(self as i64)
    }
}

pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sexpr_snapshot_basics() {
        let n = VNode::element(
            "div",
            vec![Attr::new("class", "main"), Attr::new("aria-label", "Close")],
            vec![VNode::text("hi")],
        );
        assert_eq!(
            n.to_sexpr(),
            r#"(div class="main" aria-label="Close" "hi")"#
        );
    }

    #[test]
    fn escape_roundtrip() {
        assert_eq!(escape_html("<a>&\"'"), "&lt;a&gt;&amp;&quot;&#39;");
    }
}
