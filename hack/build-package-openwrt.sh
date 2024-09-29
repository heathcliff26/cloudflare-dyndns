#!/bin/bash

set -e

base_dir="$(dirname "${BASH_SOURCE[0]}" | xargs realpath)/.."
pkg_dir="${base_dir}/packages/openwrt"

export GOOS=linux
export RELEASE_VERSION="${RELEASE_VERSION:-devel}"

build_package() {
    export GOARCH="${1}"

    case "${GOARCH}" in
    "amd64")
        export PKG_ARCH="x86_64"
        ;;
    "arm64")
        export PKG_ARCH="aarch64"
        ;;
    *)
        echo "Unsupported GOARCH ${GOARCH}"
        exit 1
        ;;
    esac

    echo "Building binary for ${GOARCH}"
    "${base_dir}"/hack/build.sh "cloudflare-dyndns-${GOARCH}"

    echo "Creating control file from template"
    envsubst <"${pkg_dir}/control.template" >"${pkg_dir}/control/control"

    echo "Bundling control folder"
    pushd "${pkg_dir}/control" >/dev/null
    tar --numeric-owner --group=0 --owner=0 -czf ../control.tar.gz ./*
    popd >/dev/null

    echo "Bundling data folder"
    mkdir -p "${pkg_dir}/data/usr/sbin"
    cp "${base_dir}/bin/cloudflare-dyndns-${GOARCH}" "${pkg_dir}/data/usr/sbin/cloudflare-dyndns"
    pushd "${pkg_dir}/data" >/dev/null
    tar --numeric-owner --group=0 --owner=0 -czf ../data.tar.gz ./*
    popd >/dev/null

    pushd "${pkg_dir}" >/dev/null
    tar --numeric-owner --group=0 --owner=0 -czf "${base_dir}/bin/cloudflare-dyndns_${RELEASE_VERSION}_${PKG_ARCH}.ipk" ./debian-binary ./data.tar.gz ./control.tar.gz
    popd >/dev/null
}

for arch in "amd64" "arm64"; do
    build_package "${arch}"
done
