# Task 002: Fix cross-compilation sysroot packages and cosmetic issues

## Context

Review feedback identified that `binutils-arm-linux-gnueabihf` and `binutils-aarch64-linux-gnu` only provide assembler/linker tools, but NOT the C library headers and startup objects (crt1.o, crti.o, crtn.o, libc.so, etc.) that clang needs to link executables. The `crossbuild-essential-*` meta-packages pull in these via `libc6-dev-*-cross`.

## Objective

Fix the setup script to install the full cross-compilation sysroot, and fix a minor cosmetic issue in build-target.

## Scope

### 1. `.config/mise-lib/rust/setup`

Replace:
```bash
install_if_needed arm-linux-gnueabihf-as binutils-arm-linux-gnueabihf
install_if_needed aarch64-linux-gnu-as binutils-aarch64-linux-gnu
```

With:
```bash
install_if_needed arm-linux-gnueabihf-as crossbuild-essential-armhf
install_if_needed aarch64-linux-gnu-as crossbuild-essential-arm64
```

The binary check (`arm-linux-gnueabihf-as`, `aarch64-linux-gnu-as`) still works because `crossbuild-essential-*` depends on `binutils-*` which provides these binaries. The difference is that `crossbuild-essential-*` also pulls in `libc6-dev-*-cross` and `gcc-*` (as dependencies), giving clang the sysroot it needs.

### 2. `.config/mise-lib/rust/build-target`

Ensure consistent blank-line spacing between case branches. Each branch should have the same visual structure. Check the current file and add blank lines between branches if they're missing, to match the style of the rest of the file.

## Non-goals

- No other changes.
