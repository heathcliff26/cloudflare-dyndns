#!/bin/sh

if ! command -V systemctl >/dev/null 2>&1; then
    exit 0
fi

systemctl daemon-reload || true
