# Task 008: Fix test script to use musl target

## Context

The `rust/test` script uses `rustup show | grep "Default host"` to determine the test target, which returns `x86_64-unknown-linux-gnu` (the host triple). After switching to musl targets, the gnu target won't be installed in fresh CI environments, and tests should run against the musl target anyway.

## Objective

Update the test script to use `x86_64-unknown-linux-musl` as the default target, matching the change made in `build-target`.

## Scope

`.config/mise-lib/rust/test` — Replace the dynamic host detection with the hardcoded musl default, same as in `build-target`:

Change:
```bash
default_target=$(rustup show | grep "Default host" | awk '{ print $3 }')
```

To:
```bash
default_target="x86_64-unknown-linux-musl"
```

## Non-goals
- No other changes to the test script.
