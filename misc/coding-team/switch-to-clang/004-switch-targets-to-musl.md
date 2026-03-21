# Task 004: Switch mise-lib from gnu targets to musl targets

## Context

The project cross-compiles Rust binaries for 3 architectures. Currently it targets glibc (`*-gnu*`), but static linking with glibc is not properly supported. We're switching to musl targets which support static linking correctly.

The project has a C dependency (`aws-lc-sys` via reqwest → rustls) that compiles C code using the `cc` crate and `cmake`. The `cc` crate looks for `CC_<target>` environment variables to find the C compiler for cross-compilation.

Rust bundles its own copy of musl libc for musl targets when statically linking, but the C code compiled by `aws-lc-sys` needs musl headers from the system.

The previous change already switched the linker from GCC to clang+lld. This task changes the target triples from gnu to musl.

## Objective

Replace all 3 release targets with their musl equivalents. Remove the 2 previously-unused musl targets (they're now the primary ones). Total targets go from 5 to 3.

Target mapping:
- `armv7-unknown-linux-gnueabihf` → `armv7-unknown-linux-musleabihf`
- `aarch64-unknown-linux-gnu` → `aarch64-unknown-linux-musl`
- `x86_64-unknown-linux-gnu` → `x86_64-unknown-linux-musl`

## Scope — 4 files in `.config/mise-lib/`

### 1. `rust/setup`

**Replace cross sysroot packages.** Remove `crossbuild-essential-armhf` and `crossbuild-essential-arm64`. Install musl-dev cross packages instead.

For musl cross-compilation sysroots on Debian, you need to enable multiarch and install `musl-dev` for each foreign architecture:
```bash
dpkg --add-architecture armhf
dpkg --add-architecture arm64
# then apt-get update, then:
# musl-dev:armhf and musl-dev:arm64
```

Adapt the `install_if_needed` pattern for these. The binary check won't work for multiarch packages (they don't install binaries to PATH). Instead, check for the existence of a known file, e.g.:
- `/usr/lib/arm-linux-musleabihf/libc.a` for armhf musl-dev
- `/usr/lib/aarch64-linux-musl/libc.a` for arm64 musl-dev

Or use `dpkg -s musl-dev:armhf` to check if installed. Pick whatever is cleanest.

For native x86_64 musl, `musl-tools` (already installed) provides `musl-dev` for the host arch.

**Update `rustup target add`** — change to the 3 musl targets only:
```
armv7-unknown-linux-musleabihf x86_64-unknown-linux-musl aarch64-unknown-linux-musl
```

### 2. `rust/build-target`

**Update the `# [USAGE]` choices** on line 5 to list only the 3 musl targets.

**Update the `case` block** — 3 branches instead of 5, using musl triples. Each branch needs:
- `CARGO_TARGET_<TRIPLE>_LINKER=clang` (note: the env var name uses the uppercased, hyphen-to-underscore triple)
- Append to RUSTFLAGS: `-C link-arg=--target=<triple> -C link-arg=-fuse-ld=lld`
- Set `CC_<target>=clang` and `CFLAGS_<target>=--target=<triple>` for the `cc` crate (the `cc` crate uses the literal target triple with hyphens replaced by underscores as the env var suffix)

The env var names for the `cc` crate use underscores, e.g.:
- `CC_armv7_unknown_linux_musleabihf=clang`
- `CFLAGS_armv7_unknown_linux_musleabihf=--target=armv7-unknown-linux-musleabihf`

**Important:** The default host target detection (`rustup show | grep "Default host"`) will return `x86_64-unknown-linux-gnu` on the build machine. But we want the default to be `x86_64-unknown-linux-musl` now. Change the default_target fallback logic: instead of using the host triple from rustup, hardcode `x86_64-unknown-linux-musl` as the default, or detect the host arch and append `-unknown-linux-musl`. The simplest approach: just hardcode `x86_64-unknown-linux-musl` as the default since this always runs on x86_64 Linux.

### 3. `virtual_tasks.toml`

Update the `rust:release-build` depends list to use the 3 musl targets:
```toml
["rust:release-build"]
depends = [
    "rust:build-target --release armv7-unknown-linux-musleabihf",
    "rust:build-target --release x86_64-unknown-linux-musl",
    "rust:build-target --release aarch64-unknown-linux-musl",
]
```

### 4. `baseimages/rust-builder.Dockerfile`

Update the `cp` commands on lines 24-26 to use musl target triples:
```
cp -v /app/target/release/${binary_name}.*.aarch64-unknown-linux-musl /bin/arm64/${binary_name}
cp -v /app/target/release/${binary_name}.*.x86_64-unknown-linux-musl /bin/amd64/${binary_name}
cp -v /app/target/release/${binary_name}.*.armv7-unknown-linux-musleabihf /bin/arm/${binary_name}
```

## Non-goals

- Don't change `rust/test`, `rust/fmt`, `rust/lint` or other scripts.
- Don't change `mise.toml` in the project root (that's a separate task).
- Don't remove `-C target-feature=+crt-static` from RUSTFLAGS.

## Caveats

- The `CC_<target>` and `CFLAGS_<target>` env var names for the `cc` crate use the target triple with hyphens replaced by underscores. Double-check the exact names.
- For `armv7-unknown-linux-musleabihf`, the Debian multiarch architecture name is `armhf`, and the musl lib path is `/usr/lib/arm-linux-musleabihf/`.
- `cmake` must remain installed — it's used by `aws-lc-sys`.
