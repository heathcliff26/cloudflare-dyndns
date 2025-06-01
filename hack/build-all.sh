#!/bin/bash

set -e

base_dir="$(dirname "${BASH_SOURCE[0]}" | xargs realpath | xargs dirname)"

export BUILD_ARCHS=(
    "amd64"
    "arm64"
)
export GOOS=linux

for arch in "${BUILD_ARCHS[@]}"; do
    echo "Building for architecture: ${arch}"
    GOARCH="${arch}" "${base_dir}"/hack/build.sh "cloudflare-dyndns-${arch}"
done

echo "Compressing binaries"
"${base_dir}"/hack/compress-binaries.sh
