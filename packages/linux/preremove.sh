#!/bin/sh

if [ "$1" = "0" ]; then
    exit 0
fi

for mode in "client" "relay" "server"; do
    echo "Clean up cloudflare-dyndns-${mode} service"
    systemctl unmask cloudflare-dyndns-${mode}.service
    systemctl stop cloudflare-dyndns-${mode}.service
    systemctl disable cloudflare-dyndns-${mode}.service
done
