//! [`InputRichMessage`] and [`InputRichMessageMedia`].

use serde::{Deserialize, Serialize};

use super::blocks::InputRichBlock;
use super::media::InputMedia;

/// Describes a rich message to be sent.
///
/// Exactly one of `html`, `markdown` or `blocks` must be set; use the
/// constructors to guarantee this.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputRichMessage {
    /// Optional. Content described as a list of blocks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<InputRichBlock>>,
    /// Optional. Content described using Rich HTML formatting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    /// Optional. Content described using Rich Markdown formatting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,
    /// Optional. Media referenced from `html`/`markdown` via `tg://photo?id=`,
    /// `tg://video?id=`, `tg://document?id=` or `tg://audio?id=` links.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub media: Vec<InputRichMessageMedia>,
    /// Optional. Show the message right-to-left.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_rtl: Option<bool>,
    /// Optional. Skip automatic detection of URLs, mentions, hashtags, etc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_entity_detection: Option<bool>,
}

impl InputRichMessage {
    fn empty() -> Self {
        Self {
            blocks: None,
            html: None,
            markdown: None,
            media: Vec::new(),
            is_rtl: None,
            skip_entity_detection: None,
        }
    }

    /// Creates a message from Rich HTML.
    pub fn html(html: impl Into<String>) -> Self {
        Self {
            html: Some(html.into()),
            ..Self::empty()
        }
    }

    /// Creates a message from Rich Markdown.
    pub fn markdown(markdown: impl Into<String>) -> Self {
        Self {
            markdown: Some(markdown.into()),
            ..Self::empty()
        }
    }

    /// Creates a message from explicit blocks.
    pub fn blocks(blocks: Vec<InputRichBlock>) -> Self {
        Self {
            blocks: Some(blocks),
            ..Self::empty()
        }
    }

    /// Adds an explicitly specified media element (HTML/Markdown modes only).
    pub fn with_media(mut self, media: InputRichMessageMedia) -> Self {
        self.media.push(media);
        self
    }

    /// Sets `skip_entity_detection`.
    pub fn with_skip_entity_detection(mut self, skip: bool) -> Self {
        self.skip_entity_detection = Some(skip);
        self
    }

    /// Sets `is_rtl`.
    pub fn with_rtl(mut self, rtl: bool) -> Self {
        self.is_rtl = Some(rtl);
        self
    }
}

/// A media element embedded in an outgoing rich message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputRichMessageMedia {
    /// Identifier used in `tg://…?id=` links; 1-64 characters of `A-Za-z0-9_-`.
    pub id: String,
    /// The media to be sent; only the media itself and its properties are used.
    pub media: InputMedia,
}

impl InputRichMessageMedia {
    /// Creates a media element.
    pub fn new(id: impl Into<String>, media: InputMedia) -> Self {
        Self {
            id: id.into(),
            media,
        }
    }

    /// Returns `true` if `id` satisfies the Bot API constraints.
    pub fn is_valid_id(id: &str) -> bool {
        (1..=64).contains(&id.len())
            && id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    }
}
