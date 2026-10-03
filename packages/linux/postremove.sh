#!/bin/sh

if [ "$1" = "0" ]; then
    exit 0
fi

systemctl daemon-reload
systemctl reset-failed
