FROM docker.io/library/rust:1.94.1

LABEL   org.opencontainers.image.authors="heathcliff@heathcliff.eu" \
        org.opencontainers.image.description="Builder and CI image for rust projects. Can be used to cross-compile." \
        org.opencontainers.image.source="https://github.com/heathcliff26/cloudflare-dyndns" \
        org.opencontainers.image.licenses="Apache-2.0" \
        org.opencontainers.image.title="rust-builder"

# renovate: datasource=github-releases depName=goreleaser/goreleaser extractVersion=^(?<version>.*)$
ARG GORELEASER_VERSION=v2.15.2

# renovate: datasource=github-releases depName=dyne/musl extractVersion=^(?<version>.*)$
ARG DYNE_MUSL_VERSION=2.1.5

COPY install-deps.sh .
RUN ./install-deps.sh

WORKDIR /app
