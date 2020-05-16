#!/bin/bash

start-docker() {
    docker-compose up
}

cargo-build() {
    docker exec \
    -it \
    ls-key_coding_1 \
    cargo build
}

cargo-bin-build() {
    docker exec \
    -it \
    ls-key_coding_1 \
    cargo build --bin lsk
}

lsk-extend-cargo-bin-build() {
    docker exec \
    -it \
    -w /ls-key/lsk-extend \
    ls-key_coding_1 \
    cargo build --bin lsk-extend
}

build() {
    cargo-build
    cargo-bin-build
}

rust-lib-test() {
    docker exec \
    -it \
    ls-key_coding_1 \
    cargo test $1 -- --test-threads=1 --nocapture
}

rust-bin-test() {
    docker exec \
    -it \
    ls-key_coding_1 \
    cargo test --bin $1
}

cp-lsk-extend() {
    docker exec \
    -it \
    -w /ls-key/lsk-extend \
    ls-key_coding_1 \
    cp target/debug/lsk-extend /usr/local/bin/lsk-extend
}

run-test() {
    if [ "$1" = "rust-lib" ]; then
        cargo-bin-build
        lsk-extend-cargo-bin-build
        cp-lsk-extend
        rust-lib-test $2
    fi

    if [ "$1" = "rust-bin" ]; then
        cargo-bin-build
        lsk-extend-cargo-bin-build
        cp-lsk-extend
        rust-bin-test $2
    fi

    if [ "$1" = "" ]; then
        cargo-bin-build
        lsk-extend-cargo-bin-build
        cp-lsk-extend
        rust-lib-test
        rust-bin-test
    fi
}

if [ "$1" == "rust-build" ]; then
    echo "rust build"
    cargo-build
fi

if [ "$1" == "start" ]; then
    echo "starting primary container"
    start-primary-container
fi

if [ "$1" == "create" ]; then
    echo "creating primary container"
    create-primary-container
fi

if [ "$1" == "test" ]; then
    echo "test"
    run-test $2 $3
fi
