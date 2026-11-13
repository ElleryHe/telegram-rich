//! Intermediate representation of a Telegram rich message document.
//!
//! Arbitrary web HTML is first normalized into this tree (see the `html`
//! module), which only contains constructs that Telegram rich messages can
//! render. A [`Document`] can also be built by hand. It can then be measured
//! against [`Limits`](crate::Limits), truncated at block boundaries with
//! [`Document::fit`] and rendered to Telegram Rich HTML with
//! [`Document::to_html`].

mod fit;
mod measure;
mod render;

pub use fit::FitOptions;

/// A normalized rich message document: an ordered list of top-level blocks.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Document {
    /// Top-level blocks of the document.
    pub blocks: Vec<Block>,
}

/// A block-level element.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// A text paragraph (`<p>`).
    Paragraph(Vec<Inline>),
    /// A section heading (`<h1>`–`<h6>`); `level` is clamped to `1..=6`.
    Heading {
        /// Heading level, 1 is the largest.
        level: u8,
        /// Heading content.
        content: Vec<Inline>,
    },
    /// A preformatted block (`<pre>` / `<pre><code class="language-…">`).
    Preformatted {
        /// Optional programming language.
        language: Option<String>,
        /// Raw text, whitespace preserved.
        text: String,
    },
    /// A footer (`<footer>`).
    Footer(Vec<Inline>),
    /// A horizontal divider (`<hr/>`).
    Divider,
    /// A block formula (`<tg-math-block>`), raw LaTeX.
    MathBlock(String),
    /// An anchor target (`<a name="…"></a>`).
    Anchor(String),
    /// An ordered or unordered list.
    List {
        /// Whether the list is ordered (`<ol>`).
        ordered: bool,
        /// Optional start number for ordered lists.
        start: Option<i64>,
        /// List items.
        items: Vec<ListItem>,
    },
    /// A block quotation (`<blockquote>`), may contain nested blocks.
    Quote {
        /// Quoted content.
        blocks: Vec<Block>,
        /// Optional credit (`<cite>`).
        credit: Option<Vec<Inline>>,
    },
    /// An expandable block quotation (`<blockquote expandable>`), inline content only.
    ExpandableQuote {
        /// Quoted content; use [`Inline::LineBreak`] for line breaks.
        content: Vec<Inline>,
        /// Optional credit (`<cite>`).
        credit: Option<Vec<Inline>>,
    },
    /// A pull quotation (`<aside>`).
    PullQuote {
        /// Quoted content.
        content: Vec<Inline>,
        /// Optional credit (`<cite>`).
        credit: Option<Vec<Inline>>,
    },
    /// A standalone media element.
    Media(Media),
    /// A table; cells contain inline content only.
    Table(Table),
    /// A disclosure block (`<details>`).
    Details {
        /// Always visible summary.
        summary: Vec<Inline>,
        /// Collapsible content.
        blocks: Vec<Block>,
        /// Whether the block is expanded by default.
        open: bool,
    },
    /// A row of URL buttons (`<tg-button-row>`), 1–8 buttons.
    Buttons {
        /// Horizontal alignment.
        align: Option<Align>,
        /// Buttons in the row.
        buttons: Vec<UrlButton>,
    },
}

/// An item of a [`Block::List`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListItem {
    /// Item content.
    pub blocks: Vec<Block>,
    /// `Some(checked)` if the item has a checkbox.
    pub checkbox: Option<bool>,
}

/// A media element.
#[derive(Debug, Clone, PartialEq)]
pub struct Media {
    /// Media kind, rendered as the matching HTML tag.
    pub kind: MediaKind,
    /// Absolute `http(s)://` URL or a `tg://photo?id=…`-style reference.
    pub src: String,
    /// Optional caption (`<figcaption>`).
    pub caption: Option<Vec<Inline>>,
    /// Optional caption credit (`<cite>` inside `<figcaption>`).
    pub credit: Option<Vec<Inline>>,
    /// Whether the media is covered by a spoiler.
    pub spoiler: bool,
}

/// Kind of a [`Media`] element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaKind {
    /// A photo (`<img>`).
    Photo,
    /// A video (`<video>`).
    Video,
    /// An animation, e.g. GIF (`<video>` pointing to an animation).
    Animation,
    /// An audio file (`<audio>`).
    Audio,
    /// A general file (`<tg-document>`).
    Document,
}

/// A table block.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Table {
    /// Rows of cells.
    pub rows: Vec<Vec<TableCell>>,
    /// Optional caption.
    pub caption: Option<Vec<Inline>>,
    /// Draw borders.
    pub bordered: bool,
    /// Striped rows.
    pub striped: bool,
    /// Compact cell padding (Bot API 10.3+).
    pub compact: bool,
}

/// A table cell.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TableCell {
    /// Inline content of the cell.
    pub content: Vec<Inline>,
    /// Header cell (`<th>`).
    pub header: bool,
    /// Column span, if greater than 1.
    pub colspan: Option<u32>,
    /// Row span, if greater than 1.
    pub rowspan: Option<u32>,
    /// Horizontal alignment.
    pub align: Option<Align>,
    /// Vertical alignment.
    pub valign: Option<VAlign>,
}

/// Horizontal alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    /// Left.
    Left,
    /// Center.
    Center,
    /// Right.
    Right,
}

/// Vertical alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VAlign {
    /// Top.
    Top,
    /// Middle.
    Middle,
    /// Bottom.
    Bottom,
}

/// A URL button inside a [`Block::Buttons`] row.
#[derive(Debug, Clone, PartialEq)]
pub struct UrlButton {
    /// Button label (plain text).
    pub text: String,
    /// HTTP(S) or `tg://` URL.
    pub url: String,
    /// Optional style.
    pub style: Option<ButtonStyle>,
}

/// Style of a [`UrlButton`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    /// Red.
    Danger,
    /// Green.
    Success,
    /// Accent color.
    Primary,
}

/// Inline (phrasing) content.
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    /// Plain text.
    Text(String),
    /// Bold.
    Bold(Vec<Inline>),
    /// Italic.
    Italic(Vec<Inline>),
    /// Underline.
    Underline(Vec<Inline>),
    /// Strikethrough.
    Strikethrough(Vec<Inline>),
    /// Spoiler.
    Spoiler(Vec<Inline>),
    /// Marked / highlighted.
    Marked(Vec<Inline>),
    /// Subscript.
    Subscript(Vec<Inline>),
    /// Superscript.
    Superscript(Vec<Inline>),
    /// Inline monospace code (plain text only).
    Code(String),
    /// Inline formula, raw LaTeX.
    Math(String),
    /// A link to an `http(s)`, `mailto:`, `tel:`, `tg://user` URL or an in-document `#anchor`.
    Link {
        /// Target URL.
        href: String,
        /// Link content.
        content: Vec<Inline>,
    },
    /// A line break (`<br>`).
    LineBreak,
}

impl Inline {
    /// Convenience constructor for [`Inline::Text`].
    pub fn text(s: impl Into<String>) -> Self {
        Inline::Text(s.into())
    }
}

impl Block {
    /// Convenience constructor for a paragraph containing plain text.
    pub fn paragraph(s: impl Into<String>) -> Self {
        Block::Paragraph(vec![Inline::Text(s.into())])
    }

    /// Convenience constructor for a single-button row linking to `url`.
    pub fn link_button(text: impl Into<String>, url: impl Into<String>) -> Self {
        Block::Buttons {
            align: None,
            buttons: vec![UrlButton {
                text: text.into(),
                url: url.into(),
                style: None,
            }],
        }
    }
}

impl Document {
    /// Creates a document from blocks.
    pub fn new(blocks: Vec<Block>) -> Self {
        Self { blocks }
    }

    /// Returns `true` if the document has no blocks.
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Visits every media element mutably, e.g. to rewrite URLs to
    /// `tg://photo?id=…` references for explicit uploads.
    pub fn for_each_media_mut(&mut self, mut f: impl FnMut(&mut Media)) {
        fn walk(blocks: &mut [Block], f: &mut dyn FnMut(&mut Media)) {
            for b in blocks {
                match b {
                    Block::Media(m) => f(m),
                    Block::List { items, .. } => {
                        for it in items {
                            walk(&mut it.blocks, f);
                        }
                    }
                    Block::Quote { blocks, .. } | Block::Details { blocks, .. } => walk(blocks, f),
                    _ => {}
                }
            }
        }
        walk(&mut self.blocks, &mut f);
    }

    /// Collects references to every media element in document order.
    pub fn media(&self) -> Vec<&Media> {
        fn walk<'a>(blocks: &'a [Block], out: &mut Vec<&'a Media>) {
            for b in blocks {
                match b {
                    Block::Media(m) => out.push(m),
                    Block::List { items, .. } => {
                        for it in items {
                            walk(&it.blocks, out);
                        }
                    }
                    Block::Quote { blocks, .. } | Block::Details { blocks, .. } => {
                        walk(blocks, out)
                    }
                    _ => {}
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.blocks, &mut out);
        out
    }
}
