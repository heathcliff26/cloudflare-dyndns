#!/bin/sh /etc/rc.common

# shellcheck disable=SC3043
# shellcheck disable=SC2034

USE_PROCD=1
START=80

PROG=/usr/sbin/cloudflare-dyndns

start_service() {
    local enabled
    local log_level
    local mode
    local token
    local proxy
    local domain
    local interval
    local relay_endpoint

    config_load cloudflare-dyndns
    config_get_bool enabled "main" enabled "1"
    config_get log_level "main" log_level "info"
    config_get mode "main" mode "client"
    config_get token "main" token ""
    config_get_bool proxy "main" proxy "1"
    config_get domain "main" domain ""
    config_get interval "main" interval "5m"
    config_get relay_endpoint "main" relay_endpoint ""

    if [ "$enabled" = "1" ]; then
        procd_open_instance
        procd_set_param command $PROG "$mode"
        procd_append_param command --config /etc/cloudflare-dyndns/config.yaml
        procd_append_param command --env

        procd_set_param env CLOUDFLARE_DYNDNS_LOG_LEVEL="$log_level"
        procd_append_param env CLOUDFLARE_DYNDNS_TOKEN="$token"
        if [ "$proxy" = "1" ]; then
            procd_append_param env CLOUDFLARE_DYNDNS_PROXY=true
        else
            procd_append_param env CLOUDFLARE_DYNDNS_PROXY=false
        fi
        procd_append_param env CLOUDFLARE_DYNDNS_DOMAIN="$domain"
        procd_append_param env CLOUDFLARE_DYNDNS_INTERVAL="$interval"
        procd_append_param env CLOUDFLARE_DYNDNS_RELAY_ENDPOINT="$relay_endpoint"

        procd_set_param stdout 1
        procd_set_param stderr 1

        procd_set_param user nobody

        procd_close_instance
    fi
}
