VERSION 0.6

FROM rust:1-bullseye

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

prepare-tier1:
    RUN apt-get --yes update && apt-get --yes install cmake gcc-aarch64-linux-gnu

    ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/aarch64-linux-gnu-gcc
    ENV CC_aarch64_unknown_linux_gnu=/usr/bin/aarch64-linux-gnu-gcc

    RUN rustup toolchain add nightly
    RUN rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

    SAVE IMAGE --cache-hint


build-target:
    ARG target
    IF [ "${target}" = "powerpc-unknown-linux-gnuspe" ]
        FROM +prepare-powerpc-unknown-linux-gnuspe
    ELSE
        FROM +prepare-tier1
    END

    COPY --dir src Cargo.lock Cargo.toml .
    RUN cargo +nightly build -Z build-std --target ${target} --release

    ARG version=unknown
    FOR executable IN bootdns ip_test web_test
        SAVE ARTIFACT --if-exists target/${target}/release/${executable} AS LOCAL target/${executable}.${version}.${target}
    END

    SAVE IMAGE --cache-hint

build:
    FOR target IN x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu powerpc-unknown-linux-gnuspe
        BUILD +build-target --target=${target}
    END
