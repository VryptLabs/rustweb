use rustweb_core::{vnode::escape_html, AttrValue, HydrationError, VNode};

#[derive(Debug, Clone)]
pub struct SsrOptions {
    pub hydration_ids: bool,

    pub checksum: bool,

    pub event_markers: bool,
}

impl Default for SsrOptions {
    fn default() -> Self {
        Self {
            hydration_ids: true,
            checksum: true,
            event_markers: true,
        }
    }
}

pub fn render_to_string(node: &VNode) -> String {
    render_to_string_with(&SsrOptions::default(), node)
}

pub fn render_to_string_with(opts: &SsrOptions, node: &VNode) -> String {
    let mut out = String::new();
    let mut counter = 0u64;
    render_node(opts, node, &mut out, &mut counter);
    if opts.checksum {
        let is_element_root = out.starts_with('<')
            && out
                .as_bytes()
                .get(1)
                .map(|b| b.is_ascii_alphabetic())
                .unwrap_or(false);
        if is_element_root {
            let sum = checksum(&out);

            if let Some(pos) = out.find('>') {
                out.insert_str(pos, &format!(" data-rwh-checksum=\"{sum}\""));
            }
        }
    }
    out
}

fn hid(counter: &mut u64) -> u64 {
    let id = *counter;
    *counter += 1;
    id
}

fn render_node(opts: &SsrOptions, node: &VNode, out: &mut String, counter: &mut u64) {
    match node {
        VNode::Text(t) => out.push_str(&escape_html(t)),
        VNode::Empty => out.push_str("<!---->"),
        VNode::Fragment(children) => {
            for c in children {
                render_node(opts, c, out, counter);
            }
        }
        VNode::Component(c) => {
            out.push_str(&format!("<!--rwc:{}-->", escape_html(&c.name)));
            render_node(opts, &c.rendered, out, counter);
            out.push_str("<!--/rwc-->");
        }
        VNode::Element(el) => {
            let id = hid(counter);
            out.push('<');
            out.push_str(&el.tag);
            if opts.hydration_ids {
                out.push_str(&format!(" data-rwh=\"{id}\""));
            }
            for a in &el.attrs {
                match &a.value {
                    AttrValue::String(v) => {
                        out.push_str(&format!(" {}=\"{}\"", a.name, escape_html(v)))
                    }
                    AttrValue::Bool(true) => out.push_str(&format!(" {}", a.name)),
                    AttrValue::Bool(false) => {}
                    AttrValue::Int(i) => out.push_str(&format!(" {}=\"{i}\"", a.name)),
                }
            }
            if opts.event_markers {
                for (ev, hid_) in &el.listeners {
                    out.push_str(&format!(" data-rwe-{}=\"{}\"", ev, escape_html(hid_)));
                }
            }

            const VOID: &[&str] = &[
                "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
                "source", "track", "wbr",
            ];
            if VOID.contains(&el.tag.as_str()) {
                out.push_str(" />");
                return;
            }
            out.push('>');
            for c in &el.children {
                render_node(opts, c, out, counter);
            }
            out.push_str("</");
            out.push_str(&el.tag);
            out.push('>');
        }
    }
}

pub fn checksum(html: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in html.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}")
}

pub fn verify_hydration(server_html: &str, client: &VNode) -> Result<(), HydrationError> {
    verify_hydration_with(
        &SsrOptions {
            checksum: false,
            ..SsrOptions::default()
        },
        server_html,
        client,
    )
}

fn strip_checksum(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    let marker = " data-rwh-checksum=\"";
    while let Some(pos) = rest.find(marker) {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + marker.len()..];
        match after.find('"') {
            Some(end) => rest = &after[end + 1..],
            None => {
                out.push_str(rest);
                return out;
            }
        }
    }
    out.push_str(rest);
    out
}

pub fn verify_hydration_with(
    opts: &SsrOptions,
    server_html: &str,
    client: &VNode,
) -> Result<(), HydrationError> {
    let mut no_sum = opts.clone();
    no_sum.checksum = false;
    let expected = render_to_string_with(&no_sum, client);
    let server_norm = strip_checksum(server_html);
    if server_norm == expected {
        return Ok(());
    }

    let (byte, hid) = first_diff(&server_norm, &expected);

    let server_ctx = context_around(&server_norm, byte);
    let client_ctx = context_around(&expected, byte);

    if server_ctx.trim_start().starts_with('<') || client_ctx.trim_start().starts_with('<') {
        let hid_str = hid.clone().unwrap_or_else(|| "?".into());
        return Err(HydrationError::Structure {
            hid: hid_str.clone(),
            message: format!("HTML diverges at byte {byte} (hid {hid_str}): server {server_ctx:?} vs client {client_ctx:?}"),
        });
    }
    Err(HydrationError::TextMismatch {
        hid: hid.unwrap_or_else(|| "?".into()),
        server: server_ctx,
        client: client_ctx,
    })
}

fn first_diff(a: &str, b: &str) -> (usize, Option<String>) {
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    let mut i = 0;
    while i < ab.len() && i < bb.len() && ab[i] == bb[i] {
        i += 1;
    }

    let hid = a[..i.min(a.len())].rfind("data-rwh=\"").and_then(|p| {
        let rest = &a[p + 10..];
        rest.find('"').map(|e| rest[..e].to_string())
    });
    (i, hid)
}

fn context_around(s: &str, byte: usize) -> String {
    let start = byte.saturating_sub(40);
    let end = (byte + 40).min(s.len());

    let mut a = start;
    while a < s.len() && !s.is_char_boundary(a) {
        a += 1;
    }
    let mut b = end;
    while b > 0 && !s.is_char_boundary(b) {
        b -= 1;
    }
    s[a.min(b)..b].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustweb_core::{Attr, VNode};

    #[test]
    fn ssr_escapes_and_marks() {
        let n = VNode::element(
            "div",
            vec![Attr::new("title", "<x>&")],
            vec![VNode::text("<hi>&")],
        );
        let html = render_to_string(&n);
        assert!(html.contains("data-rwh=\"0\""), "{html}");
        assert!(html.contains("&lt;hi&gt;&amp;"), "{html}");
        assert!(html.contains("title=\"&lt;x&gt;&amp;\""), "{html}");
    }

    #[test]
    fn deterministic_ids() {
        let a = VNode::element(
            "div",
            vec![],
            vec![VNode::element("span", vec![], vec![VNode::text("x")])],
        );
        assert_eq!(render_to_string(&a), render_to_string(&a));
    }

    #[test]
    fn hydration_ok_when_same() {
        let n = VNode::element(
            "main",
            vec![Attr::new("aria-label", "App")],
            vec![VNode::text("hi")],
        );
        let html = render_to_string(&n);
        assert!(verify_hydration(&html, &n).is_ok());
    }

    #[test]
    fn hydration_detects_text_mismatch() {
        let server = VNode::element("div", vec![], vec![VNode::text("a")]);
        let client = VNode::element("div", vec![], vec![VNode::text("b")]);
        let html = render_to_string(&server);
        let err = verify_hydration(&html, &client).unwrap_err();
        assert!(
            matches!(
                err,
                HydrationError::TextMismatch { .. } | HydrationError::Structure { .. }
            ),
            "{err}"
        );
    }

    #[test]
    fn empty_keeps_placeholder() {
        let html = render_to_string(&VNode::Empty);
        assert_eq!(html, "<!---->");
    }
}
