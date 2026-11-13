# Security Policy

## Supported Versions

`telegram-rich` is currently in early `0.0.x` development; only the latest
released version receives security fixes.

| Version | Supported          |
| ------- | ------------------ |
| 0.0.x   | :white_check_mark: |

## Scope

`telegram-rich` parses untrusted, arbitrary HTML (such as RSS feed entries and
scraped web pages) and renders a restricted HTML subset for the Telegram Bot API.
Security-relevant issues include:

- Improper HTML text or attribute escaping leading to tag/attribute injection in
  rendered output.
- Dangerous URI schemes (such as `javascript:`, `data:`, or `vbscript:`) passing
  through link or media URL resolution.
- Inputs causing panics, unbounded memory allocation, or algorithmic complexity
  denial-of-service (DoS) during parsing, measurement, or truncation.

## Reporting a Vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.

Instead, report vulnerabilities privately through
[GitHub Private Vulnerability Reporting](https://github.com/ElleryHe/telegram-rich/security/advisories/new)
with a minimal reproducing HTML snippet and expected vs. actual behavior. You
can expect an initial response within 7 days.
