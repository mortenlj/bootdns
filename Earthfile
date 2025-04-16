VERSION 0.8

IMPORT github.com/mortenlj/earthly-lib/rust/commands AS lib-commands
IMPORT github.com/mortenlj/earthly-lib/rust/targets AS lib-targets

FROM rust:1

WORKDIR /code

chef-planner:
    FROM lib-targets+common-build-setup

    DO lib-commands+CHEF_PREPARE
    SAVE ARTIFACT recipe.json

build-target:
    FROM lib-targets+prepare-tier1

    COPY +chef-planner/recipe.json recipe.json

    ARG target
    DO lib-commands+BUILD --target ${target}

    ARG version=unknown
    FOR executable IN bootdns ip_test web_test
        SAVE ARTIFACT --if-exists target/${target}/release/${executable} AS LOCAL target/${executable}.${version}.${target}
    END

    SAVE IMAGE --push ghcr.io/mortenlj/bootdns/cache:build-${target}

build:
    FOR target IN x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu armv7-unknown-linux-gnueabihf
        BUILD +build-target --target=${target}
    END
