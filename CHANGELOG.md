# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

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
