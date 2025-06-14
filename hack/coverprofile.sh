#!/bin/bash

set -e

base_dir="$(dirname "${BASH_SOURCE[0]}" | xargs realpath | xargs dirname)"

pushd "${base_dir}" >/dev/null

OUT_DIR="${base_dir}/coverprofiles"

if [ ! -d "${OUT_DIR}" ]; then
    mkdir "${OUT_DIR}"
fi

make test
go tool cover -html "coverprofile.out" -o "${OUT_DIR}/index.html"

popd >/dev/null
