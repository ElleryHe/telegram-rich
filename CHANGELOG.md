# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.0.2] - 2026-11-22

### Added

- `Block::Collage` (`<tg-collage>`) and `Block::Slideshow` (`<tg-slideshow>`) in
  the `Document` model, with full support in `to_html`, `stats`, `fit`,
  `media`, and `for_each_media_mut`.
- `ParseOptions::heading_offset` (default `0`) to demote parsed HTML headings
  (`h1`–`h6`, clamped to `h6`) when prepending an article title header.
- `ParseOptions::group_consecutive_media` (default `false`) to automatically
  merge runs of 2 or more consecutive visual media items (photos, videos,
  animations) into `<tg-collage>` (up to `MAX_COLLAGE_MEDIA = 10` items) or
  `<tg-slideshow>` when exceeding `MAX_COLLAGE_MEDIA`.

## [0.0.1] - 2026-11-13

### Added

- Serde types for Bot API 10.4 rich messages: `InputRichMessage`,
  `InputRichMessageMedia`, `InputRichBlock` (all block kinds), `RichText` /
  `RichTextEntity` (all entity kinds including `ton_address`),
  `RichMessageButton` and input media types.
- `Document` intermediate representation with Telegram Rich HTML rendering,
  measurement (`Stats`) against official `Limits`, and block-boundary truncation
  (`Document::fit`) with tail blocks and truncation notice.
- `html` feature: normalizer from arbitrary web HTML to `Document`.
