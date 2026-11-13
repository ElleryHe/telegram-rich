//! Serde types for outgoing Telegram rich messages (Bot API 10.4).
//!
//! These types mirror the official
//! [Bot API documentation](https://core.telegram.org/bots/api#rich-messages)
//! one-to-one and can be serialized directly into the `rich_message`
//! parameter of `sendRichMessage`.

mod blocks;
mod button;
mod common;
mod media;
mod message;
mod rich_text;

pub use blocks::{
    HorizontalAlign, InputRichBlock, InputRichBlockListItem, ListLabelType, RichBlockCaption,
    RichBlockTableCell, VerticalAlign,
};
pub use button::{ButtonStyle, RichMessageButton};
pub use common::{
    CopyTextButton, DisabledButton, Location, LoginUrl, SwitchInlineQueryChosenChat, User,
    WebAppInfo,
};
pub use media::{
    InputFileRef, InputMedia, InputMediaAnimation, InputMediaAudio, InputMediaDocument,
    InputMediaPhoto, InputMediaVideo, InputMediaVoiceNote,
};
pub use message::{InputRichMessage, InputRichMessageMedia};
pub use rich_text::{RichText, RichTextEntity};
