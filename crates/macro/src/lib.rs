use proc_macro::TokenStream;
use proc_macro2::{
    Delimiter, Ident, Literal, Punct, Spacing, Span, TokenStream as TS2, TokenTree as TT2,
};
use quote::{quote, ToTokens};
use std::collections::HashMap;

const KNOWN_ARIA: &[&str] = &[
    "aria-activedescendant",
    "aria-atomic",
    "aria-autocomplete",
    "aria-busy",
    "aria-checked",
    "aria-colcount",
    "aria-colindex",
    "aria-colspan",
    "aria-controls",
    "aria-current",
    "aria-describedby",
    "aria-details",
    "aria-disabled",
    "aria-dropeffect",
    "aria-errormessage",
    "aria-expanded",
    "aria-flowto",
    "aria-grabbed",
    "aria-haspopup",
    "aria-hidden",
    "aria-invalid",
    "aria-keyshortcuts",
    "aria-label",
    "aria-labelledby",
    "aria-level",
    "aria-live",
    "aria-modal",
    "aria-multiline",
    "aria-multiselectable",
    "aria-orientation",
    "aria-owns",
    "aria-placeholder",
    "aria-posinset",
    "aria-pressed",
    "aria-readonly",
    "aria-relevant",
    "aria-required",
    "aria-roledescription",
    "aria-rowcount",
    "aria-rowindex",
    "aria-rowspan",
    "aria-selected",
    "aria-setsize",
    "aria-sort",
    "aria-valuemax",
    "aria-valuemin",
    "aria-valuenow",
    "aria-valuetext",
    "role",
];

fn levenshtein(a: &str, b: &str) -> usize {
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.chars().enumerate() {
            let cost = usize::from(ca != cb);
            cur.push((prev[j] + cost).min(cur[j] + 1).min(prev[j + 1] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

fn suggest_aria(bad: &str) -> Option<&'static str> {
    if !bad.starts_with("aria-") && bad != "role" {
        return None;
    }
    KNOWN_ARIA
        .iter()
        .map(|k| (*k, levenshtein(bad, k)))
        .filter(|(_, d)| *d <= 3)
        .min_by_key(|(_, d)| *d)
        .map(|(k, _)| k)
}

fn props_path_for(path: &TS2, short: &str) -> TS2 {
    let props_short = format!("{short}Props");

    let s = path.to_string();

    let norm = s.replace(' ', "");
    let mut segs: Vec<&str> = norm.split("::").collect();
    if segs.is_empty() {
        let id = Ident::new(&props_short, Span::call_site());
        return quote! { #id };
    }
    segs.pop();
    let mut out = TS2::new();
    for (i, seg) in segs.iter().enumerate() {
        if i > 0 {
            Punct::new(':', Spacing::Joint).to_tokens(&mut out);
            Punct::new(':', Spacing::Alone).to_tokens(&mut out);
        }
        Ident::new(seg, Span::call_site()).to_tokens(&mut out);
    }
    if !segs.is_empty() {
        Punct::new(':', Spacing::Joint).to_tokens(&mut out);
        Punct::new(':', Spacing::Alone).to_tokens(&mut out);
    }
    Ident::new(&props_short, Span::call_site()).to_tokens(&mut out);
    out
}

fn err(span: Span, msg: String) -> syn::Error {
    syn::Error::new(span, msg)
}

struct Cursor {
    toks: Vec<TT2>,
    pos: usize,
}

impl Cursor {
    fn new(stream: TS2) -> Self {
        Self {
            toks: stream.into_iter().collect(),
            pos: 0,
        }
    }
    fn peek(&self) -> Option<&TT2> {
        self.toks.get(self.pos)
    }
    fn peek2(&self) -> Option<&TT2> {
        self.toks.get(self.pos + 1)
    }
    fn next(&mut self) -> Option<TT2> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }
    fn eos(&self) -> bool {
        self.pos >= self.toks.len()
    }
    fn expect_punct(&mut self, ch: char) -> Result<Span, syn::Error> {
        match self.next() {
            Some(TT2::Punct(p)) if p.as_char() == ch => Ok(p.span()),
            Some(other) => Err(err(
                other.span(),
                format!("expected `{ch}`, found `{other}`"),
            )),
            None => Err(err(
                Span::call_site(),
                format!("expected `{ch}`, found end of macro input"),
            )),
        }
    }
    fn peek_punct(&self, ch: char) -> bool {
        matches!(self.peek(), Some(TT2::Punct(p)) if p.as_char() == ch)
    }
}

enum Node {
    Element(ElNode),
    Component(CompNode),
    Fragment(Vec<Node>),
    Expr(TS2),
    Text(String),
}

struct ElNode {
    tag: String,
    attrs: Vec<(String, AttrVal, Span)>,
    listeners: Vec<(String, TS2, Span)>,
    key: Option<(TS2, Span)>,
    children: Vec<Node>,
}

struct CompNode {
    path: TS2,
    name: String,
    props: Vec<(String, TS2, Span)>,
    key: Option<(TS2, Span)>,
    children: Vec<Node>,
}

enum AttrVal {
    Bool(bool),
    Expr(TS2),
}

fn is_component_tag(tag: &str) -> bool {
    tag.chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
}

fn parse_tag_name(c: &mut Cursor) -> Result<(String, Span, TS2), syn::Error> {
    let mut name = String::new();
    let mut ts = TS2::new();
    let first_span: Span;
    match c.next() {
        Some(TT2::Ident(id)) => {
            first_span = id.span();
            name.push_str(&id.to_string());
            id.to_tokens(&mut ts);
        }
        Some(other) => {
            return Err(err(
                other.span(),
                format!("expected tag name, found `{other}`"),
            ))
        }
        None => {
            return Err(err(
                Span::call_site(),
                "expected tag name, found end of input".to_string(),
            ))
        }
    }

    while c.peek_punct(':') && matches!(c.peek2(), Some(TT2::Punct(p)) if p.as_char() == ':') {
        let c1 = c.next().unwrap();
        let c2 = c.next().unwrap();
        c1.to_tokens(&mut ts);
        c2.to_tokens(&mut ts);
        name.push_str("::");
        match c.next() {
            Some(TT2::Ident(id)) => {
                name.push_str(&id.to_string());
                id.to_tokens(&mut ts);
            }
            Some(other) => {
                return Err(err(
                    other.span(),
                    "expected identifier after `::` in tag path".to_string(),
                ))
            }
            None => {
                return Err(err(
                    Span::call_site(),
                    "expected identifier after `::`".to_string(),
                ))
            }
        }
    }

    let short = name.rsplit("::").next().unwrap_or(&name).to_string();
    Ok((short, first_span, ts))
}

fn parse_attr_name(c: &mut Cursor) -> Result<(String, Span), syn::Error> {
    let mut name = String::new();
    let span: Span;
    match c.next() {
        Some(TT2::Ident(id)) => {
            span = id.span();
            name.push_str(&id.to_string());
        }
        Some(other) => {
            return Err(err(
                other.span(),
                format!("expected attribute name, found `{other}`"),
            ))
        }
        None => {
            return Err(err(
                Span::call_site(),
                "expected attribute name".to_string(),
            ))
        }
    }

    loop {
        let dash = c.peek_punct('-');
        let colon = c.peek_punct(':');
        if dash || colon {
            let p: Punct = match c.next().unwrap() {
                TT2::Punct(p) => p,
                _ => unreachable!(),
            };
            name.push(p.as_char());
            match c.peek().cloned() {
                Some(TT2::Ident(id)) => {
                    name.push_str(&id.to_string());
                    c.next();
                }
                Some(other) => {
                    return Err(err(
                        other.span(),
                        format!(
                            "expected identifier after `{}` in attribute `{name}`",
                            p.as_char()
                        ),
                    ))
                }
                None => {
                    return Err(err(
                        Span::call_site(),
                        format!("expected identifier after `{}`", p.as_char()),
                    ))
                }
            }
        } else {
            break;
        }
    }
    Ok((name, span))
}

fn parse_attr_value(c: &mut Cursor) -> Result<AttrVal, syn::Error> {
    match c.peek().cloned() {
        Some(TT2::Group(g)) if g.delimiter() == Delimiter::Brace => {
            let g = match c.next().unwrap() {
                TT2::Group(g) => g,
                _ => unreachable!(),
            };
            Ok(AttrVal::Expr(g.stream()))
        }
        Some(TT2::Literal(lit)) => {
            c.next();
            Ok(AttrVal::Expr({
                let mut t = TS2::new();
                t.extend([TT2::Literal(lit)]);
                t
            }))
        }
        Some(TT2::Ident(id)) if id == "true" || id == "false" => {
            c.next();
            Ok(AttrVal::Bool(id == "true"))
        }
        Some(TT2::Ident(_)) => {

            let id = match c.next().unwrap() {
                TT2::Ident(id) => id,
                _ => unreachable!(),
            };
            let mut t = TS2::new();
            id.to_tokens(&mut t);
            Ok(AttrVal::Expr(t))
        }
        Some(other) => Err(err(other.span(), format!("expected attribute value after `=`, found `{other}`. Hint: wrap Rust expressions in braces, e.g. `attr={{expr}}`"))),
        None => Err(err(Span::call_site(), "expected attribute value, found end of input".to_string())),
    }
}

fn normalize_event(name: &str) -> Option<String> {
    if !name.starts_with("on") || name.len() <= 2 {
        return None;
    }
    let rest = &name[2..];
    let rest = rest.trim_start_matches([':', '-', '_']);
    if rest.is_empty() {
        return None;
    }
    Some(rest.to_lowercase())
}

fn parse_nodes(c: &mut Cursor, stop_tag: Option<&str>) -> Result<Vec<Node>, syn::Error> {
    let mut out = Vec::new();
    loop {
        if c.eos() {
            if let Some(tag) = stop_tag {
                return Err(err(
                    Span::call_site(),
                    format!("unclosed tag `<{tag}>`: missing `</{tag}>`"),
                ));
            }
            break;
        }

        if c.peek_punct('<') {
            let is_close = matches!(c.peek2(), Some(TT2::Punct(p)) if p.as_char() == '/');
            if is_close {
                break;
            }
            out.push(parse_element_or_fragment(c)?);
            continue;
        }

        if let Some(TT2::Group(g)) = c.peek().cloned() {
            if g.delimiter() == Delimiter::Brace {
                let span = g.span();
                let g = match c.next().unwrap() {
                    TT2::Group(g) => g,
                    _ => unreachable!(),
                };
                if g.stream().is_empty() {
                    return Err(err(span, "empty `{}` block is not a valid child. Hint: remove it or use `{html!{…}}`".to_string()));
                }
                out.push(Node::Expr(g.stream()));
                continue;
            }
        }

        if let Some(TT2::Literal(lit)) = c.peek().cloned() {
            let s = lit.to_string();
            if s.starts_with('"') {
                c.next();

                let lit2: syn::LitStr = syn::parse2(quote! { #lit })
                    .map_err(|e| err(lit.span(), format!("invalid string literal child: {e}")))?;
                out.push(Node::Text(lit2.value()));
                continue;
            }
        }

        {
            let mut buf = String::new();
            let mut any = false;
            while let Some(t) = c.peek().cloned() {
                match &t {
                    TT2::Punct(p) if p.as_char() == '<' => break,
                    TT2::Group(g) if g.delimiter() == Delimiter::Brace => break,
                    _ => {
                        buf.push_str(&t.to_string());
                        buf.push(' ');
                        c.next();
                        any = true;
                    }
                }
            }
            if any {
                let trimmed = buf.trim().to_string();
                if !trimmed.is_empty() {
                    out.push(Node::Text(trimmed));
                }
                continue;
            }

            if let Some(t) = c.next() {
                return Err(err(t.span(), format!("unexpected token `{t}` as child. Hint: wrap Rust expressions in braces `{{ … }}`, and text in quotes if it contains special characters")));
            }
        }
    }
    Ok(out)
}

fn parse_element_or_fragment(c: &mut Cursor) -> Result<Node, syn::Error> {
    c.expect_punct('<')?;

    if c.peek_punct('>') {
        c.next();
        let children = parse_nodes(c, Some(""))?;

        c.expect_punct('<')?;
        c.expect_punct('/')?;
        c.expect_punct('>')?;
        return Ok(Node::Fragment(children));
    }
    let (short, tag_span, tag_path) = parse_tag_name(c)?;
    let component = is_component_tag(&short);

    let mut attrs: Vec<(String, AttrVal, Span)> = Vec::new();
    let mut listeners: Vec<(String, TS2, Span)> = Vec::new();
    let mut key: Option<(TS2, Span)> = None;

    let mut seen: HashMap<String, Span> = HashMap::new();

    loop {
        if c.peek_punct('/') {
            c.next();
            c.expect_punct('>')?;
            return finish_tag(TagParts {
                component,
                short,
                tag_path,
                tag_span,
                attrs,
                listeners,
                key,
                children: Vec::new(),
                self_closed: true,
            });
        }
        if c.peek_punct('>') {
            c.next();
            break;
        }
        if c.eos() {
            return Err(err(
                tag_span,
                format!("unclosed tag `<{short}>`: expected `>` or `/>`"),
            ));
        }
        let (aname, aspan) = parse_attr_name(c)?;
        if let Some(prev) = seen.insert(aname.clone(), aspan) {
            let _ = prev;
            return Err(err(
                aspan,
                format!(
                    "duplicate attribute `{aname}` on `<{short}>`. Hint: remove one occurrence"
                ),
            ));
        }

        if aname.starts_with("aria") && aname.len() > 4 && !KNOWN_ARIA.contains(&aname.as_str()) {
            if let Some(sug) = suggest_aria(&aname) {
                return Err(err(aspan, format!("unknown ARIA attribute `{aname}` on `<{short}>`. Did you mean `{sug}`? See https://www.w3.org/TR/wai-aria-1.2/")));
            } else {
                return Err(err(aspan, format!("unknown ARIA attribute `{aname}` on `<{short}>`. Expected a valid `aria-*` attribute or `role`; see https://www.w3.org/TR/wai-aria-1.2/")));
            }
        }

        if aname == "key" {
            if !c.peek_punct('=') {
                return Err(err(aspan, "`key` requires a value: use `key={expr}`. Hint: keys must be unique among siblings".to_string()));
            }
            c.next();
            let v = parse_attr_value(c)?;
            match v {
                AttrVal::Expr(e) => key = Some((e, aspan)),
                AttrVal::Bool(_) => {
                    return Err(err(
                        aspan,
                        "`key` must be a string/number expression, e.g. `key={item.id}`"
                            .to_string(),
                    ))
                }
            }
            continue;
        }

        if let Some(ev) = normalize_event(&aname) {
            if !c.peek_punct('=') {
                return Err(err(aspan, format!("event binding `{aname}` requires a handler: use `{aname}={{handler}}`. Hint: handlers are delegated at the root, no per-node listener cost")));
            }
            c.next();
            let v = parse_attr_value(c)?;
            match v {
                AttrVal::Expr(e) => {
                    if e.is_empty() {
                        return Err(err(aspan, format!("empty handler for `{aname}`. Hint: pass a closure or `Link::send` expression")));
                    }
                    listeners.push((ev, e, aspan));
                }
                _ => return Err(err(aspan, format!("event `{aname}` value must be a Rust expression in braces, e.g. `{aname}={{on_click}}`"))),
            }
            continue;
        }

        if c.peek_punct('=') {
            c.next();
            let v = parse_attr_value(c)?;
            attrs.push((aname, v, aspan));
        } else {
            attrs.push((aname, AttrVal::Bool(true), aspan));
        }
    }

    let children = parse_nodes(c, Some(&short))?;

    c.expect_punct('<').map_err(|_| {
        err(
            tag_span,
            format!("unclosed tag `<{short}>`: missing `</{short}>`"),
        )
    })?;
    c.expect_punct('/').map_err(|_| {
        err(
            tag_span,
            format!("unclosed tag `<{short}>`: missing `</{short}>`"),
        )
    })?;

    let (close_short, _, _) = parse_tag_name(c)?;

    c.expect_punct('>').map_err(|e| {
        err(
            e.span(),
            format!("expected `>` to close `</{close_short}>`"),
        )
    })?;
    if close_short != short {
        return Err(err(tag_span, format!("mismatched tags: opened `<{short}>` but closed `</{close_short}>`. Hint: tags are case-sensitive")));
    }
    finish_tag(TagParts {
        component,
        short,
        tag_path,
        tag_span,
        attrs,
        listeners,
        key,
        children,
        self_closed: false,
    })
}

struct TagParts {
    component: bool,
    short: String,
    tag_path: TS2,
    tag_span: Span,
    attrs: Vec<(String, AttrVal, Span)>,
    listeners: Vec<(String, TS2, Span)>,
    key: Option<(TS2, Span)>,
    children: Vec<Node>,
    self_closed: bool,
}

fn finish_tag(parts: TagParts) -> Result<Node, syn::Error> {
    let TagParts {
        component,
        short,
        tag_path,
        tag_span,
        attrs,
        listeners,
        key,
        children,
        self_closed,
    } = parts;
    if !component && !self_closed {
        const VOID: &[&str] = &[
            "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
            "track", "wbr",
        ];
        if VOID.contains(&short.as_str()) && !children.is_empty() {
            return Err(err(tag_span, format!("<{short}> is a void element and cannot have children. Hint: use `<{short} … />`")));
        }
    }
    if component {
        let mut props = Vec::new();
        for (n, v, s) in attrs {
            let expr = match v {
                AttrVal::Expr(e) => e,
                AttrVal::Bool(b) => {
                    let mut t = TS2::new();
                    Ident::new(if b { "true" } else { "false" }, s).to_tokens(&mut t);
                    t
                }
            };

            let field = if n.contains('-') || n.contains(':') {
                let renamed: String = n
                    .chars()
                    .map(|ch| if ch == '-' || ch == ':' { '_' } else { ch })
                    .collect();
                return Err(err(s, format!("prop `{n}` on `<{short}>` is not a valid Rust field name. Hint: rename the prop to `{renamed}` (dashes become underscores)")));
            } else {
                n
            };
            props.push((field, expr, s));
        }

        for (ev, handler, s) in listeners {
            props.push((format!("on_{ev}"), handler, s));
        }
        return Ok(Node::Component(CompNode {
            path: tag_path,
            name: short,
            props,
            key,
            children,
        }));
    }
    Ok(Node::Element(ElNode {
        tag: short,
        attrs,
        listeners,
        key,
        children,
    }))
}

fn codegen_nodes(nodes: &[Node]) -> TS2 {
    let parts: Vec<TS2> = nodes.iter().map(codegen_node_as_vnodes).collect();
    if parts.is_empty() {
        return quote! { ::rustweb_core::VNode::Empty };
    }
    if parts.len() == 1 {
        let p = &parts[0];
        return quote! {{
            let mut __out: Vec<::rustweb_core::VNode> = Vec::new();
            __out.extend(#p);
            if __out.len() == 1 { __out.into_iter().next().unwrap() } else { ::rustweb_core::VNode::Fragment(__out) }
        }};
    }
    quote! {{
        let mut __out: Vec<::rustweb_core::VNode> = Vec::new();
        #( __out.extend(#parts); )*
        ::rustweb_core::VNode::Fragment(__out)
    }}
}

fn codegen_node_as_vnodes(n: &Node) -> TS2 {
    match n {
        Node::Text(s) => {
            let lit = Literal::string(s);
            quote! { ::rustweb_core::HtmlChild::into_vnodes(#lit) }
        }
        Node::Expr(e) => {
            quote! { ::rustweb_core::HtmlChild::into_vnodes(#e) }
        }
        Node::Fragment(children) => {
            let inner = codegen_children(children);
            quote! { vec![::rustweb_core::VNode::Fragment(#inner)] }
        }
        Node::Element(el) => {
            let inner = codegen_element(el);
            quote! { vec![#inner] }
        }
        Node::Component(c) => {
            let inner = codegen_component(c);
            quote! { vec![#inner] }
        }
    }
}

fn codegen_children(children: &[Node]) -> TS2 {
    if children.is_empty() {
        return quote! { Vec::new() };
    }
    let parts: Vec<TS2> = children.iter().map(codegen_node_as_vnodes).collect();
    quote! {{
        let mut __c: Vec<::rustweb_core::VNode> = Vec::new();
        #( __c.extend(#parts); )*
        __c
    }}
}

fn codegen_element(el: &ElNode) -> TS2 {
    let tag = &el.tag;
    let mut attr_builders = Vec::new();
    for (name, val, _) in &el.attrs {
        let name_lit = Literal::string(name);
        match val {
            AttrVal::Bool(true) => attr_builders.push(quote! { ::rustweb_core::Attr::new(#name_lit, true) }),
            AttrVal::Bool(false) => attr_builders.push(quote! { ::rustweb_core::Attr::new(#name_lit, false) }),
            AttrVal::Expr(e) => attr_builders.push(quote! { ::rustweb_core::Attr::new(#name_lit, ::rustweb_core::IntoAttrValue::into_attr_value(#e)) }),
        }
    }
    let mut listener_builders = Vec::new();
    for (ev, handler, _) in &el.listeners {
        let ev_lit = Literal::string(ev);

        listener_builders.push(quote! {
            (::std::string::ToString::to_string(#ev_lit), ::std::format!("{:?}", &#handler as &dyn ::std::fmt::Debug))
        });
    }
    let children = codegen_children(&el.children);
    let key_setter = match &el.key {
        Some((e, _)) => quote! { __el.key = Some(::std::string::ToString::to_string(&(#e))); },
        None => quote! {},
    };

    let ns_setter = if tag == "svg" || el.tag.starts_with("svg:") {
        quote! { __el.namespace = Some("http://www.w3.org/2000/svg".to_string()); }
    } else {
        quote! {}
    };
    let tag_lit = Literal::string(tag);
    quote! {{
        {
            let __children: Vec<::rustweb_core::VNode> = #children;
            let __attrs: Vec<::rustweb_core::Attr> = vec![#(#attr_builders),*];
            let __listeners: Vec<(String, String)> = vec![#(#listener_builders),*];
            let mut __el = ::rustweb_core::Element {
                tag: ::std::string::ToString::to_string(#tag_lit),
                key: None,
                attrs: __attrs,
                listeners: __listeners,
                children: __children,
                hid: None,
                namespace: None,
            };
            #key_setter
            #ns_setter
            ::rustweb_core::VNode::Element(__el)
        }
    }}
}

fn codegen_component(c: &CompNode) -> TS2 {
    let path = &c.path;
    let props_path = props_path_for(path, &c.name);
    let name_str = Literal::string(&c.name);

    let mut fields: Vec<TS2> = Vec::new();
    for (fname, expr, _) in &c.props {
        let ident = Ident::new(fname, Span::call_site());

        fields.push(quote! { #ident: ::std::convert::Into::into(#expr) });
    }
    let has_children = !c.children.is_empty();
    let children_code = if has_children {
        let inner = codegen_children(&c.children);
        Some(quote! { children: #inner })
    } else {
        None
    };
    let key_code = match &c.key {
        Some((e, _)) => quote! { Some(::std::string::ToString::to_string(&(#e))) },
        None => quote! { None },
    };

    quote! {{
        {

            let __props = #props_path { #(#fields,)* #children_code };

            if let Err(__e) = ::rustweb_core::Props::validate(&__props) {
                ::std::eprintln!("[rustweb] invalid props for component {}: {}", #name_str, __e);
            }
            let __link = ::rustweb_core::Link::new(|_| {});
            let __ctx = ::rustweb_core::Context::new(__props.clone(), __link, Default::default());
            let __rendered: ::rustweb_core::VNode = match <#path as ::rustweb_core::Component>::create(&__ctx) {
                Ok(__state) => match __state.view(&__ctx) {
                    Ok(__v) => __v,
                    Err(__e) => {
                        ::std::eprintln!("[rustweb] view failed for component {}: {}", #name_str, __e);
                        ::rustweb_core::VNode::Empty
                    }
                },
                Err(__e) => {
                    ::std::eprintln!("[rustweb] create failed for component {}: {}", #name_str, __e);
                    ::rustweb_core::VNode::Empty
                }
            };
            ::rustweb_core::VNode::component(#name_str, #key_code, ::std::format!("{:?}", __props), __rendered)
        }
    }}
}

#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    let ts2: TS2 = input.into();
    if ts2.is_empty() {
        return syn::Error::new(Span::call_site(), "html! requires at least one node. Hint: use `html! { <div></div> }` or `html! { <>…</> }`")
            .to_compile_error()
            .into();
    }
    let mut cursor = Cursor::new(ts2);
    let nodes = match parse_nodes(&mut cursor, None) {
        Ok(n) => n,
        Err(e) => return e.to_compile_error().into(),
    };
    if !cursor.eos() {
        let rest: Vec<String> = {
            let mut v = Vec::new();
            while let Some(t) = cursor.next() {
                v.push(t.to_string());
            }
            v
        };
        let msg = format!("unexpected trailing tokens `{}`. Hint: close all tags and wrap siblings in `<>…</>` if needed", rest.join(" "));
        return syn::Error::new(Span::call_site(), msg)
            .to_compile_error()
            .into();
    }
    codegen_nodes(&nodes).into()
}

#[proc_macro_derive(ComponentProps)]
pub fn derive_component_props(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = match syn::parse(input) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };
    let name = &ast.ident;
    let expanded = quote! {
        impl ::rustweb_core::Props for #name {}
    };
    expanded.into()
}
