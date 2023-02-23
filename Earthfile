VERSION 0.6

FROM rust:1.67

WORKDIR /code

# Constants, do not override
ARG cross_version=0.2.5  # https://github.com/cross-rs/cross/releases

ds-qoriq-sdk:
    FROM alpine:3
    WORKDIR /tmp/ds-qoriq-sdk
    RUN wget --no-verbose https://global.download.synology.com/download/ToolChain/toolkit/6.2/qoriq/ds.qoriq-6.2.env.txz
    RUN tar xf ds.qoriq-6.2.env.txz
    SAVE ARTIFACT /tmp/ds-qoriq-sdk/usr/local/powerpc-e500v2-linux-gnuspe
    SAVE IMAGE --cache-hint

prepare-powerpc-unknown-linux-gnuspe:
    COPY --dir +ds-qoriq-sdk/ /ds-qoriq-sdk/

    ENV PKG_CONFIG_SYSROOT_DIR=/ds-qoriq-sdk/usr/local/powerpc-e500v2-linux-gnuspe/powerpc-e500v2-linux-gnuspe/sysroot/
    ENV TOOLKIT_BIN=/ds-qoriq-sdk/powerpc-e500v2-linux-gnuspe/bin
    ENV CARGO_TARGET_POWERPC_UNKNOWN_LINUX_GNUSPE_LINKER=${TOOLKIT_BIN}/powerpc-e500v2-linux-gnuspe-gcc

    ENV CMAKE_C_COMPILER=${TOOLKIT_BIN}/powerpc-e500v2-linux-gnuspe-gcc
    ENV CMAKE_CXX_COMPILER=${TOOLKIT_BIN}/powerpc-e500v2-linux-gnuspe-g++
    ENV CMAKE_ASM_COMPILER=${TOOLKIT_BIN}/powerpc-e500v2-linux-gnuspe-gcc
    ENV CC_powerpc_unknown_linux_gnuspe=${TOOLKIT_BIN}/powerpc-e500v2-linux-gnuspe-gcc

    ENV RUSTFLAGS="-Ctarget-cpu=e500"

    RUN apt-get --yes update && apt-get --yes install cmake
    RUN ${CARGO_TARGET_POWERPC_UNKNOWN_LINUX_GNUSPE_LINKER} --version
    RUN rustup toolchain add nightly
    RUN rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

    SAVE IMAGE --cache-hint

build-powerpc-unknown-linux-gnuspe:
    FROM +prepare-powerpc-unknown-linux-gnuspe
    ARG target=powerpc-unknown-linux-gnuspe

    COPY --dir src Cargo.lock Cargo.toml .
    RUN cargo +nightly build -Z build-std --target ${target} --release

    ARG version=unknown
    FOR executable IN bootdns ip_test web_test
        SAVE ARTIFACT --if-exists target/${target}/release/${executable} AS LOCAL target/${executable}.${version}.${target}
    END

    SAVE IMAGE --cache-hint

prepare-tier1:
    RUN cargo install cross --version ${cross_version}
    COPY --dir src Cargo.lock Cargo.toml .
    SAVE IMAGE --cache-hint

build-tier1:
    FROM +prepare-tier1
    ARG target

    WITH DOCKER \
        --pull ghcr.io/cross-rs/${target}:${cross_version}
        RUN cross build --target ${target} --release
    END

    ARG version=unknown
    FOR executable IN bootdns ip_test web_test
        SAVE ARTIFACT --if-exists target/${target}/release/${executable} AS LOCAL target/${executable}.${version}.${target}
    END

    SAVE IMAGE --cache-hint

build:
    BUILD +build-powerpc-unknown-linux-gnuspe
    FOR target IN x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
        BUILD +build-tier1 --target=${target}
    END
