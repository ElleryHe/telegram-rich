# telegram-rich

[![crates.io](https://img.shields.io/crates/v/telegram-rich.svg)](https://crates.io/crates/telegram-rich)
[![docs.rs](https://docs.rs/telegram-rich/badge.svg)](https://docs.rs/telegram-rich)
[![CI](https://github.com/ElleryHe/telegram-rich/actions/workflows/ci.yml/badge.svg)](https://github.com/ElleryHe/telegram-rich/actions/workflows/ci.yml)
![license](https://img.shields.io/crates/l/telegram-rich.svg)

Rust toolkit for Telegram Bot API **rich messages** (`sendRichMessage`, introduced in
Bot API 10.2; types up to date with **Bot API 10.4**).

Telegram rich messages render headings, lists, tables, block quotes, collapsible
details, formulas, media and buttons natively — but they only accept a specific
HTML subset and enforce hard limits. Feeding them raw web HTML from RSS items or
articles fails or renders badly. `telegram-rich` bridges that gap:

- **HTML normalizer** — turns arbitrary, messy web HTML into Telegram Rich HTML:
  resolves relative URLs against a base URL, lifts images/videos out of paragraphs
  and links into standalone media blocks, handles lazy-loaded images
  (`data-src`, `srcset`), drops scripts/iframes/forms/hidden elements and 1×1
  tracking pixels, unwraps layout tables, maps `<figure>/<figcaption>/<cite>` to
  captioned media, preserves code languages, and extracts TeX from MathML.
- **Limit-aware truncation** — measures documents against the official limits
  (32 768 characters, 500 blocks, 16 nesting levels, 50 media, 20 table columns)
  and cuts at block boundaries, descending into lists, quotes, details and tables
  and cutting text mid-paragraph only as a last resort. Reserve room for a
  "Read original" button row and an optional truncation notice.
- **Typed document model** — build documents by hand, rewrite media URLs
  (e.g. to `tg://photo?id=…` for explicit uploads) and render to Rich HTML.
- **Serde types** — `InputRichMessage`, `InputRichMessageMedia`, every
  `InputRichBlock`, every `RichText` entity and `RichMessageButton`, mirroring the
  official documentation one-to-one, for callers who prefer explicit blocks.

No network I/O, no Telegram client dependency — plug the result into the HTTP
client or bot framework of your choice.

## Usage

```toml
[dependencies]
telegram-rich = "0.0.1"
```

```rust
use telegram_rich::{Block, FitOptions};
use telegram_rich::html::{normalize, ParseOptions};

let article_url = "https://example.com/post/1";
let parse = ParseOptions::with_base_url(article_url.parse().unwrap());
let fit = FitOptions {
    tail: vec![Block::link_button("Read original", article_url)],
    truncation_notice: vec![Block::paragraph("The article was truncated.")],
    ..FitOptions::default()
};

let out = normalize(r#"<article><h2>Title</h2><p>Body <img src="/a.png"></p></article>"#, &parse, &fit);
assert!(out.stats.fits(&fit.limits));

// `rich_message` parameter of sendRichMessage:
let rich_message = serde_json::to_value(out.into_input_rich_message()).unwrap();
```

Building blocks explicitly with the serde types:

```rust
use telegram_rich::types::{InputRichBlock, InputRichMessage, RichText, RichTextEntity};

let msg = InputRichMessage::blocks(vec![
    InputRichBlock::Heading { text: "Release notes".into(), size: 2 },
    InputRichBlock::Paragraph {
        text: RichText::Concat(vec![
            "Now with ".into(),
            RichTextEntity::Bold { text: "rich messages".into() }.into(),
        ]),
    },
]);
```

## Cargo features

| Feature | Default | Description |
|---|---|---|
| `html` | ✓ | HTML normalizer (`telegram_rich::html`), pulls in `scraper` and `url`. Disable it to use only the types and the document model. |

## Status

Early development (`0.0.x`): the API may change between releases. Feedback and
real-world HTML samples that render poorly are very welcome.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
