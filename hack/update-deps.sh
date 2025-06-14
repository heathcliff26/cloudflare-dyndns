#!/bin/bash

set -e

base_dir="$(dirname "${BASH_SOURCE[0]}" | xargs realpath | xargs dirname)"

pushd "${base_dir}" >/dev/null
go mod tidy
go mod vendor
popd >/dev/null
