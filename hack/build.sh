#!/bin/bash

set -e

base_dir="$(dirname "${BASH_SOURCE[0]}" | xargs realpath | xargs dirname)"

bin_dir="${base_dir}/bin"
name="$(yq -r '.package.name' "${base_dir}/Cargo.toml")"

[ -d "${bin_dir}" ] || mkdir -p "${bin_dir}"

CI_COMMIT_SHA="$(git rev-parse HEAD)"
export CI_COMMIT_SHA

cargo build --release

mv "${base_dir}/target/release/${name}" "${bin_dir}/${name}"
