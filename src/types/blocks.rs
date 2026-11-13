//! Outgoing rich message blocks ([`InputRichBlock`]).

use serde::{Deserialize, Serialize};

use super::button::RichMessageButton;
use super::common::Location;
use super::media::{
    InputMediaAnimation, InputMediaAudio, InputMediaDocument, InputMediaPhoto, InputMediaVideo,
    InputMediaVoiceNote,
};
use super::rich_text::RichText;

/// Horizontal alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HorizontalAlign {
    /// Left.
    Left,
    /// Center.
    Center,
    /// Right.
    Right,
}

/// Vertical alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerticalAlign {
    /// Top.
    Top,
    /// Middle.
    Middle,
    /// Bottom.
    Bottom,
}

/// Label type of an ordered list item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListLabelType {
    /// Lowercase letters.
    #[serde(rename = "a")]
    LowerAlpha,
    /// Uppercase letters.
    #[serde(rename = "A")]
    UpperAlpha,
    /// Lowercase Roman numerals.
    #[serde(rename = "i")]
    LowerRoman,
    /// Uppercase Roman numerals.
    #[serde(rename = "I")]
    UpperRoman,
    /// Decimal numbers.
    #[serde(rename = "1")]
    Decimal,
}

/// Caption of a rich formatted block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RichBlockCaption {
    /// Block caption.
    pub text: RichText,
    /// Optional. Block credit (`<cite>`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credit: Option<RichText>,
}

/// A cell in a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RichBlockTableCell {
    /// Optional. Text in the cell; omitted means the cell is invisible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<RichText>,
    /// Optional. True, if the cell is a header cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_header: Option<bool>,
    /// Optional. Number of columns the cell spans, if bigger than 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colspan: Option<u32>,
    /// Optional. Number of rows the cell spans, if bigger than 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rowspan: Option<u32>,
    /// Horizontal cell content alignment.
    pub align: HorizontalAlign,
    /// Vertical cell content alignment.
    pub valign: VerticalAlign,
}

/// An item of a list to be sent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputRichBlockListItem {
    /// The content of the item.
    pub blocks: Vec<InputRichBlock>,
    /// Optional. The item has a checkbox.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_checkbox: Option<bool>,
    /// Optional. The item has a checked checkbox.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_checked: Option<bool>,
    /// Optional. For ordered lists, the numeric value of the item label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    /// Optional. For ordered lists, the type of the item label.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub label_type: Option<ListLabelType>,
}

impl InputRichBlockListItem {
    /// Creates a list item with the given content.
    pub fn new(blocks: Vec<InputRichBlock>) -> Self {
        Self {
            blocks,
            has_checkbox: None,
            is_checked: None,
            value: None,
            label_type: None,
        }
    }
}

/// A block in a rich formatted message to be sent, tagged by the `type` field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputRichBlock {
    /// A text paragraph (`<p>`).
    Paragraph {
        /// Text of the block.
        text: RichText,
    },
    /// A section heading (`<h1>`–`<h6>`).
    Heading {
        /// Text of the block.
        text: RichText,
        /// Relative font size; 1-6, 1 is the largest.
        size: u8,
    },
    /// A preformatted text block (`<pre><code>`).
    Pre {
        /// Text of the block.
        text: RichText,
        /// Optional. The programming language of the text.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        language: Option<String>,
    },
    /// A footer (`<footer>`).
    Footer {
        /// Text of the block.
        text: RichText,
    },
    /// A divider (`<hr/>`).
    Divider,
    /// A block formula (`<tg-math-block>`).
    MathematicalExpression {
        /// The expression in LaTeX format.
        expression: String,
    },
    /// An anchor (`<a name="…">`).
    Anchor {
        /// The name of the anchor.
        name: String,
    },
    /// A list (`<ul>` / `<ol>`).
    List {
        /// Items of the list.
        items: Vec<InputRichBlockListItem>,
    },
    /// A block quotation (`<blockquote>`).
    #[serde(rename = "blockquote")]
    BlockQuotation {
        /// Content of the block.
        blocks: Vec<InputRichBlock>,
        /// Optional. Credit of the block.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        credit: Option<RichText>,
    },
    /// An expandable block quotation (`<blockquote expandable>`, Bot API 10.3).
    #[serde(rename = "expandable_blockquote")]
    ExpandableBlockQuotation {
        /// Content of the block.
        text: RichText,
        /// Optional. Credit of the block.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        credit: Option<RichText>,
    },
    /// A quotation with centered text (`<aside>`).
    #[serde(rename = "pullquote")]
    PullQuotation {
        /// Text of the block.
        text: RichText,
        /// Optional. Credit of the block.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        credit: Option<RichText>,
    },
    /// A collage (`<tg-collage>`).
    Collage {
        /// Elements of the collage.
        blocks: Vec<InputRichBlock>,
        /// Optional. Caption of the block.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    /// A slideshow (`<tg-slideshow>`).
    Slideshow {
        /// Elements of the slideshow.
        blocks: Vec<InputRichBlock>,
        /// Optional. Caption of the block.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    /// A table (`<table>`).
    Table {
        /// Cells of the table.
        cells: Vec<Vec<RichBlockTableCell>>,
        /// Optional. The table has borders.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_bordered: Option<bool>,
        /// Optional. The table is striped.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_striped: Option<bool>,
        /// Optional. Cells have smaller indents (Bot API 10.3).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_compact: Option<bool>,
        /// Optional. Caption of the table.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichText>,
    },
    /// An expandable details block (`<details>`).
    Details {
        /// Always shown summary of the block.
        summary: RichText,
        /// Content of the block.
        blocks: Vec<InputRichBlock>,
        /// Optional. The content is visible by default.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_open: Option<bool>,
    },
    /// A map (`<tg-map>`).
    Map {
        /// Location of the center of the map.
        location: Location,
        /// Optional. Zoom level; 0-24.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zoom: Option<u8>,
        /// Optional. Width; 0-10000.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        width: Option<u32>,
        /// Optional. Height; 0-10000.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        height: Option<u32>,
        /// Optional. Caption of the block.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    /// A row of 1-8 buttons (`<tg-button-row>`, Bot API 10.3).
    Buttons {
        /// List of 1-8 buttons.
        buttons: Vec<RichMessageButton>,
        /// Optional. Horizontal alignment.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        align: Option<HorizontalAlign>,
    },
    /// An animation (`<video>`).
    Animation {
        /// The animation; caption is ignored.
        animation: InputMediaAnimation,
        /// Optional. Caption of the block.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    /// A music file (`<audio>`).
    Audio {
        /// The audio; caption is ignored.
        audio: InputMediaAudio,
        /// Optional. Caption of the block.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    /// A general file (`<tg-document>`, Bot API 10.3).
    Document {
        /// The document; caption is ignored.
        document: InputMediaDocument,
        /// Optional. Caption of the block.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    /// A photo (`<img>`).
    Photo {
        /// The photo; caption is ignored.
        photo: InputMediaPhoto,
        /// Optional. Caption of the block.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    /// A video (`<video>`).
    Video {
        /// The video; caption is ignored.
        video: InputMediaVideo,
        /// Optional. Caption of the block.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    /// A voice note (`<audio>`).
    VoiceNote {
        /// The voice note; caption is ignored.
        voice_note: InputMediaVoiceNote,
        /// Optional. Caption of the block.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    /// A “Thinking…” placeholder; only valid in `sendRichMessageDraft`.
    Thinking {
        /// Text of the block.
        text: RichText,
    },
}
