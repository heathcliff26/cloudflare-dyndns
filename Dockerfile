###############################################################################
# BEGIN build-stage
# Compile the binary
FROM docker.io/library/rust:1.95.0-alpine AS build-stage

WORKDIR /app

RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static

COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Needed as we include it for docs.
RUN touch README.md

ARG CI_COMMIT_SHA=unknown

RUN cargo build --release

#
# END build-stage
###############################################################################

###############################################################################
# BEGIN final-stage
# Create final docker image
FROM docker.io/library/alpine:3.23.3@sha256:25109184c71bdad752c8312a8623239686a9a2071e8825f20acb8f2198c3f659 AS final-stage

WORKDIR /

COPY --from=build-stage /app/target/release/cloudflare-dyndns /usr/local/bin/cloudflare-dyndns

USER nobody:nobody

ENTRYPOINT ["cloudflare-dyndns"]

#
# END final-stage
###############################################################################
