#!/bin/bash

set -e

base_dir="$(dirname "${BASH_SOURCE[0]}" | xargs realpath | xargs dirname)"
dist_dir="${base_dir}/dist"
bin_dir="${base_dir}/bin"

echo "Checking if goreleaser is installed:"
if command -v goreleaser &>/dev/null; then
    echo "goreleaser is installed"
    goreleaser="$(command -v goreleaser)"
else
    echo "goreleaser is not installed, downloading latest version..."
    goreleaser="curl -sfL https://goreleaser.com/static/run | bash -s --"
    LATEST="$(curl -sf https://goreleaser.com/static/latest)"
    [ -e "${bin_dir}" ] || mkdir "${bin_dir}"
    curl -SL -o "${bin_dir}/goreleaser.tar.gz" "https://github.com/goreleaser/goreleaser/releases/download/${LATEST}/goreleaser_$(uname -s)_$(uname -m).tar.gz"
    tar -xzf "${bin_dir}/goreleaser.tar.gz" -C "${bin_dir}" goreleaser
    rm "${bin_dir}/goreleaser.tar.gz"
    goreleaser="${bin_dir}/goreleaser"
fi

echo "Building releaser artifacts with goreleaser"
${goreleaser} release --skip=announce,publish,validate --clean

echo "Cleaning up dist directory and artifact names"
rm "${dist_dir}/artifacts.json" "${dist_dir}/config.yaml" "${dist_dir}/metadata.json" "${dist_dir}"/cloudflare-dyndns_*_checksums.txt

mv "${dist_dir}/binary_linux_amd64_v1/cloudflare-dyndns" "${dist_dir}/cloudflare-dyndns-amd64"
mv "${dist_dir}/binary_linux_arm64_v8.0/cloudflare-dyndns" "${dist_dir}/cloudflare-dyndns-arm64"
mv "${dist_dir}/compressed_linux_amd64_v1/cloudflare-dyndns" "${dist_dir}/cloudflare-dyndns-amd64-compressed"
mv "${dist_dir}/compressed_linux_arm64_v8.0/cloudflare-dyndns" "${dist_dir}/cloudflare-dyndns-arm64-compressed"

rm -rf "${dist_dir}"/binary_linux_* "${dist_dir}"/compressed_linux_*

echo "Finished building release artifacts, output is in ${dist_dir}"
