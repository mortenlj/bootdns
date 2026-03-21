# Task 005: Fix update_needed flag for multiarch musl-dev install

## Context

Review found a bug: if multiarch architectures (armhf, arm64) are already enabled from a previous run but `musl-dev:armhf`/`musl-dev:arm64` are not installed, the `install_file_if_needed` function skips `apt-get update` because `update_needed` was never set to `"yes"`. This causes `apt-get install` to fail with stale package lists.

## Objective

Ensure `apt-get update` runs before attempting to install multiarch musl-dev packages.

## Scope

`.config/mise-lib/rust/setup` — Set `update_needed="yes"` unconditionally before the multiarch block (before the `dpkg --print-foreign-architectures` checks). This ensures the first `install_file_if_needed` call triggers `apt-get update`.

Change the multiarch section from:
```bash
# Enable multiarch for musl cross-compilation
if ! dpkg --print-foreign-architectures | grep -q armhf; then
  ${SUDO} dpkg --add-architecture armhf
  update_needed="yes"
fi
if ! dpkg --print-foreign-architectures | grep -q arm64; then
  ${SUDO} dpkg --add-architecture arm64
  update_needed="yes"
fi
```

To:
```bash
# Enable multiarch for musl cross-compilation
update_needed="yes"
if ! dpkg --print-foreign-architectures | grep -q armhf; then
  ${SUDO} dpkg --add-architecture armhf
fi
if ! dpkg --print-foreign-architectures | grep -q arm64; then
  ${SUDO} dpkg --add-architecture arm64
fi
```

## Non-goals
- No other changes.
