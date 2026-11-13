//! Fitting a [`Document`] into [`Limits`] by truncating at block boundaries.

use super::measure::{Cost, inline_chars};
use super::{Block, Document, Inline, ListItem, Media, Table};
use crate::limits::Limits;

/// Options for [`Document::fit`].
#[derive(Debug, Clone, PartialEq)]
pub struct FitOptions {
    /// Limits to fit into; defaults to [`Limits::TELEGRAM`].
    pub limits: Limits,
    /// Blocks that are always appended at the end, e.g. a "Read original"
    /// button row. Their cost is reserved before truncating the content.
    pub tail: Vec<Block>,
    /// Blocks appended (before `tail`) only if the content had to be
    /// truncated, e.g. "The article is too long, read the rest at …".
    pub truncation_notice: Vec<Block>,
    /// Text appended to a text block that was cut in the middle.
    pub ellipsis: String,
}

impl Default for FitOptions {
    fn default() -> Self {
        Self {
            limits: Limits::TELEGRAM,
            tail: Vec::new(),
            truncation_notice: Vec::new(),
            ellipsis: "…".to_owned(),
        }
    }
}

impl Document {
    /// Fits the document into `opts.limits` and appends `opts.tail`.
    ///
    /// The following adjustments are applied in order:
    ///
    /// 1. Table rows wider than `max_table_columns` lose their extra cells.
    /// 2. Media beyond `max_media` are replaced by plain links.
    /// 3. Formatting and containers nested deeper than `max_depth` are
    ///    unwrapped / flattened.
    /// 4. If characters or blocks still exceed the limits, the content is cut
    ///    at the last block that fits (descending into lists, quotes, details
    ///    and tables, and cutting text blocks mid-text if needed), and
    ///    `opts.truncation_notice` is appended.
    ///
    /// Returns `true` if content was truncated in step 4.
    pub fn fit(&mut self, opts: &FitOptions) -> bool {
        let limits = &opts.limits;
        let tail_cost = Cost::of_blocks(&opts.tail);
        let notice_cost = Cost::of_blocks(&opts.truncation_notice);

        limit_table_columns(&mut self.blocks, limits.max_table_columns);
        let mut media_left = limits
            .max_media
            .saturating_sub(tail_cost.media + notice_cost.media);
        limit_media(&mut self.blocks, &mut media_left);
        let blocks = std::mem::take(&mut self.blocks);
        self.blocks = limit_depth(blocks, 1, limits.max_depth);

        let full = Cost {
            chars: limits.max_chars,
            blocks: limits.max_blocks,
            media: limits.max_media,
        };
        let content_cost = Cost::of_blocks(&self.blocks);
        let truncated = if (content_cost + tail_cost).fits_in(&full) {
            false
        } else {
            let mut budget = full.saturating_sub(&(tail_cost + notice_cost));
            let blocks = std::mem::take(&mut self.blocks);
            let (kept, _) = take_blocks(blocks, &mut budget, &opts.ellipsis);
            self.blocks = kept;
            self.blocks.extend(opts.truncation_notice.iter().cloned());
            true
        };
        self.blocks.extend(opts.tail.iter().cloned());
        truncated
    }
}

// ---------------------------------------------------------------------------
// Step 1: table columns

fn limit_table_columns(blocks: &mut [Block], max_cols: usize) {
    for b in blocks {
        match b {
            Block::Table(t) => {
                for row in &mut t.rows {
                    let mut used = 0usize;
                    let mut keep = 0usize;
                    for cell in row.iter_mut() {
                        if used >= max_cols {
                            break;
                        }
                        let span = cell.colspan.unwrap_or(1).max(1) as usize;
                        if used + span > max_cols {
                            cell.colspan = Some((max_cols - used) as u32);
                        }
                        used += span.min(max_cols - used);
                        keep += 1;
                    }
                    row.truncate(keep);
                }
            }
            Block::List { items, .. } => {
                for it in items {
                    limit_table_columns(&mut it.blocks, max_cols);
                }
            }
            Block::Quote { blocks, .. } | Block::Details { blocks, .. } => {
                limit_table_columns(blocks, max_cols)
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Step 2: media count

fn media_as_link(m: &Media) -> Block {
    let content = match &m.caption {
        Some(c) if !c.is_empty() => c.clone(),
        _ => vec![Inline::Text(m.src.clone())],
    };
    Block::Paragraph(vec![Inline::Link {
        href: m.src.clone(),
        content,
    }])
}

fn limit_media(blocks: &mut [Block], left: &mut usize) {
    for b in blocks {
        match b {
            Block::Media(m) => {
                if *left == 0 {
                    *b = media_as_link(m);
                } else {
                    *left -= 1;
                }
            }
            Block::List { items, .. } => {
                for it in items {
                    limit_media(&mut it.blocks, left);
                }
            }
            Block::Quote { blocks, .. } | Block::Details { blocks, .. } => {
                limit_media(blocks, left)
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Step 3: nesting depth

fn limit_inline_depth(inl: Vec<Inline>, depth: usize, max: usize) -> Vec<Inline> {
    let mut out = Vec::with_capacity(inl.len());
    for i in inl {
        let rebuild = |c: Vec<Inline>, f: fn(Vec<Inline>) -> Inline, out: &mut Vec<Inline>| {
            if depth + 1 > max {
                out.extend(limit_inline_depth(c, depth, max));
            } else {
                out.push(f(limit_inline_depth(c, depth + 1, max)));
            }
        };
        match i {
            Inline::Bold(c) => rebuild(c, Inline::Bold, &mut out),
            Inline::Italic(c) => rebuild(c, Inline::Italic, &mut out),
            Inline::Underline(c) => rebuild(c, Inline::Underline, &mut out),
            Inline::Strikethrough(c) => rebuild(c, Inline::Strikethrough, &mut out),
            Inline::Spoiler(c) => rebuild(c, Inline::Spoiler, &mut out),
            Inline::Marked(c) => rebuild(c, Inline::Marked, &mut out),
            Inline::Subscript(c) => rebuild(c, Inline::Subscript, &mut out),
            Inline::Superscript(c) => rebuild(c, Inline::Superscript, &mut out),
            Inline::Link { href, content } => {
                if depth + 1 > max {
                    out.extend(limit_inline_depth(content, depth, max));
                } else {
                    out.push(Inline::Link {
                        href,
                        content: limit_inline_depth(content, depth + 1, max),
                    });
                }
            }
            other => out.push(other),
        }
    }
    out
}

fn limit_opt_inline(inl: Option<Vec<Inline>>, depth: usize, max: usize) -> Option<Vec<Inline>> {
    inl.map(|i| limit_inline_depth(i, depth, max))
}

/// Limits nesting; `depth` is the depth the given blocks live at.
fn limit_depth(blocks: Vec<Block>, depth: usize, max: usize) -> Vec<Block> {
    let mut out = Vec::with_capacity(blocks.len());
    for b in blocks {
        match b {
            Block::Paragraph(i) => out.push(Block::Paragraph(limit_inline_depth(i, depth, max))),
            Block::Footer(i) => out.push(Block::Footer(limit_inline_depth(i, depth, max))),
            Block::Heading { level, content } => out.push(Block::Heading {
                level,
                content: limit_inline_depth(content, depth, max),
            }),
            Block::ExpandableQuote { content, credit } => out.push(Block::ExpandableQuote {
                content: limit_inline_depth(content, depth, max),
                credit: limit_opt_inline(credit, depth, max),
            }),
            Block::PullQuote { content, credit } => out.push(Block::PullQuote {
                content: limit_inline_depth(content, depth, max),
                credit: limit_opt_inline(credit, depth, max),
            }),
            Block::List {
                ordered,
                start,
                items,
            } => {
                // list (depth) -> item (depth+1) -> item blocks (depth+2)
                if depth + 2 > max {
                    for it in items {
                        out.extend(limit_depth(it.blocks, depth, max));
                    }
                } else {
                    let items = items
                        .into_iter()
                        .map(|it| ListItem {
                            blocks: limit_depth(it.blocks, depth + 2, max),
                            checkbox: it.checkbox,
                        })
                        .collect();
                    out.push(Block::List {
                        ordered,
                        start,
                        items,
                    });
                }
            }
            Block::Quote { blocks, credit } => {
                if depth + 1 > max {
                    out.extend(limit_depth(blocks, depth, max));
                } else {
                    out.push(Block::Quote {
                        blocks: limit_depth(blocks, depth + 1, max),
                        credit: limit_opt_inline(credit, depth, max),
                    });
                }
            }
            Block::Details {
                summary,
                blocks,
                open,
            } => {
                if depth + 1 > max {
                    out.push(Block::Paragraph(limit_inline_depth(summary, depth, max)));
                    out.extend(limit_depth(blocks, depth, max));
                } else {
                    out.push(Block::Details {
                        summary: limit_inline_depth(summary, depth, max),
                        blocks: limit_depth(blocks, depth + 1, max),
                        open,
                    });
                }
            }
            Block::Table(mut t) => {
                for row in &mut t.rows {
                    for cell in row {
                        cell.content =
                            limit_inline_depth(std::mem::take(&mut cell.content), depth + 1, max);
                    }
                }
                t.caption = limit_opt_inline(t.caption, depth, max);
                out.push(Block::Table(t));
            }
            Block::Media(mut m) => {
                m.caption = limit_opt_inline(m.caption, depth, max);
                m.credit = limit_opt_inline(m.credit, depth, max);
                out.push(Block::Media(m));
            }
            other => out.push(other),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Step 4: truncation

/// Takes blocks while they fit into `budget`, partially truncating the first
/// block that does not fit. Returns the kept blocks and whether anything was
/// dropped.
fn take_blocks(blocks: Vec<Block>, budget: &mut Cost, ellipsis: &str) -> (Vec<Block>, bool) {
    let mut kept = Vec::new();
    for b in blocks {
        let c = Cost::of_block(&b);
        if c.fits_in(budget) {
            *budget = budget.saturating_sub(&c);
            kept.push(b);
        } else {
            if let Some(p) = truncate_block(b, budget, ellipsis) {
                let pc = Cost::of_block(&p);
                *budget = budget.saturating_sub(&pc);
                kept.push(p);
            }
            return (kept, true);
        }
    }
    (kept, false)
}

/// Cuts inline content to at most `max_chars` characters (including the ellipsis).
fn truncate_inlines(inl: Vec<Inline>, max_chars: usize, ellipsis: &str) -> Option<Vec<Inline>> {
    let ell = ellipsis.chars().count();
    if max_chars <= ell {
        return None;
    }
    let mut left = max_chars - ell;
    let mut out = cut_inlines(inl, &mut left);
    if out.is_empty() {
        return None;
    }
    out.push(Inline::Text(ellipsis.to_owned()));
    Some(out)
}

fn cut_inlines(inl: Vec<Inline>, left: &mut usize) -> Vec<Inline> {
    let mut out = Vec::new();
    for i in inl {
        if *left == 0 {
            break;
        }
        let n = inline_chars(std::slice::from_ref(&i));
        if n <= *left {
            *left -= n;
            out.push(i);
            continue;
        }
        let wrap = |c: Vec<Inline>, f: fn(Vec<Inline>) -> Inline, left: &mut usize| {
            let c = cut_inlines(c, left);
            (!c.is_empty()).then(|| f(c))
        };
        let part = match i {
            Inline::Text(t) => Some(Inline::Text(take_chars(&t, left))),
            Inline::Code(t) => Some(Inline::Code(take_chars(&t, left))),
            Inline::Math(_) | Inline::LineBreak => None,
            Inline::Bold(c) => wrap(c, Inline::Bold, left),
            Inline::Italic(c) => wrap(c, Inline::Italic, left),
            Inline::Underline(c) => wrap(c, Inline::Underline, left),
            Inline::Strikethrough(c) => wrap(c, Inline::Strikethrough, left),
            Inline::Spoiler(c) => wrap(c, Inline::Spoiler, left),
            Inline::Marked(c) => wrap(c, Inline::Marked, left),
            Inline::Subscript(c) => wrap(c, Inline::Subscript, left),
            Inline::Superscript(c) => wrap(c, Inline::Superscript, left),
            Inline::Link { href, content } => {
                let c = cut_inlines(content, left);
                (!c.is_empty()).then_some(Inline::Link { href, content: c })
            }
        };
        out.extend(part);
        *left = 0;
    }
    out
}

fn take_chars(s: &str, left: &mut usize) -> String {
    let out: String = s.chars().take(*left).collect();
    *left -= out.chars().count();
    out
}

fn truncate_block(b: Block, budget: &Cost, ellipsis: &str) -> Option<Block> {
    if budget.blocks == 0 {
        return None;
    }
    // Budget for content inside a container that itself costs one block.
    let inner = Cost {
        blocks: budget.blocks - 1,
        ..*budget
    };
    match b {
        Block::Paragraph(i) => truncate_inlines(i, budget.chars, ellipsis).map(Block::Paragraph),
        Block::Footer(i) => truncate_inlines(i, budget.chars, ellipsis).map(Block::Footer),
        Block::Heading { level, content } => truncate_inlines(content, budget.chars, ellipsis)
            .map(|content| Block::Heading { level, content }),
        Block::PullQuote { content, .. } => {
            truncate_inlines(content, budget.chars, ellipsis).map(|content| Block::PullQuote {
                content,
                credit: None,
            })
        }
        Block::ExpandableQuote { content, .. } => truncate_inlines(content, budget.chars, ellipsis)
            .map(|content| Block::ExpandableQuote {
                content,
                credit: None,
            }),
        Block::Preformatted { language, text } => {
            let ell = ellipsis.chars().count();
            if budget.chars <= ell {
                return None;
            }
            let mut left = budget.chars - ell;
            let mut text = take_chars(&text, &mut left);
            text.push_str(ellipsis);
            Some(Block::Preformatted { language, text })
        }
        Block::List {
            ordered,
            start,
            items,
        } => {
            let mut budget = inner;
            let mut kept = Vec::new();
            for it in items {
                let item_cost = Cost::of_blocks(&it.blocks)
                    + Cost {
                        blocks: 1,
                        ..Cost::default()
                    };
                if item_cost.fits_in(&budget) {
                    budget = budget.saturating_sub(&item_cost);
                    kept.push(it);
                } else {
                    if budget.blocks > 1 {
                        let mut ib = Cost {
                            blocks: budget.blocks - 1,
                            ..budget
                        };
                        let (blocks, _) = take_blocks(it.blocks, &mut ib, ellipsis);
                        if !blocks.is_empty() {
                            kept.push(ListItem {
                                blocks,
                                checkbox: it.checkbox,
                            });
                        }
                    }
                    break;
                }
            }
            (!kept.is_empty()).then_some(Block::List {
                ordered,
                start,
                items: kept,
            })
        }
        Block::Quote { blocks, .. } => {
            let mut budget = inner;
            let (blocks, _) = take_blocks(blocks, &mut budget, ellipsis);
            (!blocks.is_empty()).then_some(Block::Quote {
                blocks,
                credit: None,
            })
        }
        Block::Details {
            summary,
            blocks,
            open,
        } => {
            let sc = inline_chars(&summary);
            if sc >= inner.chars {
                return None;
            }
            let mut budget = Cost {
                chars: inner.chars - sc,
                ..inner
            };
            let (blocks, _) = take_blocks(blocks, &mut budget, ellipsis);
            (!blocks.is_empty()).then_some(Block::Details {
                summary,
                blocks,
                open,
            })
        }
        Block::Table(t) => truncate_table(t, &inner),
        // Atomic blocks.
        Block::Media(_)
        | Block::Divider
        | Block::Anchor(_)
        | Block::MathBlock(_)
        | Block::Buttons { .. } => None,
    }
}

fn truncate_table(t: Table, budget: &Cost) -> Option<Block> {
    let cap = t.caption.as_deref().map(inline_chars).unwrap_or(0);
    let mut chars = budget.chars.checked_sub(cap)?;
    let mut blocks = budget.blocks;
    let mut rows = Vec::new();
    for row in t.rows {
        let rc: usize = row.iter().map(|c| inline_chars(&c.content)).sum();
        if blocks == 0 || rc > chars {
            break;
        }
        blocks -= 1;
        chars -= rc;
        rows.push(row);
    }
    (!rows.is_empty()).then_some(Block::Table(Table { rows, ..t }))
}
