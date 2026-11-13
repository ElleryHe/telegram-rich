//! Official rich message limits.

/// Limits enforced by Telegram on rich messages.
///
/// [`Limits::TELEGRAM`] holds the official values documented in
/// [Rich Message Limits](https://core.telegram.org/bots/api#rich-message-formatting-options).
/// Use stricter values to keep a safety margin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Maximum number of characters of text, including formula sources.
    pub max_chars: usize,
    /// Maximum number of blocks, including nested blocks, list items, table
    /// rows, quotation blocks and details blocks.
    pub max_blocks: usize,
    /// Maximum nesting depth of formatting and blocks.
    pub max_depth: usize,
    /// Maximum number of media attachments.
    pub max_media: usize,
    /// Maximum number of columns in a table.
    pub max_table_columns: usize,
}

impl Limits {
    /// Official limits as of Bot API 10.4.
    pub const TELEGRAM: Limits = Limits {
        max_chars: 32_768,
        max_blocks: 500,
        max_depth: 16,
        max_media: 50,
        max_table_columns: 20,
    };
}

impl Default for Limits {
    fn default() -> Self {
        Self::TELEGRAM
    }
}

/// Measured size of a document, comparable against [`Limits`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    /// Number of text characters.
    pub chars: usize,
    /// Number of blocks.
    pub blocks: usize,
    /// Maximum nesting depth.
    pub depth: usize,
    /// Number of media attachments.
    pub media: usize,
    /// Maximum number of columns in any table.
    pub max_table_columns: usize,
}

impl Stats {
    /// Returns `true` if all values are within `limits`.
    pub fn fits(&self, limits: &Limits) -> bool {
        self.chars <= limits.max_chars
            && self.blocks <= limits.max_blocks
            && self.depth <= limits.max_depth
            && self.media <= limits.max_media
            && self.max_table_columns <= limits.max_table_columns
    }
}
