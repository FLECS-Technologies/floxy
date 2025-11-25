#!/bin/sh

set -e

if [ "${TARGETPLATFORM}" = "linux/amd64" ]; then
    BINUTILS=${BINUTILS:-binutils}
    CARGO_BUILD_TARGET=${CARGO_BUILD_TARGET:-x86_64-unknown-linux-musl}
    RUST_LINKER=${RUST_LINKER:-ld}
elif [ "${TARGETPLATFORM}" = "linux/arm64" ]; then
    BINUTILS=${BINUTILS:-binutils-aarch64}
    CARGO_BUILD_TARGET=${CARGO_BUILD_TARGET:-aarch64-unknown-linux-musl}
    RUST_LINKER=${RUST_LINKER:-aarch64-alpine-linux-musl-ld}
else
    echo "Unsupported platform ${TARGETPLATFORM}" 1>&2
    exit 1
fi

if [ ! -z "${BINUTILS}" ]; then
    apk add --no-cache ${BINUTILS}
fi

if [ "${BUILD_TYPE}" = "release" ]; then
    CARGO_PROFILE_ARG="release"
    CARGO_PROFILE_DIR="release"
else
    CARGO_PROFILE_ARG="dev"
    CARGO_PROFILE_DIR="debug"
fi

export CARGO_BUILD_TARGET
export CARGO_PROFILE_ARG
export CARGO_PROFILE_DIR
export RUSTFLAGS="-C linker=${RUST_LINKER}"

cargo build --profile ${CARGO_PROFILE_ARG} --target ${CARGO_BUILD_TARGET}
cp ./target/${CARGO_BUILD_TARGET}/${CARGO_PROFILE_DIR}/floxy-entrypoint /entrypoint
