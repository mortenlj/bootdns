# Architecture

## Project purpose

`bootdns` is a small Rust CLI tool that registers the current host's IPv4 address in DNS (via the [Domeneshop](https://domeneshop.no) API) on boot. It inspects all network interfaces, matches each IPv4 address against configured CIDR ranges, and upserts an A record for `<hostname>.<domain>`.

## Detected stack

| Layer | Technology | Evidence |
|---|---|---|
| Language | Rust 2021 edition | `Cargo.toml` |
| HTTP client | `reqwest` 0.13 (blocking, JSON, query) | `Cargo.toml`, `src/domeneshop.rs` |
| Configuration | `figment` (TOML + YAML + env) | `Cargo.toml`, `src/main.rs` |
| Serialisation | `serde` + `serde_json` (via reqwest) | `Cargo.toml` |
| Error handling | `anyhow` | `Cargo.toml`, all `src/*.rs` |
| Logging | `log` + `env_logger` | `Cargo.toml`, `src/main.rs` |
| Networking | `if-addrs`, `cidr`, `hostname` | `Cargo.toml` |
| Auth | `.netrc` via `netrc-rs` | `src/domeneshop.rs` |
| Build/task runner | `mise` | `mise.toml` |
| CI | GitHub Actions | `.github/workflows/main.yaml` |

## Build system

There is **no Makefile or CMake**. The build system is entirely `mise` (formerly `rtx`), a polyglot tool/task manager.

### Task hierarchy

```
mise run ci                  ← lint + fmt-check + test (the "do everything" command)
mise run build               ← debug build for host target
mise run release-build       ← cross-compiled release builds for 3 targets
mise run fmt                 ← cargo fmt --all
mise run fmt-check           ← cargo fmt --all --check
mise run lint                ← cargo clippy --no-deps --all-targets
mise run lint-fix            ← cargo clippy --fix --no-deps --all-targets
mise run test                ← cargo nextest run (with --no-tests warn)
```

All `rust:*` tasks are defined as shell scripts in `.config/mise-lib/rust/` (the `mise-lib` submodule).

## Compiler flags and toolchain

| Setting | Value | Where set |
|---|---|---|
| `RUSTFLAGS` | `-C target-feature=+crt-static` | `.config/mise-lib/rust/build-target`, `rust/setup`, `rust-base.Dockerfile` |
| Linker (armv7) | `arm-linux-gnueabihf-gcc` | `.config/mise-lib/rust/build-target` (env var `CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER`) |
| Linker (aarch64-gnu) | `aarch64-linux-gnu-gcc` | `.config/mise-lib/rust/build-target` |
| Linker (aarch64-musl) | `aarch64-linux-gnu-gcc` | `.config/mise-lib/rust/build-target` |
| Linker (x86_64-musl) | `x86_64-linux-gnu-gcc` | `.config/mise-lib/rust/build-target` |

**Result:** all release binaries are **statically linked** (confirmed by `file` on built artifacts).

### Cross-compilation targets

| Target triple | Arch | Notes |
|---|---|---|
| `x86_64-unknown-linux-gnu` | amd64 | Default host target |
| `aarch64-unknown-linux-gnu` | arm64 | Raspberry Pi 4 / 64-bit ARM |
| `armv7-unknown-linux-gnueabihf` | arm32 | Raspberry Pi 2/3 |
| `x86_64-unknown-linux-musl` | amd64 musl | Available but not in default release |
| `aarch64-unknown-linux-musl` | arm64 musl | Available but not in default release |

Targets are installed via `rustup target add` in `.config/mise-lib/rust/setup`.

### Output naming convention

Built binaries are renamed: `<name>.<version>.<target>` (e.g. `bootdns.0.0.aarch64-unknown-linux-gnu`) and copied to `target/release/`.

## What `mise-lib` is

`.config/mise-lib` is a **git submodule** pointing to `git@github.com:mortenlj/mise-lib.git`. It is a shared library of reusable `mise` tasks maintained separately and consumed by this (and other) repositories.

It provides:
- `rust/` — shell scripts for `build-target`, `fmt`, `fmt-check`, `lint`, `lint-fix`, `setup`, `test`
- `python/` — equivalent scripts for Python projects
- `docker/` — tasks for building/running Docker images and running tasks inside containers
- `baseimages/` — Dockerfiles for `rust-base`, `rust-builder`, `rust`, `python-base`, `python-builder`, `python-prod`
- `extract-ci-info` — Python script that introspects `mise` task graph and git history to produce CI matrix outputs
- `virtual_tasks.toml` — virtual task aliases (e.g. `rust:build`, `rust:release-build`, `docker:build`)
- `.github/workflows/main.yml` — the workflow for building and publishing the base Docker images themselves

The submodule is included via `mise.toml`'s `task_config.includes`:
```toml
[task_config]
includes = [".config/mise-lib", ".config/mise-lib/virtual_tasks.toml"]
```

## CI/CD

**File:** `.github/workflows/main.yaml`

Four jobs, all on `ubuntu-latest`:

| Job | Trigger | What it does |
|---|---|---|
| `setup` | every push | Runs `mise run extract-ci-info` to compute dynamic matrices |
| `ci` | after setup | Runs lint/fmt-check/test tasks (matrix from `ci-tasks` output) |
| `build` | after ci | Runs `mise run release-build` for each target; uploads artifacts |
| `publish` | after build, main branch only, if sources changed | Creates a GitHub Release with all artifacts |

Version scheme: `<major>.<YYYYMMDDHHMM>+<git-describe>` (clean: `<major>.<YYYYMMDDHHMM>`).

Publish is gated on: (1) being on `main` branch AND (2) source files having changed since the last git tag.

## Project structure

```
bootdns/
├── Cargo.toml                  # Package manifest and dependencies
├── Cargo.lock                  # Locked dependency versions
├── mise.toml                   # Task runner config; delegates to mise-lib
├── src/
│   ├── main.rs                 # Entry point: config loading, interface scan, DNS dispatch
│   ├── dns_provider.rs         # Dns trait definition
│   ├── domeneshop.rs           # Domeneshop API client implementing Dns
│   └── bin/
│       ├── ip_test.rs          # Dev utility: print all interfaces
│       └── web_test.rs         # Dev utility: test HTTPS connectivity
├── .config/
│   └── mise-lib/               # Git submodule: shared mise task library
│       ├── rust/               # Rust task scripts (build-target, lint, test, fmt, setup)
│       ├── docker/             # Docker task scripts
│       ├── baseimages/         # Dockerfiles for builder base images
│       ├── extract-ci-info     # Python script for CI matrix generation
│       ├── mise.toml           # mise-lib's own config (tools: python, uv, ruff, cargo-binstall)
│       └── virtual_tasks.toml  # Virtual task aliases
├── .github/
│   └── workflows/
│       └── main.yaml           # CI/CD pipeline
├── misc/
│   └── coding-team/
│       ├── switch-targets-to-musl/  # Empty placeholder (planned: migrate release targets to musl)
│       └── switch-to-clang/         # Empty placeholder (planned: switch cross-compile linker from gcc to clang)
└── bootdns.yaml -> (symlink to external config, gitignored)
```

## Conventions

### Error handling
- Uses `anyhow` throughout: `Result<()>`, `anyhow!()`, `.context()`, `.map_err(|e| anyhow!(e))`
- No `unwrap()` in production paths (only in `DomeneShop::new()` for infallible operations)
- `main()` returns `Result<()>` — errors propagate up and are printed by the runtime

### Logging
- `log` crate macros (`debug!`, `info!`, `warn!`) with `env_logger` backend
- Log level is configurable via `Config.log_level` (default: `"info"`)

### Configuration
- `figment` with layered providers: defaults → TOML → YAML → env vars (`BOOTDNS_` prefix, `__` separator)
- Config file located by searching: `$BOOTDNS_CONFIG_FILE`, `$XDG_CONFIG_HOME/bootdns.*`, `~/bootdns.*`, `./bootdns.*`
- Credentials via `.netrc` (standard Unix credential store), not config file

### Formatting and linting
- `rustfmt` via `cargo fmt --all` (enforced in CI)
- `clippy` via `cargo clippy --no-deps --all-targets` (enforced in CI)

### Testing
- `cargo-nextest` (not `cargo test`)
- `--no-tests warn` — missing tests are a warning, not an error (project currently has no unit tests)
- Tests run against the host's default target only (not cross-compiled)
