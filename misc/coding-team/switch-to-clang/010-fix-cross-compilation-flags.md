# Task 010: Fix cross-compilation flags for musl.cc sysroots

## Context

Testing revealed several issues with the cross-compilation flags:
1. `--gcc-toolchain` doesn't work for armv7 (triple mismatch between musl.cc's `armv7l-linux-musleabihf` and Rust's `armv7-unknown-linux-musleabihf`).
2. `aws-lc-sys` uses cmake for armv7 builds, which needs `CXX_<target>` and `CXXFLAGS_<target>` env vars.
3. `-fuse-ld=lld` in CFLAGS causes `-Werror,-Wunused-command-line-argument` during compile-only steps. Need `-Wno-unused-command-line-argument`.
4. The linker needs `-L` and `-B` pointing at the GCC lib directory inside the musl.cc toolchain to find `crtbeginS.o`, `libgcc.a`, etc.

## Objective

Update `build-target` to use the correct flags that work for all targets. Use `-L`/`-B` instead of `--gcc-toolchain` for consistency and correctness.

## Scope

`.config/mise-lib/rust/build-target` — update the armv7 and aarch64 case branches.

### armv7-unknown-linux-musleabihf

The musl.cc sysroot is at `/opt/armv7l-linux-musleabihf-cross/armv7l-linux-musleabihf`.
The GCC lib dir is at `/opt/armv7l-linux-musleabihf-cross/lib/gcc/armv7l-linux-musleabihf/11.2.1`.

```bash
armv7-unknown-linux-musleabihf)
    export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=clang
    export CC_armv7_unknown_linux_musleabihf=clang
    export CXX_armv7_unknown_linux_musleabihf=clang++
    export CFLAGS_armv7_unknown_linux_musleabihf="--target=armv7-unknown-linux-musleabihf --sysroot=/opt/armv7l-linux-musleabihf-cross/armv7l-linux-musleabihf -L/opt/armv7l-linux-musleabihf-cross/lib/gcc/armv7l-linux-musleabihf/11.2.1 -B/opt/armv7l-linux-musleabihf-cross/lib/gcc/armv7l-linux-musleabihf/11.2.1 -fuse-ld=lld -Wno-unused-command-line-argument"
    export CXXFLAGS_armv7_unknown_linux_musleabihf="--target=armv7-unknown-linux-musleabihf --sysroot=/opt/armv7l-linux-musleabihf-cross/armv7l-linux-musleabihf -L/opt/armv7l-linux-musleabihf-cross/lib/gcc/armv7l-linux-musleabihf/11.2.1 -B/opt/armv7l-linux-musleabihf-cross/lib/gcc/armv7l-linux-musleabihf/11.2.1 -fuse-ld=lld -Wno-unused-command-line-argument"
    RUSTFLAGS="${RUSTFLAGS} -C link-arg=--target=armv7-unknown-linux-musleabihf -C link-arg=--sysroot=/opt/armv7l-linux-musleabihf-cross/armv7l-linux-musleabihf -C link-arg=-L/opt/armv7l-linux-musleabihf-cross/lib/gcc/armv7l-linux-musleabihf/11.2.1 -C link-arg=-fuse-ld=lld"
    ;;
```

### aarch64-unknown-linux-musl

The musl.cc sysroot is at `/opt/aarch64-linux-musl-cross/aarch64-linux-musl`.
The GCC lib dir is at `/opt/aarch64-linux-musl-cross/lib/gcc/aarch64-linux-musl/11.2.1`.

Same pattern as armv7 but with aarch64 paths. Use `-L`/`-B` instead of `--gcc-toolchain` for consistency.

```bash
aarch64-unknown-linux-musl)
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=clang
    export CC_aarch64_unknown_linux_musl=clang
    export CXX_aarch64_unknown_linux_musl=clang++
    export CFLAGS_aarch64_unknown_linux_musl="--target=aarch64-unknown-linux-musl --sysroot=/opt/aarch64-linux-musl-cross/aarch64-linux-musl -L/opt/aarch64-linux-musl-cross/lib/gcc/aarch64-linux-musl/11.2.1 -B/opt/aarch64-linux-musl-cross/lib/gcc/aarch64-linux-musl/11.2.1 -fuse-ld=lld -Wno-unused-command-line-argument"
    export CXXFLAGS_aarch64_unknown_linux_musl="--target=aarch64-unknown-linux-musl --sysroot=/opt/aarch64-linux-musl-cross/aarch64-linux-musl -L/opt/aarch64-linux-musl-cross/lib/gcc/aarch64-linux-musl/11.2.1 -B/opt/aarch64-linux-musl-cross/lib/gcc/aarch64-linux-musl/11.2.1 -fuse-ld=lld -Wno-unused-command-line-argument"
    RUSTFLAGS="${RUSTFLAGS} -C link-arg=--target=aarch64-unknown-linux-musl -C link-arg=--sysroot=/opt/aarch64-linux-musl-cross/aarch64-linux-musl -C link-arg=-L/opt/aarch64-linux-musl-cross/lib/gcc/aarch64-linux-musl/11.2.1 -C link-arg=-fuse-ld=lld"
    ;;
```

### x86_64-unknown-linux-musl

No changes needed — native target uses `musl-tools` from apt.

## Non-goals
- No changes to `setup` or any other files.
