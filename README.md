[![CI](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/ci.yaml/badge.svg?event=push)](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/ci.yaml)
[![Coverage Status](https://coveralls.io/repos/github/heathcliff26/cloudflare-dyndns/badge.svg)](https://coveralls.io/github/heathcliff26/cloudflare-dyndns)
[![Editorconfig Check](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/editorconfig-check.yaml/badge.svg?event=push)](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/editorconfig-check.yaml)
[![Coverprofiles](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/coverprofiles.yaml/badge.svg)](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/coverprofiles.yaml)
[![Renovate](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/renovate.yaml/badge.svg)](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/renovate.yaml)
[![Builder Image](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/build-builder.yaml/badge.svg)](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/build-builder.yaml)

# cloudflare-dyndns

Standalone binary for dynamically updating your home IP on your Cloudflare-managed DNS domain.

All domains whose DNS is managed by Cloudflare can be updated with this.

It implements both POST and GET endpoints to support a wide variety of devices.

## Table of Contents

- [cloudflare-dyndns](#cloudflare-dyndns)
  - [Table of Contents](#table-of-contents)
  - [Container Images](#container-images)
    - [Image location](#image-location)
    - [Tags](#tags)
  - [Usage](#usage)
    - [Kubernetes](#kubernetes)
  - [OpenWrt](#openwrt)
  - [API (Server Mode)](#api-server-mode)
    - [Examples](#examples)

## Container Images

### Image location

| Container Registry                                                                                     | Image                                      |
| ------------------------------------------------------------------------------------------------------ | ------------------------------------------ |
| [GitHub Container](https://github.com/users/heathcliff26/packages/container/package/cloudflare-dyndns) | `ghcr.io/heathcliff26/cloudflare-dyndns`   |
| [Docker Hub](https://hub.docker.com/r/heathcliff26/cloudflare-dyndns)                                  | `docker.io/heathcliff26/cloudflare-dyndns` |
| [Quay](https://quay.io/repository/heathcliff26/cloudflare-dyndns)                                      | `quay.io/heathcliff26/cloudflare-dyndns`   |

### Tags

There are different flavors of the image:

| Tag(s)      | Description                                                 |
| ----------- | ----------------------------------------------------------- |
| **latest**  | Last released version of the image                          |
| **rolling** | Rolling update of the image, always build from main branch. |
| **vX.Y.Z**  | Released version of the image                               |

## Usage

The binary can be run either as a server, a standalone client or in relay mode where it will call a server.

The main use case for relay mode would be when you want to restrict your Cloudflare API key to a static IP.

Output of `cloudflare-dyndns help`
```bash
cloudflare-dyndns provides DynDNS functionality for Cloudflare

Usage: cloudflare-dyndns <COMMAND>

Commands:
  server   Run a server for relay clients
  client   Update DDNS Records by calling the Cloudflare API
  relay    Update DDNS Records but relay the calls through a server
  version  Print version information and exit
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
An example config can be found [here](examples/example-config.yaml).

### Kubernetes

The container image can be deployed as part of a daemonset, to ensure all nodes in your cluster have a valid DNS record.

An example daemonset using the relay mode can be found [here](examples/example-relay-daemonset.yaml).

Alternatively Helm charts are released via oci repos and can be installed with:
```
helm install cloudflare-dyndns oci://ghcr.io/heathcliff26/manifests/cloudflare-dyndns --version <version>
```
Please use the latest version from the releases page.

## OpenWrt

When installing the arm64 package on OpenWrt 24.10 or older, you need to enable arm64 as an architecture in your opkg.conf.
This is caused by goreleaser not using aarch64 as architecture string for .ipk packages.

## API (Server Mode)

| Parameter | Description                                            |
| --------- | ------------------------------------------------------ |
| token     | Token needed for accessing Cloudflare API              |
| domains   | The domain to update, can be specified multiple times  |
| ipv4      | IPv4 Address, optional, when IPv6 set                  |
| ipv6      | IPv6 Address, optional, when IPv4 set                  |
| proxy     | Indicate if domain should be proxied, defaults to true |

### Examples

Here is an example GET request:
```text
https://dyndns.example.com/?token=testtoken&domains=foo.example.net&domains=bar.example.org&domains=example.net&ipv4=100.100.100.100&ipv6=fd00::dead&proxy=true
```

When using POST the format is:
```json
{
  "token": "",
  "domains": [
    "foo.example.org",
    "bar.example.net"
  ],
  "ipv4": "100.100.100.100",
  "ipv6": "fd00::dead",
  "proxy": true
}
```
