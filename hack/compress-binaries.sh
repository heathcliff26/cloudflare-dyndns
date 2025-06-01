#!/bin/bash

set -e

base_dir="$(dirname "${BASH_SOURCE[0]}" | xargs realpath | xargs dirname)"
bin_dir="${base_dir}/bin"
file_ending="-compressed"

[ ! -d "${bin_dir}" ] && echo "Error: bin directory does not exist. Please run build.sh first." >&2 && exit 1

pushd "${bin_dir}" >/dev/null

# use nullglob in case there are no matching files
shopt -s nullglob

rm ./*"${file_ending}" || true

for file in *; do
    # Skip compressed binaries
    [[ "${file}" = *"${file_ending}" ]] && continue
    # Skip openwrt packages
    [[ "${file}" = *.ipk ]] && continue

    output="${file}${file_ending}"
    echo "Compressing ${file} to ${output}"
    upx -9 -o "${output}" "${file}"
    upx -t "${output}"
done

popd >/dev/null
