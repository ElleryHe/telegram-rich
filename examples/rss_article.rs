//! Normalize a messy RSS article HTML fragment into a `sendRichMessage` JSON payload.
//!
//! Run with:
//! ```sh
//! cargo run --example rss_article
//! ```

use telegram_rich::html::{ParseOptions, normalize};
use telegram_rich::{Block, FitOptions};

fn main() {
    let article_url = "https://blog.example.com/2026/11/rust-rich-messages/";
    let raw_html = r#"
        <div class="entry-content">
            <script>window.analytics && window.analytics.track('view');</script>
            <img src="https://tracker.example.com/open.gif" width="1" height="1" />
            <h2>Shipping Rich Messages from Rust</h2>
            <p>
                Telegram Bot API <strong>10.2–10.4</strong> introduced native rich
                documents. Check the <a href="/docs/guide">migration guide</a> for details:
                <img data-src="../assets/architecture.png" src="data:image/gif;base64,AAAA" />
            </p>
            <pre><code class="language-rust">let msg = InputRichMessage::html("&lt;p&gt;Hello!&lt;/p&gt;");</code></pre>
            <blockquote>
                <p>No external Instant View templates required.</p>
                <cite>FeedCourier Team</cite>
            </blockquote>
        </div>
    "#;

    let parse_opts = ParseOptions::with_base_url(article_url.parse().expect("valid URL"));
    let fit_opts = FitOptions {
        tail: vec![Block::link_button("Read original", article_url)],
        truncation_notice: vec![Block::paragraph(
            "Article exceeded Telegram limits and was truncated.",
        )],
        ..FitOptions::default()
    };

    let result = normalize(raw_html, &parse_opts, &fit_opts);
    println!("=== Stats ===");
    println!("{:?} (truncated = {})\n", result.stats, result.truncated);

    let payload = serde_json::to_string_pretty(&result.into_input_rich_message())
        .expect("serialize InputRichMessage");
    println!("=== sendRichMessage `rich_message` JSON ===");
    println!("{payload}");
}
