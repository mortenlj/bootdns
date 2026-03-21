# Task 007: Fix musl include paths for C compilation

## Context

Build verification revealed that `aws-lc-sys` C code compiled with `clang --target=x86_64-unknown-linux-musl` picks up glibc headers from the host system instead of musl headers. This causes linker errors for glibc-specific symbols like `__isoc23_sscanf` and `__isoc23_strtol` that don't exist in musl.

The fix: add `-isystem /usr/include/<musl-triple>` and `-nostdinc` to the CFLAGS so clang uses musl headers exclusively.

On Debian, musl headers are installed at:
- x86_64: `/usr/include/x86_64-linux-musl/`
- aarch64: `/usr/include/aarch64-linux-musl/`
- armhf: `/usr/include/arm-linux-musleabihf/`

## Objective

Update the `CFLAGS_<target>` env vars in `build-target` to include musl header paths.

## Scope

`.config/mise-lib/rust/build-target` — update each case branch's `CFLAGS_<target>` to include the musl system include path. The clang built-in headers (for stddef.h, stdarg.h, etc.) must still be available, so use `-nostdinc -isystem <clang-builtins> -isystem <musl-headers>`.

Actually, the simplest approach: just prepend `-isystem /usr/include/<musl-triple>` to the CFLAGS. This makes clang search musl headers before the default system headers. No need for `-nostdinc` since the musl headers will be found first.

Update each branch:
```bash
armv7-unknown-linux-musleabihf)
    ...
    export CFLAGS_armv7_unknown_linux_musleabihf="--target=armv7-unknown-linux-musleabihf -isystem /usr/include/arm-linux-musleabihf"
    ...

aarch64-unknown-linux-musl)
    ...
    export CFLAGS_aarch64_unknown_linux_musl="--target=aarch64-unknown-linux-musl -isystem /usr/include/aarch64-linux-musl"
    ...

x86_64-unknown-linux-musl)
    ...
    export CFLAGS_x86_64_unknown_linux_musl="--target=x86_64-unknown-linux-musl -isystem /usr/include/x86_64-linux-musl"
    ...
```

## Non-goals
- No other changes.
