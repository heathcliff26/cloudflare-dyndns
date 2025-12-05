#!/bin/bash

set -e

echo "Check if source code is formatted"
make fmt
rc=0
git update-index --refresh && git diff-index --quiet HEAD -- || rc=1
if [ $rc -ne 0 ]; then
    echo "FATAL: Need to run \"make fmt\"" >&2
    exit 1
fi

echo "Check if go.mod and vendor are up to date"
make update-deps
rc=0
git update-index --refresh && git diff-index --quiet HEAD -- || rc=1
if [ $rc -ne 0 ]; then
    echo "FATAL: Need to run \"make update-deps\"" >&2
    exit 1
fi

echo "Check if metainfo file is valid"
make validate-metainfo

echo "Linting helm chart"
make lint-helm
