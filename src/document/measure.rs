//! Measuring a [`Document`] against [`Limits`](crate::Limits).

use super::{Block, Document, Inline, ListItem, Media, Table};
use crate::limits::Stats;

impl Document {
    /// Measures the document.
    ///
    /// Counting is conservative: list items, table rows and the content blocks
    /// of containers are all counted as blocks, and every formatting wrapper
    /// adds one nesting level.
    pub fn stats(&self) -> Stats {
        let mut s = Stats::default();
        for b in &self.blocks {
            block_stats(b, 1, &mut s);
        }
        s
    }
}

/// Cost of a single block in the dimensions that truncation trades off.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Cost {
    pub chars: usize,
    pub blocks: usize,
    pub media: usize,
}

impl Cost {
    pub fn of_block(b: &Block) -> Cost {
        let mut s = Stats::default();
        block_stats(b, 1, &mut s);
        Cost {
            chars: s.chars,
            blocks: s.blocks,
            media: s.media,
        }
    }

    pub fn of_blocks(bs: &[Block]) -> Cost {
        bs.iter()
            .map(Cost::of_block)
            .fold(Cost::default(), |a, b| a + b)
    }

    pub fn fits_in(&self, budget: &Cost) -> bool {
        self.chars <= budget.chars && self.blocks <= budget.blocks && self.media <= budget.media
    }

    pub fn saturating_sub(&self, other: &Cost) -> Cost {
        Cost {
            chars: self.chars.saturating_sub(other.chars),
            blocks: self.blocks.saturating_sub(other.blocks),
            media: self.media.saturating_sub(other.media),
        }
    }
}

impl std::ops::Add for Cost {
    type Output = Cost;
    fn add(self, o: Cost) -> Cost {
        Cost {
            chars: self.chars + o.chars,
            blocks: self.blocks + o.blocks,
            media: self.media + o.media,
        }
    }
}

fn bump_depth(s: &mut Stats, d: usize) {
    s.depth = s.depth.max(d);
}

fn block_stats(b: &Block, depth: usize, s: &mut Stats) {
    s.blocks += 1;
    bump_depth(s, depth);
    match b {
        Block::Paragraph(inl) | Block::Footer(inl) | Block::Heading { content: inl, .. } => {
            inlines_stats(inl, depth, s)
        }
        Block::Preformatted { text, .. } | Block::MathBlock(text) => {
            s.chars += text.chars().count()
        }
        Block::Divider | Block::Anchor(_) => {}
        Block::List { items, .. } => {
            for it in items {
                item_stats(it, depth + 1, s);
            }
        }
        Block::Quote { blocks, credit } => {
            for c in blocks {
                block_stats(c, depth + 1, s);
            }
            opt_inlines_stats(credit, depth, s);
        }
        Block::ExpandableQuote { content, credit } | Block::PullQuote { content, credit } => {
            inlines_stats(content, depth, s);
            opt_inlines_stats(credit, depth, s);
        }
        Block::Media(m) => media_stats(m, depth, s),
        Block::Table(t) => table_stats(t, depth, s),
        Block::Details {
            summary, blocks, ..
        } => {
            inlines_stats(summary, depth, s);
            for c in blocks {
                block_stats(c, depth + 1, s);
            }
        }
        Block::Buttons { buttons, .. } => {
            for btn in buttons {
                s.chars += btn.text.chars().count();
            }
        }
    }
}

fn item_stats(it: &ListItem, depth: usize, s: &mut Stats) {
    s.blocks += 1;
    bump_depth(s, depth);
    for c in &it.blocks {
        block_stats(c, depth + 1, s);
    }
}

fn media_stats(m: &Media, depth: usize, s: &mut Stats) {
    s.media += 1;
    opt_inlines_stats(&m.caption, depth, s);
    opt_inlines_stats(&m.credit, depth, s);
}

fn table_stats(t: &Table, depth: usize, s: &mut Stats) {
    opt_inlines_stats(&t.caption, depth, s);
    for row in &t.rows {
        s.blocks += 1;
        bump_depth(s, depth + 1);
        let cols: usize = row
            .iter()
            .map(|c| c.colspan.unwrap_or(1).max(1) as usize)
            .sum();
        s.max_table_columns = s.max_table_columns.max(cols);
        for cell in row {
            inlines_stats(&cell.content, depth + 1, s);
        }
    }
}

fn opt_inlines_stats(inl: &Option<Vec<Inline>>, depth: usize, s: &mut Stats) {
    if let Some(i) = inl {
        inlines_stats(i, depth, s);
    }
}

pub(crate) fn inlines_stats(inl: &[Inline], depth: usize, s: &mut Stats) {
    for i in inl {
        match i {
            Inline::Text(t) | Inline::Code(t) | Inline::Math(t) => s.chars += t.chars().count(),
            Inline::LineBreak => s.chars += 1,
            Inline::Bold(c)
            | Inline::Italic(c)
            | Inline::Underline(c)
            | Inline::Strikethrough(c)
            | Inline::Spoiler(c)
            | Inline::Marked(c)
            | Inline::Subscript(c)
            | Inline::Superscript(c)
            | Inline::Link { content: c, .. } => {
                bump_depth(s, depth + 1);
                inlines_stats(c, depth + 1, s);
            }
        }
    }
}

/// Number of characters of inline content.
pub(crate) fn inline_chars(inl: &[Inline]) -> usize {
    let mut s = Stats::default();
    inlines_stats(inl, 0, &mut s);
    s.chars
}
