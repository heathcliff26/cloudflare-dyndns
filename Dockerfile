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
FROM docker.io/library/alpine:3.23.4@sha256:5b10f432ef3da1b8d4c7eb6c487f2f5a8f096bc91145e68878dd4a5019afde11 AS final-stage

WORKDIR /

COPY --from=build-stage /app/target/release/cloudflare-dyndns /usr/local/bin/cloudflare-dyndns

USER nobody:nobody

ENTRYPOINT ["cloudflare-dyndns"]

#
# END final-stage
###############################################################################
