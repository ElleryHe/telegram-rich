#![cfg(feature = "html")]

use pretty_assertions::assert_eq;
use telegram_rich::FitOptions;
use telegram_rich::html::{ParseOptions, normalize};

fn base() -> ParseOptions {
    ParseOptions::with_base_url("https://example.com/blog/post.html".parse().unwrap())
}

fn html(input: &str) -> String {
    normalize(input, &base(), &FitOptions::default()).html
}

#[test]
fn inline_formatting_and_whitespace() {
    assert_eq!(
        html("<p>  Hello   <strong>bold</strong>\n<em>it</em> <del>x</del>  </p>"),
        "<p>Hello <b>bold</b> <i>it</i> <s>x</s></p>"
    );
}

#[test]
fn bare_text_becomes_paragraphs_split_by_blocks() {
    assert_eq!(
        html("intro<div>one</div><div>two</div>tail"),
        "<p>intro</p><p>one</p><p>two</p><p>tail</p>"
    );
}

#[test]
fn drops_scripts_styles_and_hidden() {
    assert_eq!(
        html(
            r#"<p>a<script>evil()</script><style>p{}</style><span hidden>h</span><span style="display: none">n</span>b</p><iframe src="x"></iframe>"#
        ),
        "<p>ab</p>"
    );
}

#[test]
fn resolves_relative_links_and_filters_schemes() {
    assert_eq!(
        html(
            r##"<p><a href="../about">rel</a> <a href="javascript:alert(1)">js</a> <a href="#top">anchor</a> <a href="mailto:a@b.c">mail</a></p>"##
        ),
        r#"<p><a href="https://example.com/about">rel</a> js anchor <a href="mailto:a@b.c">mail</a></p>"#
    );
}

#[test]
fn relative_urls_without_base_are_dropped() {
    let out = normalize(
        r#"<p><a href="/x">link</a><img src="/a.png"></p>"#,
        &ParseOptions::default(),
        &FitOptions::default(),
    );
    assert_eq!(out.html, "<p>link</p>");
}

#[test]
fn lifts_images_out_of_paragraphs_and_links() {
    assert_eq!(
        html(r#"<p>before <a href="/x"><b>bold <img src="img/a.jpg"> after</b></a> end</p>"#),
        concat!(
            r#"<p>before <a href="https://example.com/x"><b>bold</b></a></p>"#,
            r#"<img src="https://example.com/blog/img/a.jpg"/>"#,
            r#"<p><a href="https://example.com/x"><b>after</b></a> end</p>"#
        )
    );
}

#[test]
fn lazy_images_gifs_and_tracking_pixels() {
    assert_eq!(
        html(concat!(
            r#"<img src="data:image/gif;base64,AAAA" data-src="/lazy.jpg">"#,
            r#"<img srcset="/s.png 1x, /l.png 2x">"#,
            r#"<img src="/anim.GIF?v=1">"#,
            r#"<img src="https://t.example/pixel" width="1" height="1">"#
        )),
        concat!(
            r#"<img src="https://example.com/lazy.jpg"/>"#,
            r#"<img src="https://example.com/s.png"/>"#,
            r#"<video src="https://example.com/anim.GIF?v=1"></video>"#
        )
    );
}

#[test]
fn figure_with_caption_and_credit() {
    assert_eq!(
        html(
            r#"<figure><img src="/a.jpg"><figcaption>A <b>cat</b> <cite>Bob</cite></figcaption></figure>"#
        ),
        r#"<figure><img src="https://example.com/a.jpg"/><figcaption>A <b>cat</b><cite>Bob</cite></figcaption></figure>"#
    );
}

#[test]
fn video_and_audio_sources() {
    assert_eq!(
        html(
            r#"<video poster="/p.jpg"><source src="/v.mp4" type="video/mp4"></video><audio src="https://cdn.example/a.mp3"></audio>"#
        ),
        r#"<video src="https://example.com/v.mp4"></video><audio src="https://cdn.example/a.mp3"></audio>"#
    );
}

#[test]
fn headings_lists_and_checkboxes() {
    assert_eq!(
        html(
            r#"<h2>Title</h2><ol start="3"><li>one</li><li><p>two</p><ul><li><input type="checkbox" checked> done</li></ul></li></ol>"#
        ),
        concat!(
            "<h2>Title</h2>",
            r#"<ol start="3"><li>one</li><li><p>two</p><ul><li><input type="checkbox" checked>done</li></ul></li></ol>"#
        )
    );
}

#[test]
fn preformatted_code_keeps_whitespace_and_language() {
    assert_eq!(
        html(
            "<pre><code class=\"language-rust\">fn main() {\n    println!(\"&lt;hi&gt;\");\n}\n</code></pre><p>use <code>x &amp; y</code></p>"
        ),
        "<pre><code class=\"language-rust\">fn main() {\n    println!(\"&lt;hi&gt;\");\n}</code></pre><p>use <code>x &amp; y</code></p>"
    );
}

#[test]
fn blockquote_with_credit_and_expandable() {
    assert_eq!(
        html("<blockquote><p>Quoted</p><cite>Someone</cite></blockquote>"),
        "<blockquote>Quoted<cite>Someone</cite></blockquote>"
    );
    assert_eq!(
        html("<blockquote expandable><p>a</p><p>b</p></blockquote>"),
        "<blockquote expandable>a<br/>b</blockquote>"
    );
}

#[test]
fn data_table_and_layout_table() {
    assert_eq!(
        html(
            r#"<table><caption>Cap</caption><thead><tr><th>A</th><th align="right">B</th></tr></thead><tbody><tr><td colspan="2">x</td></tr></tbody></table>"#
        ),
        r#"<table bordered><caption>Cap</caption><tr><th>A</th><th align="right">B</th></tr><tr><td colspan="2">x</td></tr></table>"#
    );
    assert_eq!(
        html(
            r#"<table><tr><td><img src="/logo.png"></td><td><p>Newsletter body</p></td></tr></table>"#
        ),
        r#"<img src="https://example.com/logo.png"/><p>Newsletter body</p>"#
    );
}

#[test]
fn details_and_mathml() {
    assert_eq!(
        html(concat!(
            "<details open><summary>More</summary><p>hidden</p></details>",
            r#"<p>Energy <math><semantics><mi>E</mi><annotation encoding="application/x-tex">E=mc^2</annotation></semantics></math></p>"#
        )),
        "<details open><summary>More</summary><p>hidden</p></details><p>Energy <tg-math>E=mc^2</tg-math></p>"
    );
}

#[test]
fn escapes_text_and_attributes() {
    assert_eq!(
        html(r#"<p>1 &lt; 2 &amp; "q"</p><a href="https://e.com/?a=1&amp;b=&quot;2&quot;">l</a>"#),
        r#"<p>1 &lt; 2 &amp; "q"</p><p><a href="https://e.com/?a=1&amp;b=%222%22">l</a></p>"#
    );
}

#[test]
fn line_breaks_are_trimmed() {
    assert_eq!(
        html("<p><br>line one <br> line two<br><br></p>"),
        "<p>line one<br/>line two</p>"
    );
}

#[test]
fn empty_input() {
    let out = normalize(
        "   <div> </div> <!-- c -->",
        &base(),
        &FitOptions::default(),
    );
    assert_eq!(out.html, "");
    assert!(out.document.is_empty());
}
