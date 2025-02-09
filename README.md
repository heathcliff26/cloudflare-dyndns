[![CI](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/ci.yaml/badge.svg?event=push)](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/ci.yaml)
[![Coverage Status](https://coveralls.io/repos/github/heathcliff26/cloudflare-dyndns/badge.svg)](https://coveralls.io/github/heathcliff26/cloudflare-dyndns)
[![Editorconfig Check](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/editorconfig-check.yaml/badge.svg?event=push)](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/editorconfig-check.yaml)
[![Generate go test cover report](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/go-testcover-report.yaml/badge.svg)](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/go-testcover-report.yaml)
[![Renovate](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/renovate.yaml/badge.svg)](https://github.com/heathcliff26/cloudflare-dyndns/actions/workflows/renovate.yaml)

# cloudflare-dyndns

Implements the API from [Fritz!Box DynDNS Script for Cloudflare](https://github.com/1rfsNet/Fritz-Box-Cloudflare-DynDNS), but can also be used as a standalone client.

Additionally to consuming less resources and being a smaller image, it also implements POST in addition to GET requests, meaning no longer does the token need to be included in the url.

The client package can also be used as a golang API, should you want to build your application with included cloudflare dyndns capabilities.

## Table of Contents

- [cloudflare-dyndns](#cloudflare-dyndns)
  - [Table of Contents](#table-of-contents)
  - [Container Images](#container-images)
    - [Image location](#image-location)
    - [Tags](#tags)
  - [Usage](#usage)
    - [Kubernetes](#kubernetes)
  - [API (Server Mode)](#api-server-mode)
    - [Examples](#examples)

## Container Images

### Image location

| Container Registry                                                                                     | Image                                      |
| ------------------------------------------------------------------------------------------------------ | ------------------------------------------ |
| [Github Container](https://github.com/users/heathcliff26/packages/container/package/cloudflare-dyndns) | `ghcr.io/heathcliff26/cloudflare-dyndns`   |
| [Docker Hub](https://hub.docker.com/r/heathcliff26/cloudflare-dyndns)                  | `docker.io/heathcliff26/cloudflare-dyndns` |

### Tags

There are different flavors of the image:

| Tag(s)      | Description                                                 |
| ----------- | ----------------------------------------------------------- |
| **latest**  | Last released version of the image                          |
| **rolling** | Rolling update of the image, always build from main branch. |
| **vX.Y.Z**  | Released version of the image                               |

## Usage

The binary can be run either as a server, a standalone client or in relay mode where it will call a server.

The main use case for relay mode would be when you want to restrict your cloudflare API key to a static IP.

Output of `cloudflare-dyndns help`
```
cloudflare-dyndns provides DynDNS functionality for cloudflare.

Usage:
  cloudflare-dyndns [flags]
  cloudflare-dyndns [command]

Available Commands:
  client      Update DDNS Records by calling the cloudflare API
  completion  Generate the autocompletion script for the specified shell
  help        Help about any command
  relay       Update DDNS Records but relay the calls through a server
  server      Run a server for relay clients
  version     Print version information and exit

Flags:
  -h, --help   help for cloudflare-dyndns

Use "cloudflare-dyndns [command] --help" for more information about a command.
```
An example config can be found [here](examples/example-config.yaml).

### Kubernetes

The container image can be deployed as part of a daemonset, to ensure all nodes in your cluster have a valid DNS record.

An example daemonset using the relay mode can be found [here](examples/example-relay-daemonset.yaml).

## API (Server Mode)

| Parameter        | Description                                                                    |
| ---------------- | ------------------------------------------------------------------------------ |
| token (cf_key)   | Token needed for accessing cloudflare api                                      |
| domains (domain) | The domain to update, parsed from comma (,) separated string, needs at least 1 |
| ipv4             | IPv4 Address, optional, when IPv6 set                                          |
| ipv6             | IPv6 Address, optional, when IPv4 set                                          |
| proxy            | Indicate if domain should be proxied, defaults to true                         |

### Examples

Here is an example GET request:
```
https://dyndns.example.com/?token=testtoken&domains=foo.example.net,bar.example.org,example.net&ipv4=100.100.100.100&ipv6=fd00::dead&proxy=true
```
or alternatively in the format [Fritz!Box DynDNS Script for Cloudflare](https://github.com/1rfsNet/Fritz-Box-Cloudflare-DynDNS) from :
```
http://example.org/?cf_key=testtoken&domain=foo.example.net&ipv4=100.100.100.100&ipv6=fd00::dead&proxy=true
```
When using POST the format is:
```
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
