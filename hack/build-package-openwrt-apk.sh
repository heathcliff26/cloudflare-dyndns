#!/bin/bash

set -e

base_dir="$(dirname "${BASH_SOURCE[0]}" | xargs realpath | xargs dirname)"
pkg_dir="${base_dir}/packages/openwrt-apk"

export RELEASE_VERSION="${RELEASE_VERSION:-v0.0.0_alpha}"

arch="${1}"
case "$(uname -m)" in
"amd64"|"x86_64")
    export PKG_ARCH="x86_64"
    export arch="amd64"
    ;;
"arm64"|"aarch64")
    export PKG_ARCH="aarch64"
    export arch="arm64"
    ;;
*)
    # shellcheck disable=SC2155
    export PKG_ARCH="$(uname -m)"
    # shellcheck disable=SC2155
    export arch="$(uname -m)"
    ;;
esac

echo "Building package for ${PKG_ARCH} (${arch})"

export BUILD_ARCHS="${arch}"
"${base_dir}"/hack/build-all.sh

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

echo "Creating APKBUILD file from template"
sed "${pkg_dir}/APKBUILD.template" \
    -e "s/RELEASE_VERSION/${RELEASE_VERSION#v}/g" \
    -e "s/PKG_ARCH/${PKG_ARCH}/g" \
    >"${pkg_dir}/APKBUILD"

echo "Moving binary to build folder"
cp "${base_dir}/bin/cloudflare-dyndns-${arch}-compressed" "${pkg_dir}/cloudflare-dyndns"

echo "Running package build in container"
mkdir -p "${pkg_dir}/dst"
podman run --rm --name cloudflare-dyndns-openwrt-apk-builder \
    -v "${pkg_dir}:/build:z" \
    -v "${pkg_dir}/dst:/root/packages:z" \
    -e "PACKAGER_PRIVKEY=/build/abuild.rsa" \
    localhost/alpine-builder:latest \
    abuild -r -F checksum prepare validate clean fetch rootpkg

echo "Moving package to bin folder"
mv "${pkg_dir}/dst/${PKG_ARCH}/cloudflare-dyndns-${RELEASE_VERSION#v}-r0.apk" "${base_dir}/bin/cloudflare-dyndns_${RELEASE_VERSION}_openwrt-${PKG_ARCH}.apk"

echo "Cleaning up build files"
rm -rf "${pkg_dir}/APKBUILD" "${pkg_dir}/cloudflare-dyndns" "${pkg_dir}/dst" "${pkg_dir}/src" "${pkg_dir}/pkg"
