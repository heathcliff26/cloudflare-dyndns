# cloudflare-dyndns Helm Chart

This Helm chart for cloudflare-dyndns - DynDNS client or relay server for cloudflare

## Prerequisites

- Kubernetes 1.32+
- Helm 3.19+
- FluxCD installed in the cluster (recommended)

## Installation

### Installing from OCI Registry (GitHub Packages)

```bash
# Install the chart
helm install cloudflare-dyndns oci://ghcr.io/heathcliff26/manifests/cloudflare-dyndns --version <version>
```

## Configuration

### Minimal Configuration (No Ingress)

Leaving all options empty will deploy a server with no ingress.

## Values Reference

See [values.yaml](./values.yaml) for all available configuration options.

### Key Parameters

| Parameter                | Description                                              | Default                                  |
| ------------------------ | -------------------------------------------------------- | ---------------------------------------- |
| `type`                   | The mode to deploy, can be "server", "relay" or "client" | `server`                                 |
| `image.repository`       | Container image repository                               | `ghcr.io/heathcliff26/cloudflare-dyndns` |
| `image.tag`              | Container image tag                                      | Same as chart version                    |
| `ingress.enabled`        | Enable ingress                                           | `false`                                  |
| `servicemonitor.enabled` | Create a ServiceMonitor for the Prometheus Operator      | `false`                                  |

## Support

For more information, visit: https://github.com/heathcliff26/cloudflare-dyndns
