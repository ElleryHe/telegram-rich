use pretty_assertions::assert_eq;
use telegram_rich::{
    Block, Document, FitOptions, Inline, Limits, ListItem, Media, MediaKind, Table, TableCell,
};

fn para(n: usize) -> Block {
    Block::paragraph("x".repeat(n))
}

fn photo(i: usize) -> Block {
    Block::Media(Media {
        kind: MediaKind::Photo,
        src: format!("https://e.com/{i}.jpg"),
        caption: None,
        credit: None,
        spoiler: false,
    })
}

#[test]
fn fits_without_changes_and_appends_tail() {
    let mut doc = Document::new(vec![para(10)]);
    let opts = FitOptions {
        tail: vec![Block::link_button("Open", "https://e.com")],
        truncation_notice: vec![Block::paragraph("cut")],
        ..FitOptions::default()
    };
    assert!(!doc.fit(&opts));
    assert_eq!(
        doc.to_html(),
        r#"<p>xxxxxxxxxx</p><tg-button-row><tg-button type="url" url="https://e.com">Open</tg-button></tg-button-row>"#
    );
}

#[test]
fn truncates_by_block_count_with_notice_and_tail() {
    let mut doc = Document::new((0..10).map(|_| para(1)).collect());
    let opts = FitOptions {
        limits: Limits {
            max_blocks: 5,
            ..Limits::TELEGRAM
        },
        tail: vec![Block::link_button("Open", "https://e.com")],
        truncation_notice: vec![Block::paragraph("cut")],
        ..FitOptions::default()
    };
    assert!(doc.fit(&opts));
    assert_eq!(doc.blocks.len(), 5); // 3 content + notice + tail
    assert_eq!(doc.blocks[3], Block::paragraph("cut"));
    assert!(doc.stats().fits(&opts.limits));
}

#[test]
fn cuts_long_paragraph_mid_text() {
    let mut doc = Document::new(vec![para(5), para(100)]);
    let opts = FitOptions {
        limits: Limits {
            max_chars: 20,
            ..Limits::TELEGRAM
        },
        ..FitOptions::default()
    };
    assert!(doc.fit(&opts));
    assert_eq!(
        doc.blocks[1],
        Block::Paragraph(vec![Inline::text("x".repeat(14)), Inline::text("…")])
    );
    assert_eq!(doc.stats().chars, 20);
}

#[test]
fn descends_into_lists() {
    let items = (0..10)
        .map(|_| ListItem {
            blocks: vec![para(1)],
            checkbox: None,
        })
        .collect();
    let mut doc = Document::new(vec![Block::List {
        ordered: false,
        start: None,
        items,
    }]);
    let opts = FitOptions {
        limits: Limits {
            max_blocks: 7, // list + 3 × (item + paragraph)
            ..Limits::TELEGRAM
        },
        ..FitOptions::default()
    };
    assert!(doc.fit(&opts));
    match &doc.blocks[0] {
        Block::List { items, .. } => assert_eq!(items.len(), 3),
        other => panic!("unexpected {other:?}"),
    }
    assert!(doc.stats().fits(&opts.limits));
}

#[test]
fn excess_media_become_links() {
    let mut doc = Document::new((0..3).map(photo).collect());
    let opts = FitOptions {
        limits: Limits {
            max_media: 2,
            ..Limits::TELEGRAM
        },
        ..FitOptions::default()
    };
    assert!(!doc.fit(&opts));
    assert_eq!(doc.stats().media, 2);
    assert_eq!(
        doc.blocks[2],
        Block::Paragraph(vec![Inline::Link {
            href: "https://e.com/2.jpg".into(),
            content: vec![Inline::text("https://e.com/2.jpg")]
        }])
    );
}

#[test]
fn deep_nesting_is_flattened() {
    let mut inl = vec![Inline::text("deep")];
    for _ in 0..30 {
        inl = vec![Inline::Bold(inl)];
    }
    let mut blocks = vec![Block::Paragraph(inl)];
    for _ in 0..30 {
        blocks = vec![Block::Quote {
            blocks,
            credit: None,
        }];
    }
    let mut doc = Document::new(blocks);
    doc.fit(&FitOptions::default());
    let stats = doc.stats();
    assert!(stats.depth <= 16, "depth {}", stats.depth);
    assert!(doc.to_html().contains("deep"));
}

#[test]
fn wide_tables_lose_extra_columns() {
    let row: Vec<TableCell> = (0..30)
        .map(|i| TableCell {
            content: vec![Inline::text(i.to_string())],
            ..TableCell::default()
        })
        .collect();
    let mut doc = Document::new(vec![Block::Table(Table {
        rows: vec![row],
        ..Table::default()
    })]);
    doc.fit(&FitOptions::default());
    assert_eq!(doc.stats().max_table_columns, 20);
}

#[test]
fn media_can_be_rewritten() {
    let mut doc = Document::new(vec![photo(1)]);
    doc.for_each_media_mut(|m| m.src = "tg://photo?id=p1".into());
    assert_eq!(doc.to_html(), r#"<img src="tg://photo?id=p1"/>"#);
    assert_eq!(doc.media().len(), 1);
}
