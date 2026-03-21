# Task 006: Update bootdns mise.toml for musl targets

## Context

The project's `mise.toml` references gnu target triples in its `outputs` globs. These need to match the new musl targets.

## Objective

Update all target triple references in `mise.toml` from gnu to musl.

## Scope

`mise.toml` in the project root. Update the `outputs` arrays:

- `release-build` outputs: change `aarch64-unknown-linux-gnu` → `aarch64-unknown-linux-musl`, `armv7-unknown-linux-gnueabihf` → `armv7-unknown-linux-musleabihf`, `x86_64-unknown-linux-gnu` → `x86_64-unknown-linux-musl`.
- `build` outputs: change `x86_64-unknown-linux-gnu` → `x86_64-unknown-linux-musl`.

## Non-goals
- No other changes to mise.toml.
