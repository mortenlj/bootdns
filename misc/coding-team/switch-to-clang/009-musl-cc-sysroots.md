# Task 009: Switch to musl.cc toolchains for cross-compilation sysroots

## Context

The multiarch `musl-dev:arm64` / `musl-dev:armhf` approach doesn't work reliably on Ubuntu. The standard solution for musl cross-compilation sysroots is pre-built toolchains from musl.cc.

We only need the **sysroot** (headers + static libs) from these toolchains, not the GCC compiler (we use clang+lld).

## Objective

Replace the multiarch musl-dev package installation with musl.cc toolchain downloads. Update CFLAGS to point at the correct sysroot paths.

## Scope — 2 files in `.config/mise-lib/rust/`

### 1. `rust/setup`

**Remove entirely:**
- The `install_file_if_needed` function
- The multiarch `dpkg --add-architecture` blocks
- The `install_file_if_needed` calls for `musl-dev:armhf` and `musl-dev:arm64`

**Add instead:** Download and extract musl.cc cross toolchains. Use a simple idempotent pattern — check if the directory exists, download if not:

```bash
# Download musl.cc cross-compilation sysroots
if [[ ! -d /opt/aarch64-linux-musl-cross ]]; then
  curl -fsSL https://musl.cc/aarch64-linux-musl-cross.tgz | ${SUDO} tar xz -C /opt/
fi
if [[ ! -d /opt/armv7l-linux-musleabihf-cross ]]; then
  curl -fsSL https://musl.cc/armv7l-linux-musleabihf-cross.tgz | ${SUDO} tar xz -C /opt/
fi
```

Keep everything else unchanged (`clang`, `lld`, `cmake`, `musl-tools`, rustup targets, etc.).

### 2. `rust/build-target`

**Update `CFLAGS_<target>`** for the cross targets to use `--sysroot` pointing at the musl.cc sysroot:

- `armv7-unknown-linux-musleabihf`: `--target=armv7-unknown-linux-musleabihf --sysroot=/opt/armv7l-linux-musleabihf-cross/armv7l-linux-musleabihf`
- `aarch64-unknown-linux-musl`: `--target=aarch64-unknown-linux-musl --sysroot=/opt/aarch64-linux-musl-cross/aarch64-linux-musl`

For `x86_64-unknown-linux-musl` (native), keep using `-isystem /usr/include/x86_64-linux-musl` since `musl-tools` provides the native musl headers.

Also update the RUSTFLAGS link args for cross targets to include `--sysroot` so lld can find the musl libc:
- Add `-C link-arg=--sysroot=/opt/<toolchain>/<triple>` to the RUSTFLAGS for armv7 and aarch64 targets.

## Non-goals
- No changes to `virtual_tasks.toml`, `rust-builder.Dockerfile`, `mise.toml`, `rust/test`, or any other files.

## Caveats
- The musl.cc tarball directory names use `armv7l-linux-musleabihf` (with `l` suffix), while the Rust target triple is `armv7-unknown-linux-musleabihf` (no `l`). The sysroot path inside the tarball is `/opt/armv7l-linux-musleabihf-cross/armv7l-linux-musleabihf/`.
- The `${SUDO}` variable is already defined in the setup script for commands that need root.
