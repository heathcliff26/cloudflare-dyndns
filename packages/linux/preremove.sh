#!/bin/sh

if [ "$1" != "0" ] && [ "$1" != "remove" ] && [ "$1" != "purge" ]; then
    exit 0
fi
if ! command -V systemctl >/dev/null 2>&1; then
    exit 0
fi

for mode in "client" "relay" "server"; do
    echo "Clean up cloudflare-dyndns-${mode} service"
    systemctl unmask cloudflare-dyndns-${mode}.service  || true
    systemctl disable --now cloudflare-dyndns-${mode}.service
done
