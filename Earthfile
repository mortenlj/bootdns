VERSION 0.7

FROM rust:1-bullseye

WORKDIR /code

ds-qoriq-sdk:
    WORKDIR /tmp/ds-qoriq-sdk
    RUN wget --no-verbose https://global.download.synology.com/download/ToolChain/toolkit/6.2/qoriq/ds.qoriq-6.2.env.txz
    RUN tar xf ds.qoriq-6.2.env.txz
    SAVE ARTIFACT /tmp/ds-qoriq-sdk/usr/local/powerpc-e500v2-linux-gnuspe

    ARG EARTHLY_GIT_PROJECT_NAME
    ARG cache_image=ghcr.io/$EARTHLY_GIT_PROJECT_NAME/cache
    SAVE IMAGE --push ${cache_image}:ds-qoriq-sdk

common-build:
    RUN cargo install cargo-chef
    RUN apt-get --yes update && apt-get --yes install cmake gcc-aarch64-linux-gnu
    RUN rustup toolchain add nightly
    RUN rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

    ARG EARTHLY_GIT_PROJECT_NAME
    ARG cache_image=ghcr.io/$EARTHLY_GIT_PROJECT_NAME/cache
    SAVE IMAGE --push ${cache_image}:common-build

prepare-powerpc-unknown-linux-gnuspe:
    FROM +common-build
    COPY --dir +ds-qoriq-sdk/ /ds-qoriq-sdk/

    ENV PKG_CONFIG_SYSROOT_DIR=/ds-qoriq-sdk/usr/local/powerpc-e500v2-linux-gnuspe/powerpc-e500v2-linux-gnuspe/sysroot/
    ENV TOOLKIT_BIN=/ds-qoriq-sdk/powerpc-e500v2-linux-gnuspe/bin
    ENV CARGO_TARGET_POWERPC_UNKNOWN_LINUX_GNUSPE_LINKER=${TOOLKIT_BIN}/powerpc-e500v2-linux-gnuspe-gcc
    ENV CMAKE_C_COMPILER=${TOOLKIT_BIN}/powerpc-e500v2-linux-gnuspe-gcc
    ENV CMAKE_CXX_COMPILER=${TOOLKIT_BIN}/powerpc-e500v2-linux-gnuspe-g++
    ENV CMAKE_ASM_COMPILER=${TOOLKIT_BIN}/powerpc-e500v2-linux-gnuspe-gcc
    ENV CC_powerpc_unknown_linux_gnuspe=${TOOLKIT_BIN}/powerpc-e500v2-linux-gnuspe-gcc
    ENV RUSTFLAGS="-Ctarget-cpu=e500"

prepare-tier1:
    FROM +common-build

    ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/aarch64-linux-gnu-gcc
    ENV CC_aarch64_unknown_linux_gnu=/usr/bin/aarch64-linux-gnu-gcc

chef-planner:
    FROM +common-build

    COPY --dir src Cargo.lock Cargo.toml .
    RUN cargo chef prepare --recipe-path recipe.json
    SAVE ARTIFACT recipe.json

build-target:
    ARG target
    IF [ "${target}" = "powerpc-unknown-linux-gnuspe" ]
        FROM +prepare-powerpc-unknown-linux-gnuspe
    ELSE
        FROM +prepare-tier1
    END

    COPY +chef-planner/recipe.json recipe.json
    RUN cargo +nightly chef cook --recipe-path recipe.json -Z build-std --target ${target} --release

    COPY --dir src Cargo.lock Cargo.toml .
    RUN cargo +nightly build -Z build-std --target ${target} --release

    ARG version=unknown
    FOR executable IN bootdns ip_test web_test
        SAVE ARTIFACT --if-exists target/${target}/release/${executable} AS LOCAL target/${executable}.${version}.${target}
    END

    ARG EARTHLY_GIT_PROJECT_NAME
    ARG cache_image=ghcr.io/$EARTHLY_GIT_PROJECT_NAME/cache
    SAVE IMAGE --push ${cache_image}:build-${target}

build:
    FOR target IN x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu powerpc-unknown-linux-gnuspe
        BUILD +build-target --target=${target}
    END
