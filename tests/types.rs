use pretty_assertions::assert_eq;
use serde_json::json;
use telegram_rich::types::*;

#[test]
fn html_message_serializes_minimal() {
    let msg = InputRichMessage::html("<p>x</p>").with_skip_entity_detection(true);
    assert_eq!(
        serde_json::to_value(&msg).unwrap(),
        json!({"html": "<p>x</p>", "skip_entity_detection": true})
    );
}

#[test]
fn media_carry_their_type_tag() {
    let msg = InputRichMessage::html(r#"<img src="tg://photo?id=p1"/>"#).with_media(
        InputRichMessageMedia::new("p1", InputMedia::Photo(InputMediaPhoto::new("attach://p1"))),
    );
    let v = serde_json::to_value(&msg).unwrap();
    assert_eq!(
        v["media"],
        json!([{"id": "p1", "media": {"type": "photo", "media": "attach://p1"}}])
    );
    let back: InputRichMessage = serde_json::from_value(v).unwrap();
    assert_eq!(back, msg);
    assert!(InputRichMessageMedia::is_valid_id("p_1-A"));
    assert!(!InputRichMessageMedia::is_valid_id("bad id"));
}

#[test]
fn blocks_and_rich_text_serialize_like_the_docs() {
    let blocks = vec![
        InputRichBlock::Heading {
            text: "Title".into(),
            size: 1,
        },
        InputRichBlock::Paragraph {
            text: RichText::Concat(vec![
                "Hello ".into(),
                RichTextEntity::Bold {
                    text: RichTextEntity::Url {
                        text: "world".into(),
                        url: "https://t.me".into(),
                    }
                    .into(),
                }
                .into(),
            ]),
        },
        InputRichBlock::List {
            items: vec![InputRichBlockListItem {
                label_type: Some(ListLabelType::LowerRoman),
                ..InputRichBlockListItem::new(vec![InputRichBlock::Divider])
            }],
        },
        InputRichBlock::ExpandableBlockQuotation {
            text: "long".into(),
            credit: None,
        },
        InputRichBlock::Photo {
            photo: InputMediaPhoto::new("https://e.com/a.jpg"),
            caption: Some(RichBlockCaption {
                text: "cap".into(),
                credit: Some("me".into()),
            }),
        },
        InputRichBlock::Buttons {
            buttons: vec![
                RichMessageButton::url("Open", "https://e.com").with_style(ButtonStyle::Primary),
            ],
            align: Some(HorizontalAlign::Center),
        },
        InputRichBlock::Table {
            cells: vec![vec![RichBlockTableCell {
                text: Some("c".into()),
                is_header: Some(true),
                colspan: None,
                rowspan: None,
                align: HorizontalAlign::Left,
                valign: VerticalAlign::Top,
            }]],
            is_bordered: None,
            is_striped: None,
            is_compact: Some(true),
            caption: None,
        },
    ];
    let v = serde_json::to_value(InputRichMessage::blocks(blocks.clone())).unwrap();
    assert_eq!(
        v,
        json!({"blocks": [
            {"type": "heading", "text": "Title", "size": 1},
            {"type": "paragraph", "text": ["Hello ", {"type": "bold", "text": {"type": "url", "text": "world", "url": "https://t.me"}}]},
            {"type": "list", "items": [{"blocks": [{"type": "divider"}], "type": "i"}]},
            {"type": "expandable_blockquote", "text": "long"},
            {"type": "photo", "photo": {"type": "photo", "media": "https://e.com/a.jpg"}, "caption": {"text": "cap", "credit": "me"}},
            {"type": "buttons", "buttons": [{"text": "Open", "style": "primary", "url": "https://e.com"}], "align": "center"},
            {"type": "table", "cells": [[{"text": "c", "is_header": true, "align": "left", "valign": "top"}]], "is_compact": true}
        ]})
    );
    let back: InputRichMessage = serde_json::from_value(v).unwrap();
    assert_eq!(back.blocks.unwrap(), blocks);
}

#[test]
fn ton_address_entity_from_10_4() {
    let e: RichText =
        serde_json::from_value(json!({"type": "ton_address", "text": "addr", "address": "EQ..."}))
            .unwrap();
    assert_eq!(
        e,
        RichTextEntity::TonAddress {
            text: "addr".into(),
            address: "EQ...".into()
        }
        .into()
    );
}
