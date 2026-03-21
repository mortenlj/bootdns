# Task 003: Fix pre-existing formatting issue

## Context

`cargo fmt --check` fails due to a formatting issue in `src/domeneshop.rs` around line 231. This blocks `mise run ci`.

## Objective

Run `cargo fmt --all` to fix the formatting, then verify with `cargo fmt --all --check`.

## Scope

- `src/domeneshop.rs` — the only file with a formatting issue.

## Non-goals

- No logic changes. Only whitespace/formatting.
