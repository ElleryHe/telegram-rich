//! # telegram-rich
//!
//! Toolkit for Telegram Bot API **rich messages** (`sendRichMessage`, Bot API 10.2+,
//! types up to date with 10.4).
//!
//! - [`html`] *(feature `html`, on by default)*: turns arbitrary, messy web HTML
//!   (RSS items, articles, newsletters) into the HTML subset that Telegram rich
//!   messages accept, resolving relative URLs, lifting images out of paragraphs,
//!   dropping scripts and tracking pixels, and detecting layout tables.
//! - [`Document`]: an intermediate representation that you can also build by hand,
//!   measure against the official [`Limits`] (32 768 characters, 500 blocks,
//!   16 nesting levels, 50 media, 20 table columns), truncate at block boundaries
//!   with [`Document::fit`], and render with [`Document::to_html`].
//! - [`types`]: serde types for `InputRichMessage`, every `InputRichBlock`,
//!   `RichText` entity and `RichMessageButton`, for callers who prefer the
//!   explicit `blocks` representation.
//!
//! The crate performs no network I/O and does not depend on any Telegram client;
//! serialize [`types::InputRichMessage`] into the `rich_message` parameter with
//! the HTTP client of your choice.
//!
//! ```
//! # #[cfg(feature = "html")] {
//! use telegram_rich::{Block, FitOptions};
//! use telegram_rich::html::{normalize, ParseOptions};
//!
//! let fit = FitOptions {
//!     tail: vec![Block::link_button("Read original", "https://example.com/post/1")],
//!     truncation_notice: vec![Block::paragraph("The article was truncated.")],
//!     ..FitOptions::default()
//! };
//! let msg = normalize("<p>Hello <b>world</b></p>", &ParseOptions::default(), &fit)
//!     .into_input_rich_message();
//! let json = serde_json::to_string(&msg).unwrap();
//! assert!(json.starts_with(r#"{"html":"<p>Hello <b>world</b></p><tg-button-row>"#));
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod document;
#[cfg(feature = "html")]
#[cfg_attr(docsrs, doc(cfg(feature = "html")))]
pub mod html;
mod limits;
pub mod types;

pub use document::{
    Align, Block, ButtonStyle, Document, FitOptions, Inline, ListItem, Media, MediaKind, Table,
    TableCell, UrlButton, VAlign,
};
pub use limits::{Limits, Stats};
