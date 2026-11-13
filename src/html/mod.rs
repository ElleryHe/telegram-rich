//! Normalizing arbitrary web HTML into Telegram Rich HTML.
//!
//! ```
//! use telegram_rich::html::{normalize, ParseOptions};
//! use telegram_rich::FitOptions;
//!
//! let opts = ParseOptions::with_base_url("https://example.com/post/1".parse().unwrap());
//! let out = normalize(
//!     r#"<div class="post"><h2>Hello</h2><p>See <a href="/about">this</a>.<img src="/a.png"></p><script>x()</script></div>"#,
//!     &opts,
//!     &FitOptions::default(),
//! );
//! assert_eq!(
//!     out.html,
//!     r#"<h2>Hello</h2><p>See <a href="https://example.com/about">this</a>.</p><img src="https://example.com/a.png"/>"#
//! );
//! assert!(!out.truncated);
//! ```

mod parse;

pub use parse::{ParseOptions, parse};

use crate::document::{Document, FitOptions};
use crate::limits::Stats;
use crate::types::InputRichMessage;

/// Result of [`normalize`].
#[derive(Debug, Clone, PartialEq)]
pub struct Normalized {
    /// The normalized and fitted document.
    pub document: Document,
    /// Rendered Telegram Rich HTML.
    pub html: String,
    /// Measured size of the final document.
    pub stats: Stats,
    /// Whether content had to be truncated to fit the limits.
    pub truncated: bool,
}

impl Normalized {
    /// Wraps the rendered HTML into an [`InputRichMessage`].
    pub fn into_input_rich_message(self) -> InputRichMessage {
        InputRichMessage::html(self.html)
    }
}

/// Parses `html`, fits it into `fit.limits` and renders Telegram Rich HTML.
pub fn normalize(html: &str, parse_opts: &ParseOptions, fit: &FitOptions) -> Normalized {
    let mut document = parse(html, parse_opts);
    let truncated = document.fit(fit);
    Normalized {
        html: document.to_html(),
        stats: document.stats(),
        document,
        truncated,
    }
}
