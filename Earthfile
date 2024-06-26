VERSION 0.7

IMPORT github.com/mortenlj/earthly-lib/rust/commands AS lib-commands
IMPORT github.com/mortenlj/earthly-lib/rust/targets AS lib-targets

FROM rust:1-bullseye

WORKDIR /code

chef-planner:
    FROM lib-targets+common-build-setup

    DO lib-commands+CHEF_PREPARE
    SAVE ARTIFACT recipe.json

build-target:
    ARG target
    IF [ "${target}" = "powerpc-unknown-linux-gnuspe" ]
        FROM lib-targets+prepare-powerpc-unknown-linux-gnuspe
    ELSE
        FROM lib-targets+prepare-tier1
    END

    COPY +chef-planner/recipe.json recipe.json
    DO lib-commands+BUILD --target ${target}

    ARG version=unknown
    FOR executable IN bootdns ip_test web_test
        SAVE ARTIFACT --if-exists target/${target}/release/${executable} AS LOCAL target/${executable}.${version}.${target}
    END

    SAVE IMAGE --push ghcr.io/mortenlj/bootdns/cache:build-${target}

build:
    FOR target IN x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu powerpc-unknown-linux-gnuspe armv7-unknown-linux-gnueabihf
        BUILD +build-target --target=${target}
    END
