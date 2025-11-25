#!/bin/sh

set -e

if [ "${TARGETPLATFORM}" = "linux/amd64" ]; then
    EXPECTED_ARCH="x86-64"
elif [ "${TARGETPLATFORM}" = "linux/arm64" ]; then
    EXPECTED_ARCH="aarch64"
else
    echo "Unsupported platform ${TARGETPLATFORM}"
    exit 1
fi

if ! file /final/entrypoint | grep "${EXPECTED_ARCH}" >/dev/null 2>&1; then
    echo "architecture mismatch in final binary: expected ${EXPECTED_ARCH} " 1>&2
    file /final/entrypoint 1>&2
    exit 1
fi

if [ "${BUILD_TYPE}" = "release" ]; then
    if [ -f /final/bin/sh ]; then
        echo 'Error: `sh` present in release build' 1>&2
        exit 1
    fi
    if file /final/entrypoint | grep "not stripped"; then
        echo "Error: Release binary is not stripped" 1>&2
        exit 1
    fi
else
    if [ ! -f /final/bin/sh ]; then
        echo 'Error: No `sh` present in debug build' 1>&2
        exit 1
    fi
    if ! file /final/entrypoint | grep "not stripped"; then
        echo "Error: Debug binary is stripped" 1>&2
        exit 1
    fi
fi
