#!/bin/bash

set -e

base_dir="$(dirname "${BASH_SOURCE[0]}" | xargs realpath | xargs dirname)"
dist_dir="${base_dir}/dist"

echo "Building releaser artifacts with goreleaser"
podman run --name cloudflare-dyndns-builder --rm -v "${base_dir}:/app:z" ghcr.io/heathcliff26/rust-builder:latest goreleaser release --skip=announce,archive,publish,validate --clean

echo "Moving release artifacts to top level of dist directory"
artifacts="$(cat "${dist_dir}/artifacts.json" | jq -r -c '.[]')"
echo "${artifacts}" | while read -r artifact; do
    if [ "$(echo "${artifact}" | jq -r '.name')" != "cloudflare-dyndns" ]; then
        continue
    fi

    path="${base_dir}/$(echo "${artifact}" | jq -r '.path')"
    goarch="$(echo "${artifact}" | jq -r '.goarch')"
    id="$(echo "${artifact}" | jq -r '.extra.ID')"

    if [ "${id}" == "dynamic" ]; then
        mv "${path}" "${dist_dir}/cloudflare-dyndns-${goarch}"
    else
        mv "${path}" "${dist_dir}/cloudflare-dyndns-${goarch}-${id}"
    fi
    path="$(dirname "${path}")"
    rm -r "${path}"
done


echo "Cleaning up dist directory"
rm -r "${dist_dir}/artifacts.json" "${dist_dir}/config.yaml" "${dist_dir}/metadata.json" "${dist_dir}"/cloudflare-dyndns_*_checksums.txt
