//! Rendering a [`Document`] to Telegram Rich HTML.

use std::fmt::Write as _;

use super::{
    Align, Block, ButtonStyle, Document, Inline, ListItem, Media, MediaKind, Table, VAlign,
};
use crate::types::InputRichMessage;

impl Document {
    /// Renders the document to Telegram Rich HTML, suitable for
    /// [`InputRichMessage::html`].
    pub fn to_html(&self) -> String {
        let mut out = String::new();
        render_blocks(&mut out, &self.blocks);
        out
    }

    /// Renders the document and wraps it into an [`InputRichMessage`].
    pub fn to_input_rich_message(&self) -> InputRichMessage {
        InputRichMessage::html(self.to_html())
    }
}

/// Escapes text content. Only `&`, `<` and `>` need escaping; the API supports
/// a limited set of named entities, so no others are produced.
pub(crate) fn escape_text(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
}

fn escape_attr(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
}

fn attr(out: &mut String, name: &str, value: &str) {
    out.push(' ');
    out.push_str(name);
    out.push_str("=\"");
    escape_attr(out, value);
    out.push('"');
}

fn render_blocks(out: &mut String, blocks: &[Block]) {
    for b in blocks {
        render_block(out, b);
    }
}

/// Renders block content; a lone paragraph is rendered inline to keep list
/// items and quotes compact.
fn render_flow(out: &mut String, blocks: &[Block]) {
    match blocks {
        [Block::Paragraph(inl)] => render_inlines(out, inl),
        _ => render_blocks(out, blocks),
    }
}

fn wrap_inlines(out: &mut String, tag: &str, inl: &[Inline]) {
    let _ = write!(out, "<{tag}>");
    render_inlines(out, inl);
    let _ = write!(out, "</{tag}>");
}

fn render_cite(out: &mut String, credit: &Option<Vec<Inline>>) {
    if let Some(c) = credit {
        wrap_inlines(out, "cite", c);
    }
}

fn render_block(out: &mut String, b: &Block) {
    match b {
        Block::Paragraph(inl) => wrap_inlines(out, "p", inl),
        Block::Heading { level, content } => {
            let tag = format!("h{}", (*level).clamp(1, 6));
            wrap_inlines(out, &tag, content);
        }
        Block::Preformatted { language, text } => {
            out.push_str("<pre>");
            match language {
                Some(lang) => {
                    out.push_str("<code");
                    attr(out, "class", &format!("language-{lang}"));
                    out.push('>');
                    escape_text(out, text);
                    out.push_str("</code>");
                }
                None => escape_text(out, text),
            }
            out.push_str("</pre>");
        }
        Block::Footer(inl) => wrap_inlines(out, "footer", inl),
        Block::Divider => out.push_str("<hr/>"),
        Block::MathBlock(src) => {
            out.push_str("<tg-math-block>");
            escape_text(out, src);
            out.push_str("</tg-math-block>");
        }
        Block::Anchor(name) => {
            out.push_str("<a");
            attr(out, "name", name);
            out.push_str("></a>");
        }
        Block::List {
            ordered,
            start,
            items,
        } => {
            if *ordered {
                out.push_str("<ol");
                if let Some(s) = start {
                    attr(out, "start", &s.to_string());
                }
                out.push('>');
            } else {
                out.push_str("<ul>");
            }
            for it in items {
                render_list_item(out, it);
            }
            out.push_str(if *ordered { "</ol>" } else { "</ul>" });
        }
        Block::Quote { blocks, credit } => {
            out.push_str("<blockquote>");
            render_flow(out, blocks);
            render_cite(out, credit);
            out.push_str("</blockquote>");
        }
        Block::ExpandableQuote { content, credit } => {
            out.push_str("<blockquote expandable>");
            render_inlines(out, content);
            render_cite(out, credit);
            out.push_str("</blockquote>");
        }
        Block::PullQuote { content, credit } => {
            out.push_str("<aside>");
            render_inlines(out, content);
            render_cite(out, credit);
            out.push_str("</aside>");
        }
        Block::Media(m) => render_media(out, m),
        Block::Table(t) => render_table(out, t),
        Block::Details {
            summary,
            blocks,
            open,
        } => {
            out.push_str(if *open { "<details open>" } else { "<details>" });
            wrap_inlines(out, "summary", summary);
            render_blocks(out, blocks);
            out.push_str("</details>");
        }
        Block::Buttons { align, buttons } => {
            out.push_str("<tg-button-row");
            if let Some(a) = align {
                attr(out, "align", align_str(*a));
            }
            out.push('>');
            for btn in buttons {
                out.push_str("<tg-button type=\"url\"");
                if let Some(s) = btn.style {
                    attr(
                        out,
                        "style",
                        match s {
                            ButtonStyle::Danger => "danger",
                            ButtonStyle::Success => "success",
                            ButtonStyle::Primary => "primary",
                        },
                    );
                }
                attr(out, "url", &btn.url);
                out.push('>');
                escape_text(out, &btn.text);
                out.push_str("</tg-button>");
            }
            out.push_str("</tg-button-row>");
        }
    }
}

fn render_list_item(out: &mut String, it: &ListItem) {
    out.push_str("<li>");
    match it.checkbox {
        Some(true) => out.push_str("<input type=\"checkbox\" checked>"),
        Some(false) => out.push_str("<input type=\"checkbox\">"),
        None => {}
    }
    render_flow(out, &it.blocks);
    out.push_str("</li>");
}

fn render_media(out: &mut String, m: &Media) {
    let has_caption = m.caption.is_some() || m.credit.is_some();
    if has_caption {
        out.push_str("<figure>");
    }
    match m.kind {
        MediaKind::Photo => {
            out.push_str("<img");
            attr(out, "src", &m.src);
            if m.spoiler {
                out.push_str(" tg-spoiler");
            }
            out.push_str("/>");
        }
        MediaKind::Video | MediaKind::Animation => {
            out.push_str("<video");
            attr(out, "src", &m.src);
            if m.spoiler {
                out.push_str(" tg-spoiler");
            }
            out.push_str("></video>");
        }
        MediaKind::Audio => {
            out.push_str("<audio");
            attr(out, "src", &m.src);
            out.push_str("></audio>");
        }
        MediaKind::Document => {
            out.push_str("<tg-document");
            attr(out, "src", &m.src);
            out.push_str("></tg-document>");
        }
    }
    if has_caption {
        out.push_str("<figcaption>");
        if let Some(c) = &m.caption {
            render_inlines(out, c);
        }
        render_cite(out, &m.credit);
        out.push_str("</figcaption></figure>");
    }
}

fn align_str(a: Align) -> &'static str {
    match a {
        Align::Left => "left",
        Align::Center => "center",
        Align::Right => "right",
    }
}

fn render_table(out: &mut String, t: &Table) {
    out.push_str("<table");
    if t.bordered {
        out.push_str(" bordered");
    }
    if t.striped {
        out.push_str(" striped");
    }
    if t.compact {
        out.push_str(" compact");
    }
    out.push('>');
    if let Some(c) = &t.caption {
        wrap_inlines(out, "caption", c);
    }
    for row in &t.rows {
        out.push_str("<tr>");
        for cell in row {
            let tag = if cell.header { "th" } else { "td" };
            out.push('<');
            out.push_str(tag);
            if let Some(n) = cell.colspan.filter(|n| *n > 1) {
                attr(out, "colspan", &n.to_string());
            }
            if let Some(n) = cell.rowspan.filter(|n| *n > 1) {
                attr(out, "rowspan", &n.to_string());
            }
            if let Some(a) = cell.align {
                attr(out, "align", align_str(a));
            }
            if let Some(v) = cell.valign {
                attr(
                    out,
                    "valign",
                    match v {
                        VAlign::Top => "top",
                        VAlign::Middle => "middle",
                        VAlign::Bottom => "bottom",
                    },
                );
            }
            out.push('>');
            render_inlines(out, &cell.content);
            let _ = write!(out, "</{tag}>");
        }
        out.push_str("</tr>");
    }
    out.push_str("</table>");
}

pub(crate) fn render_inlines(out: &mut String, inl: &[Inline]) {
    for i in inl {
        render_inline(out, i);
    }
}

fn render_inline(out: &mut String, i: &Inline) {
    match i {
        Inline::Text(t) => escape_text(out, t),
        Inline::Bold(c) => wrap_inlines(out, "b", c),
        Inline::Italic(c) => wrap_inlines(out, "i", c),
        Inline::Underline(c) => wrap_inlines(out, "u", c),
        Inline::Strikethrough(c) => wrap_inlines(out, "s", c),
        Inline::Spoiler(c) => wrap_inlines(out, "tg-spoiler", c),
        Inline::Marked(c) => wrap_inlines(out, "mark", c),
        Inline::Subscript(c) => wrap_inlines(out, "sub", c),
        Inline::Superscript(c) => wrap_inlines(out, "sup", c),
        Inline::Code(t) => {
            out.push_str("<code>");
            escape_text(out, t);
            out.push_str("</code>");
        }
        Inline::Math(t) => {
            out.push_str("<tg-math>");
            escape_text(out, t);
            out.push_str("</tg-math>");
        }
        Inline::Link { href, content } => {
            out.push_str("<a");
            attr(out, "href", href);
            out.push('>');
            render_inlines(out, content);
            out.push_str("</a>");
        }
        Inline::LineBreak => out.push_str("<br/>"),
    }
}
