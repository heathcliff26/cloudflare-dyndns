#!/bin/bash

set -e

base_dir="$(dirname "${BASH_SOURCE[0]}" | xargs realpath | xargs dirname)"
pkg_dir="${base_dir}/packages/openwrt"

export RELEASE_VERSION="${RELEASE_VERSION:-devel}"

build_package() {
    arch="${1}"

    case "${arch}" in
    "amd64")
        export PKG_ARCH="x86_64"
        ;;
    "arm64")
        export PKG_ARCH="aarch64"
        ;;
    *)
        export PKG_ARCH="${arch}"
        ;;
    esac

    echo "Creating control file from template"
    envsubst <"${pkg_dir}/control.template" >"${pkg_dir}/control/control"

    echo "Bundling control folder"
    pushd "${pkg_dir}/control" >/dev/null
    tar --numeric-owner --group=0 --owner=0 -czf ../control.tar.gz ./*
    popd >/dev/null

    echo "Bundling data folder"
    mkdir -p "${pkg_dir}/data/usr/sbin"
    cp "${base_dir}/bin/cloudflare-dyndns-${arch}-compressed" "${pkg_dir}/data/usr/sbin/cloudflare-dyndns"
    pushd "${pkg_dir}/data" >/dev/null
    tar --numeric-owner --group=0 --owner=0 -czf ../data.tar.gz ./*
    popd >/dev/null

    pushd "${pkg_dir}" >/dev/null
    tar --numeric-owner --group=0 --owner=0 -czf "${base_dir}/bin/cloudflare-dyndns_${RELEASE_VERSION}_${PKG_ARCH}.ipk" ./debian-binary ./data.tar.gz ./control.tar.gz
    popd >/dev/null
}

# shellcheck source=build-all.sh
source "${base_dir}"/hack/build-all.sh

for arch in "${BUILD_ARCHS[@]}"; do
    build_package "${arch}"
done
