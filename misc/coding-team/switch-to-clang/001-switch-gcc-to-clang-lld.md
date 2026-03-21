# Task 001: Switch mise-lib rust toolchain from GCC to clang/lld

## Context

The shared mise-lib submodule (`.config/mise-lib/`) configures cross-compilation for Rust projects. It currently installs per-arch GCC cross-compilers and uses them as linkers via `CARGO_TARGET_<TRIPLE>_LINKER` env vars. We're replacing all of this with clang + lld.

Clang is a single compiler that can cross-compile to any target via `--target=<triple>`, eliminating the need for per-arch GCC packages. `lld` is the LLVM linker.

## Objective

Replace all GCC usage with clang/lld in the mise-lib rust toolchain scripts.

## Scope

Two files to modify, both in `.config/mise-lib/rust/`:

### 1. `setup`

**Replace GCC package installs** (lines 31-32):
```bash
# REMOVE these:
install_if_needed arm-linux-gnueabihf-gcc gcc-arm-linux-gnueabihf
install_if_needed aarch64-linux-gnu-gcc gcc-aarch64-linux-gnu
```

**Install instead:**
```bash
install_if_needed clang
install_if_needed lld
install_if_needed dpkg-architecture crossbuild-essential-armhf
install_if_needed dpkg-architecture crossbuild-essential-arm64
```

Note: `crossbuild-essential-armhf` and `crossbuild-essential-arm64` are meta-packages. They don't install a single obvious binary to check, so use a binary from one of their dependencies for the `install_if_needed` check (e.g. `arm-linux-gnueabihf-as` from `binutils-arm-linux-gnueabihf` for armhf, `aarch64-linux-gnu-as` from `binutils-aarch64-linux-gnu` for arm64). Or just use a different install pattern for these — your call on what's cleanest.

Keep `musl-tools` and `cmake` installs as-is.

### 2. `build-target`

**Replace the entire `case` block** (lines 34-51) that sets per-target GCC linkers.

New approach for ALL targets (including `x86_64-unknown-linux-gnu`):
- Set `CARGO_TARGET_<TRIPLE>_LINKER=clang`
- Append to `RUSTFLAGS`: `-C link-arg=--target=<triple> -C link-arg=-fuse-ld=lld`

The case statement should cover all 5 targets. For musl targets, clang still needs `--target` and `-fuse-ld=lld`. The `x86_64-unknown-linux-gnu` host target should also be set explicitly for consistency.

Example pattern for one target:
```bash
x86_64-unknown-linux-gnu)
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang
    RUSTFLAGS="${RUSTFLAGS} -C link-arg=--target=x86_64-unknown-linux-gnu -C link-arg=-fuse-ld=lld"
    ;;
```

Note: `RUSTFLAGS` is set to `"-C target-feature=+crt-static"` on line 53. The case block runs before that line, so either move the base RUSTFLAGS assignment before the case block, or initialize RUSTFLAGS before the case and append in both places. The key thing: the final RUSTFLAGS must contain both `-C target-feature=+crt-static` AND the per-target link args.

## Non-goals

- Do not change target triples (no gnu→musl migration).
- Do not modify `test`, `fmt`, `lint`, or any other scripts.
- Do not modify Dockerfiles.

## Constraints

- Keep `musl-tools` and `cmake` installs unchanged.
- Preserve the `install_if_needed` pattern used in `setup`.
- Preserve the binary renaming/copying logic in `build-target` (lines 59-73) — don't touch it.
- The `RUSTFLAGS` must still include `-C target-feature=+crt-static` for all targets.
