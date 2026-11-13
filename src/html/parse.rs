//! Converting arbitrary web HTML into a [`Document`].

use ego_tree::NodeRef;
use scraper::{Html, Node};
use url::Url;

use crate::document::{
    Align, Block, Document, Inline, ListItem, Media, MediaKind, Table, TableCell, VAlign,
};

/// Options for [`parse`](super::parse).
#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    /// Base URL used to resolve relative links and media sources, typically
    /// the article URL. Relative URLs are dropped if no base URL is set.
    pub base_url: Option<Url>,
    /// Keep 1×1 images (tracking pixels). Off by default.
    pub keep_tracking_pixels: bool,
}

impl ParseOptions {
    /// Creates options with the given base URL.
    pub fn with_base_url(base_url: Url) -> Self {
        Self {
            base_url: Some(base_url),
            ..Self::default()
        }
    }
}

/// Parses an HTML fragment into a normalized [`Document`].
pub fn parse(html: &str, opts: &ParseOptions) -> Document {
    let dom = Html::parse_fragment(html);
    let root = dom.root_element();
    let cx = Cx { opts };
    let mut pieces = Vec::new();
    for child in root.children() {
        cx.node(child, &mut pieces);
    }
    Document::new(pieces_to_blocks(pieces))
}

/// Output of the converter: inline content, a lifted block, or a paragraph break.
#[derive(Debug)]
enum Piece {
    Inline(Inline),
    Block(Block),
    Break,
}

struct Cx<'o> {
    opts: &'o ParseOptions,
}

type N<'a> = NodeRef<'a, Node>;

const DROP: &[&str] = &[
    "script", "style", "noscript", "template", "iframe", "frame", "frameset", "object", "embed",
    "applet", "form", "button", "select", "option", "textarea", "input", "svg", "canvas", "map",
    "area", "head", "title", "meta", "link", "base", "nav", "dialog", "track", "param", "source",
];

const BLOCK_CONTAINERS: &[&str] = &[
    "div",
    "section",
    "article",
    "main",
    "header",
    "footer",
    "address",
    "aside",
    "center",
    "hgroup",
    "fieldset",
    "legend",
    "dl",
    "dd",
    "figure",
    "figcaption",
    "caption",
    "summary",
    "body",
    "html",
    "tbody",
    "thead",
    "tfoot",
    "tr",
    "td",
    "th",
    "li",
    "blockquote",
];

fn is_html_ws(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0C')
}

fn collapse_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_ws = false;
    for c in s.chars() {
        if is_html_ws(c) {
            if !in_ws {
                out.push(' ');
            }
            in_ws = true;
        } else {
            out.push(c);
            in_ws = false;
        }
    }
    out
}

fn el_name<'a>(n: &N<'a>) -> Option<&'a str> {
    n.value().as_element().map(|e| e.name())
}

fn attr<'a>(n: &N<'a>, name: &str) -> Option<&'a str> {
    n.value().as_element().and_then(|e| e.attr(name))
}

fn has_attr(n: &N<'_>, name: &str) -> bool {
    attr(n, name).is_some()
}

fn raw_text(n: N<'_>) -> String {
    let mut s = String::new();
    for d in n.descendants() {
        if let Some(t) = d.value().as_text() {
            s.push_str(t);
        }
    }
    s
}

fn is_hidden(n: &N<'_>) -> bool {
    if has_attr(n, "hidden") || attr(n, "aria-hidden") == Some("true") {
        return true;
    }
    attr(n, "style").is_some_and(|s| {
        let s: String = s
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .to_ascii_lowercase();
        s.contains("display:none") || s.contains("visibility:hidden")
    })
}

fn parse_align(s: Option<&str>) -> Option<Align> {
    match s?.trim().to_ascii_lowercase().as_str() {
        "left" => Some(Align::Left),
        "center" | "middle" => Some(Align::Center),
        "right" => Some(Align::Right),
        _ => None,
    }
}

fn parse_valign(s: Option<&str>) -> Option<VAlign> {
    match s?.trim().to_ascii_lowercase().as_str() {
        "top" => Some(VAlign::Top),
        "middle" | "center" => Some(VAlign::Middle),
        "bottom" => Some(VAlign::Bottom),
        _ => None,
    }
}

fn parse_span(s: Option<&str>) -> Option<u32> {
    s?.trim().parse::<u32>().ok().filter(|n| *n > 1)
}

fn language_of(n: &N<'_>) -> Option<String> {
    let class = attr(n, "class")?;
    class.split_ascii_whitespace().find_map(|c| {
        c.strip_prefix("language-")
            .or_else(|| c.strip_prefix("lang-"))
            .filter(|l| !l.is_empty())
            .map(str::to_owned)
    })
}

fn is_gif(url: &str) -> bool {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    path.to_ascii_lowercase().ends_with(".gif")
}

/// Wraps runs of inline pieces with `f`, keeping lifted blocks and breaks in place.
fn wrap_pieces(pieces: Vec<Piece>, f: impl Fn(Vec<Inline>) -> Inline) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut run = Vec::new();
    for p in pieces {
        match p {
            Piece::Inline(i) => run.push(i),
            other => {
                if !run.is_empty() {
                    out.push(Piece::Inline(f(std::mem::take(&mut run))));
                }
                out.push(other);
            }
        }
    }
    if !run.is_empty() {
        out.push(Piece::Inline(f(run)));
    }
    out
}

/// Turns pieces into blocks: runs of inline content become paragraphs.
fn pieces_to_blocks(pieces: Vec<Piece>) -> Vec<Block> {
    blocks_with(pieces, Block::Paragraph)
}

fn blocks_with(pieces: Vec<Piece>, f: impl Fn(Vec<Inline>) -> Block) -> Vec<Block> {
    let mut out = Vec::new();
    let mut run = Vec::new();
    let flush = |run: &mut Vec<Inline>, out: &mut Vec<Block>| {
        let inl = normalize_inlines(std::mem::take(run));
        if !inl.is_empty() {
            out.push(f(inl));
        }
    };
    for p in pieces {
        match p {
            Piece::Inline(i) => run.push(i),
            Piece::Break => flush(&mut run, &mut out),
            Piece::Block(b) => {
                flush(&mut run, &mut out);
                out.push(b);
            }
        }
    }
    flush(&mut run, &mut out);
    out
}

/// Flattens pieces into inline content only, turning breaks into spaces and
/// dropping lifted blocks.
fn pieces_to_inlines(pieces: Vec<Piece>) -> Vec<Inline> {
    let mut out = Vec::new();
    for p in pieces {
        match p {
            Piece::Inline(i) => out.push(i),
            Piece::Break => out.push(Inline::Text(" ".into())),
            Piece::Block(_) => {}
        }
    }
    normalize_inlines(out)
}

// ---------------------------------------------------------------------------
// Inline whitespace normalization

fn is_empty_inline(i: &Inline) -> bool {
    match i {
        Inline::Text(t) | Inline::Code(t) | Inline::Math(t) => t.is_empty(),
        Inline::LineBreak => false,
        Inline::Bold(c)
        | Inline::Italic(c)
        | Inline::Underline(c)
        | Inline::Strikethrough(c)
        | Inline::Spoiler(c)
        | Inline::Marked(c)
        | Inline::Subscript(c)
        | Inline::Superscript(c)
        | Inline::Link { content: c, .. } => c.is_empty(),
    }
}

fn children_mut(i: &mut Inline) -> Option<&mut Vec<Inline>> {
    match i {
        Inline::Bold(c)
        | Inline::Italic(c)
        | Inline::Underline(c)
        | Inline::Strikethrough(c)
        | Inline::Spoiler(c)
        | Inline::Marked(c)
        | Inline::Subscript(c)
        | Inline::Superscript(c)
        | Inline::Link { content: c, .. } => Some(c),
        _ => None,
    }
}

/// Collapses whitespace across inline boundaries, trims the edges, merges
/// adjacent text and removes empty wrappers.
pub(crate) fn normalize_inlines(inl: Vec<Inline>) -> Vec<Inline> {
    let mut v = inl;
    let mut prev_space = true; // at start: drop leading whitespace
    collapse_pass(&mut v, &mut prev_space);
    trim_end(&mut v);
    prune(&mut v);
    v
}

fn collapse_pass(v: &mut [Inline], prev_space: &mut bool) {
    for i in v.iter_mut() {
        match i {
            Inline::Text(t) => {
                let mut out = String::with_capacity(t.len());
                for c in t.chars() {
                    if c == ' ' {
                        if !*prev_space {
                            out.push(' ');
                        }
                        *prev_space = true;
                    } else {
                        out.push(c);
                        *prev_space = false;
                    }
                }
                *t = out;
            }
            Inline::LineBreak => *prev_space = true,
            Inline::Code(t) | Inline::Math(t) => {
                if !t.is_empty() {
                    *prev_space = false;
                }
            }
            other => {
                if let Some(c) = children_mut(other) {
                    collapse_pass(c, prev_space);
                }
            }
        }
    }
    // Remove spaces right before line breaks.
    for idx in 0..v.len() {
        if matches!(v.get(idx + 1), Some(Inline::LineBreak)) {
            trim_end_one(&mut v[idx]);
        }
    }
}

fn trim_end_one(i: &mut Inline) -> bool {
    match i {
        Inline::Text(t) => {
            let trimmed = t.trim_end_matches(' ').len();
            t.truncate(trimmed);
            !t.is_empty()
        }
        Inline::LineBreak | Inline::Code(_) | Inline::Math(_) => true,
        other => match children_mut(other) {
            Some(c) => trim_end(c),
            None => true,
        },
    }
}

/// Trims trailing whitespace; returns `true` if non-whitespace content remains.
fn trim_end(v: &mut Vec<Inline>) -> bool {
    while let Some(last) = v.last_mut() {
        if trim_end_one(last) && !is_empty_inline(last) {
            // Trailing line breaks are meaningless.
            if matches!(last, Inline::LineBreak) {
                v.pop();
                continue;
            }
            return true;
        }
        v.pop();
    }
    false
}

fn prune(v: &mut Vec<Inline>) {
    for i in v.iter_mut() {
        if let Some(c) = children_mut(i) {
            prune(c);
        }
    }
    v.retain(|i| !is_empty_inline(i));
    // Drop leading line breaks.
    while matches!(v.first(), Some(Inline::LineBreak)) {
        v.remove(0);
    }
    // Merge adjacent text nodes.
    let mut merged: Vec<Inline> = Vec::with_capacity(v.len());
    for i in v.drain(..) {
        match (merged.last_mut(), i) {
            (Some(Inline::Text(a)), Inline::Text(b)) => a.push_str(&b),
            (_, i) => merged.push(i),
        }
    }
    *v = merged;
}

// ---------------------------------------------------------------------------
// Conversion

impl Cx<'_> {
    fn resolve(&self, raw: &str) -> Option<Url> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }
        match Url::parse(raw) {
            Ok(u) => Some(u),
            Err(url::ParseError::RelativeUrlWithoutBase) => {
                self.opts.base_url.as_ref().and_then(|b| b.join(raw).ok())
            }
            Err(_) => None,
        }
    }

    fn resolve_http(&self, raw: &str) -> Option<String> {
        let u = self.resolve(raw)?;
        matches!(u.scheme(), "http" | "https").then(|| u.to_string())
    }

    fn resolve_link(&self, raw: &str) -> Option<String> {
        if raw.trim_start().starts_with('#') {
            return None;
        }
        let u = self.resolve(raw)?;
        match u.scheme() {
            "http" | "https" | "mailto" | "tel" => Some(u.to_string()),
            "tg" if u.host_str() == Some("user") => Some(u.to_string()),
            _ => None,
        }
    }

    fn children(&self, n: N<'_>, out: &mut Vec<Piece>) {
        for c in n.children() {
            self.node(c, out);
        }
    }

    fn child_pieces(&self, n: N<'_>) -> Vec<Piece> {
        let mut v = Vec::new();
        self.children(n, &mut v);
        v
    }

    fn node(&self, n: N<'_>, out: &mut Vec<Piece>) {
        match n.value() {
            Node::Text(t) => out.push(Piece::Inline(Inline::Text(collapse_ws(t)))),
            Node::Element(_) => self.element(n, out),
            _ => {}
        }
    }

    fn element(&self, n: N<'_>, out: &mut Vec<Piece>) {
        let Some(name) = el_name(&n) else { return };
        if DROP.contains(&name) || is_hidden(&n) {
            return;
        }
        let wrap = |out: &mut Vec<Piece>, f: fn(Vec<Inline>) -> Inline| {
            out.extend(wrap_pieces(self.child_pieces(n), f));
        };
        match name {
            "br" => out.push(Piece::Inline(Inline::LineBreak)),
            "b" | "strong" => wrap(out, Inline::Bold),
            "i" | "em" | "cite" | "dfn" | "var" => wrap(out, Inline::Italic),
            "u" | "ins" => wrap(out, Inline::Underline),
            "s" | "strike" | "del" => wrap(out, Inline::Strikethrough),
            "mark" => wrap(out, Inline::Marked),
            "sub" => wrap(out, Inline::Subscript),
            "sup" => wrap(out, Inline::Superscript),
            "tg-spoiler" => wrap(out, Inline::Spoiler),
            "code" | "kbd" | "samp" | "tt" => {
                let t = collapse_ws(&raw_text(n));
                if !t.trim().is_empty() {
                    out.push(Piece::Inline(Inline::Code(t)));
                }
            }
            "q" => {
                out.push(Piece::Inline(Inline::Text("“".into())));
                self.children(n, out);
                out.push(Piece::Inline(Inline::Text("”".into())));
            }
            "a" => match attr(&n, "href").and_then(|h| self.resolve_link(h)) {
                Some(href) => {
                    out.extend(wrap_pieces(self.child_pieces(n), |content| Inline::Link {
                        href: href.clone(),
                        content,
                    }))
                }
                None => self.children(n, out),
            },
            "math" => self.mathml(n, out),
            "p" => {
                out.push(Piece::Break);
                self.children(n, out);
                out.push(Piece::Break);
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = name.as_bytes()[1] - b'0';
                let pieces = self.child_pieces(n);
                out.push(Piece::Break);
                for b in blocks_with(pieces, |content| Block::Heading { level, content }) {
                    out.push(Piece::Block(b));
                }
                out.push(Piece::Break);
            }
            "pre" => self.pre(n, out),
            "hr" => out.push(Piece::Block(Block::Divider)),
            "ul" | "ol" | "menu" => self.list(n, name == "ol", out),
            "blockquote" => self.blockquote(n, out),
            "details" => self.details(n, out),
            "table" => self.table(n, out),
            "img" => {
                if let Some(m) = self.img(n) {
                    out.push(Piece::Block(Block::Media(m)));
                }
            }
            "picture" => {
                if let Some(m) = self.picture(n) {
                    out.push(Piece::Block(Block::Media(m)));
                }
            }
            "video" | "audio" => {
                if let Some(m) = self.av(n, name == "audio") {
                    out.push(Piece::Block(Block::Media(m)));
                }
            }
            "figure" => self.figure(n, out),
            "dt" => {
                out.push(Piece::Break);
                wrap(out, Inline::Bold);
                out.push(Piece::Break);
            }
            _ if BLOCK_CONTAINERS.contains(&name) => {
                out.push(Piece::Break);
                self.children(n, out);
                out.push(Piece::Break);
            }
            // Unknown and transparent inline elements (span, small, abbr, time, font, …).
            _ => self.children(n, out),
        }
    }

    fn mathml(&self, n: N<'_>, out: &mut Vec<Piece>) {
        let tex = n.descendants().find_map(|d| {
            (el_name(&d) == Some("annotation")
                && attr(&d, "encoding")
                    .is_some_and(|e| e.eq_ignore_ascii_case("application/x-tex")))
            .then(|| raw_text(d).trim().to_owned())
        });
        match tex.filter(|t| !t.is_empty()) {
            Some(t) if attr(&n, "display") == Some("block") => {
                out.push(Piece::Block(Block::MathBlock(t)))
            }
            Some(t) => out.push(Piece::Inline(Inline::Math(t))),
            None => out.push(Piece::Inline(Inline::Text(collapse_ws(&raw_text(n))))),
        }
    }

    fn pre(&self, n: N<'_>, out: &mut Vec<Piece>) {
        let code_child = n.children().find(|c| el_name(c) == Some("code"));
        let language = language_of(&n).or_else(|| code_child.as_ref().and_then(language_of));
        let mut text = raw_text(n);
        if let Some(stripped) = text.strip_prefix('\n') {
            text = stripped.to_owned();
        }
        let text = text.trim_end().to_owned();
        if !text.trim().is_empty() {
            out.push(Piece::Block(Block::Preformatted { language, text }));
        }
    }

    fn list(&self, n: N<'_>, ordered: bool, out: &mut Vec<Piece>) {
        let start = if ordered {
            attr(&n, "start").and_then(|s| s.trim().parse::<i64>().ok())
        } else {
            None
        };
        let mut items = Vec::new();
        let mut stray = Vec::new();
        for c in n.children() {
            if el_name(&c) == Some("li") {
                if !stray.is_empty() {
                    let blocks = pieces_to_blocks(std::mem::take(&mut stray));
                    if !blocks.is_empty() {
                        items.push(ListItem {
                            blocks,
                            checkbox: None,
                        });
                    }
                }
                let checkbox = c
                    .children()
                    .chain(c.children().flat_map(|p| p.children()))
                    .find(|x| el_name(x) == Some("input") && attr(x, "type") == Some("checkbox"))
                    .map(|x| has_attr(&x, "checked"));
                let blocks = pieces_to_blocks(self.child_pieces(c));
                if !blocks.is_empty() || checkbox.is_some() {
                    items.push(ListItem { blocks, checkbox });
                }
            } else {
                self.node(c, &mut stray);
            }
        }
        let blocks = pieces_to_blocks(stray);
        if !blocks.is_empty() {
            items.push(ListItem {
                blocks,
                checkbox: None,
            });
        }
        if !items.is_empty() {
            out.push(Piece::Block(Block::List {
                ordered,
                start,
                items,
            }));
        }
    }

    fn blockquote(&self, n: N<'_>, out: &mut Vec<Piece>) {
        let mut pieces = Vec::new();
        let mut credit = None;
        for c in n.children() {
            if matches!(el_name(&c), Some("cite"))
                && credit.is_none()
                && c.next_sibling().is_none_or(|s| {
                    s.value().as_text().is_some_and(|t| t.trim().is_empty())
                        && s.next_sibling().is_none()
                })
            {
                let inl = pieces_to_inlines(self.child_pieces(c));
                if !inl.is_empty() {
                    credit = Some(inl);
                }
            } else {
                self.node(c, &mut pieces);
            }
        }
        let blocks = pieces_to_blocks(pieces);
        if blocks.is_empty() {
            return;
        }
        let quote = if has_attr(&n, "expandable") {
            let mut content = Vec::new();
            for (i, b) in blocks.into_iter().enumerate() {
                if i > 0 {
                    content.push(Inline::LineBreak);
                }
                content.extend(block_to_inlines(b));
            }
            Block::ExpandableQuote { content, credit }
        } else {
            Block::Quote { blocks, credit }
        };
        out.push(Piece::Block(quote));
    }

    fn details(&self, n: N<'_>, out: &mut Vec<Piece>) {
        let mut summary = Vec::new();
        let mut pieces = Vec::new();
        for c in n.children() {
            if el_name(&c) == Some("summary") && summary.is_empty() {
                summary = pieces_to_inlines(self.child_pieces(c));
            } else {
                self.node(c, &mut pieces);
            }
        }
        let blocks = pieces_to_blocks(pieces);
        if summary.is_empty() && blocks.is_empty() {
            return;
        }
        if summary.is_empty() {
            summary.push(Inline::Text("Details".into()));
        }
        out.push(Piece::Block(Block::Details {
            summary,
            blocks,
            open: has_attr(&n, "open"),
        }));
    }

    fn table(&self, n: N<'_>, out: &mut Vec<Piece>) {
        const LAYOUT_MARKERS: &[&str] = &[
            "table",
            "img",
            "picture",
            "video",
            "audio",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "ul",
            "ol",
            "pre",
            "blockquote",
            "figure",
        ];
        let is_layout = n
            .descendants()
            .skip(1)
            .any(|d| el_name(&d).is_some_and(|e| LAYOUT_MARKERS.contains(&e)));
        if is_layout {
            out.push(Piece::Break);
            self.children(n, out);
            out.push(Piece::Break);
            return;
        }
        let mut table = Table::default();
        for d in n.descendants() {
            match el_name(&d) {
                Some("caption") if table.caption.is_none() => {
                    let c = pieces_to_inlines(self.child_pieces(d));
                    if !c.is_empty() {
                        table.caption = Some(c);
                    }
                }
                Some("tr") => {
                    let row: Vec<TableCell> = d
                        .children()
                        .filter(|c| matches!(el_name(c), Some("td" | "th")))
                        .map(|c| TableCell {
                            content: pieces_to_inlines(self.child_pieces(c)),
                            header: el_name(&c) == Some("th"),
                            colspan: parse_span(attr(&c, "colspan")),
                            rowspan: parse_span(attr(&c, "rowspan")),
                            align: parse_align(attr(&c, "align")),
                            valign: parse_valign(attr(&c, "valign")),
                        })
                        .collect();
                    if !row.is_empty() {
                        table.rows.push(row);
                    }
                }
                _ => {}
            }
        }
        if table
            .rows
            .iter()
            .any(|r| r.iter().any(|c| !c.content.is_empty()))
        {
            table.bordered = true;
            out.push(Piece::Block(Block::Table(table)));
        }
    }

    fn img_src(&self, n: &N<'_>) -> Option<String> {
        for key in [
            "data-src",
            "data-original",
            "data-lazy-src",
            "data-actualsrc",
            "src",
        ] {
            if let Some(u) = attr(n, key).and_then(|v| self.resolve_http(v)) {
                return Some(u);
            }
        }
        for key in ["data-srcset", "srcset"] {
            if let Some(first) = attr(n, key).and_then(|s| s.split(',').next())
                && let Some(u) = first
                    .split_ascii_whitespace()
                    .next()
                    .and_then(|v| self.resolve_http(v))
            {
                return Some(u);
            }
        }
        None
    }

    fn img(&self, n: N<'_>) -> Option<Media> {
        if !self.opts.keep_tracking_pixels {
            let tiny = |k: &str| {
                attr(&n, k)
                    .and_then(|v| v.trim().trim_end_matches("px").parse::<u32>().ok())
                    .is_some_and(|v| v <= 1)
            };
            if tiny("width") || tiny("height") {
                return None;
            }
        }
        let src = self.img_src(&n)?;
        let kind = if is_gif(&src) {
            MediaKind::Animation
        } else {
            MediaKind::Photo
        };
        Some(Media {
            kind,
            src,
            caption: None,
            credit: None,
            spoiler: false,
        })
    }

    fn picture(&self, n: N<'_>) -> Option<Media> {
        if let Some(img) = n.descendants().find(|d| el_name(d) == Some("img"))
            && let Some(m) = self.img(img)
        {
            return Some(m);
        }
        let src = n
            .descendants()
            .filter(|d| el_name(d) == Some("source"))
            .find_map(|s| self.img_src(&s))?;
        Some(Media {
            kind: if is_gif(&src) {
                MediaKind::Animation
            } else {
                MediaKind::Photo
            },
            src,
            caption: None,
            credit: None,
            spoiler: false,
        })
    }

    fn av(&self, n: N<'_>, audio: bool) -> Option<Media> {
        let src = attr(&n, "src")
            .and_then(|s| self.resolve_http(s))
            .or_else(|| {
                n.children()
                    .filter(|c| el_name(c) == Some("source"))
                    .find_map(|c| attr(&c, "src").and_then(|s| self.resolve_http(s)))
            })?;
        let kind = if audio {
            MediaKind::Audio
        } else if is_gif(&src) {
            MediaKind::Animation
        } else {
            MediaKind::Video
        };
        Some(Media {
            kind,
            src,
            caption: None,
            credit: None,
            spoiler: false,
        })
    }

    fn figure(&self, n: N<'_>, out: &mut Vec<Piece>) {
        let caption_node = n.children().find(|c| el_name(c) == Some("figcaption"));
        let mut pieces = Vec::new();
        for c in n.children() {
            if el_name(&c) != Some("figcaption") {
                self.node(c, &mut pieces);
            }
        }
        let media_count = pieces
            .iter()
            .filter(|p| matches!(p, Piece::Block(Block::Media(_))))
            .count();
        let has_text = pieces.iter().any(|p| match p {
            Piece::Inline(Inline::Text(t)) => !t.trim().is_empty(),
            Piece::Inline(_) => true,
            Piece::Block(Block::Media(_)) | Piece::Break => false,
            Piece::Block(_) => true,
        });
        out.push(Piece::Break);
        if media_count > 0 && !has_text {
            // Media figure: attach the caption to the last media element.
            let (caption, credit) = match caption_node {
                Some(cn) => self.figcaption(cn),
                None => (None, None),
            };
            let mut medias: Vec<Media> = pieces
                .into_iter()
                .filter_map(|p| match p {
                    Piece::Block(Block::Media(m)) => Some(m),
                    _ => None,
                })
                .collect();
            if let Some(last) = medias.last_mut() {
                last.caption = caption;
                last.credit = credit;
            }
            out.extend(medias.into_iter().map(|m| Piece::Block(Block::Media(m))));
        } else {
            out.extend(pieces);
            if let Some(cn) = caption_node {
                out.push(Piece::Break);
                self.children(cn, out);
            }
        }
        out.push(Piece::Break);
    }

    fn figcaption(&self, n: N<'_>) -> (Option<Vec<Inline>>, Option<Vec<Inline>>) {
        let mut pieces = Vec::new();
        let mut credit = None;
        for c in n.children() {
            if el_name(&c) == Some("cite") && credit.is_none() {
                let inl = pieces_to_inlines(self.child_pieces(c));
                if !inl.is_empty() {
                    credit = Some(inl);
                }
            } else {
                self.node(c, &mut pieces);
            }
        }
        let caption = pieces_to_inlines(pieces);
        ((!caption.is_empty()).then_some(caption), credit)
    }
}

/// Flattens a block into inline content (used for expandable quotes).
fn block_to_inlines(b: Block) -> Vec<Inline> {
    match b {
        Block::Paragraph(i) | Block::Footer(i) => i,
        Block::Heading { content, .. } => vec![Inline::Bold(content)],
        Block::Preformatted { text, .. } => vec![Inline::Code(text)],
        Block::MathBlock(t) => vec![Inline::Math(t)],
        Block::Quote { blocks, .. } | Block::Details { blocks, .. } => {
            let mut v = Vec::new();
            for (i, b) in blocks.into_iter().enumerate() {
                if i > 0 {
                    v.push(Inline::LineBreak);
                }
                v.extend(block_to_inlines(b));
            }
            v
        }
        Block::List { items, .. } => {
            let mut v = Vec::new();
            for (i, it) in items.into_iter().enumerate() {
                if i > 0 {
                    v.push(Inline::LineBreak);
                }
                v.push(Inline::Text("• ".into()));
                for b in it.blocks {
                    v.extend(block_to_inlines(b));
                }
            }
            v
        }
        Block::ExpandableQuote { content, .. } | Block::PullQuote { content, .. } => content,
        Block::Media(m) => vec![Inline::Link {
            href: m.src.clone(),
            content: vec![Inline::Text(m.src)],
        }],
        Block::Table(_) | Block::Divider | Block::Anchor(_) | Block::Buttons { .. } => Vec::new(),
    }
}
