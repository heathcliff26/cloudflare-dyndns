###############################################################################
# BEGIN build-stage
# Compile the binary
FROM docker.io/library/rust:1.99.0-alpine AS build-stage

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

RUN cargo rustc --release -- -C relocation-model=static

#
# END build-stage
###############################################################################

###############################################################################
# BEGIN final-stage
# Create final docker image
FROM docker.io/library/alpine:3.24.4@sha256:93b41ef40b1677fce8fcc64ff4af09daddfacf5040afde618bc46c1e2d1cc7d4 AS final-stage

WORKDIR /

COPY --from=build-stage /app/target/release/cloudflare-dyndns /usr/local/bin/cloudflare-dyndns

USER nobody:nobody

ENTRYPOINT ["cloudflare-dyndns"]

#
# END final-stage
###############################################################################
