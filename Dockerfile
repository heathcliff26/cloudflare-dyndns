###############################################################################
# BEGIN build-stage
# Compile the binary
FROM --platform=$BUILDPLATFORM docker.io/library/golang:1.22.5@sha256:86a3c48a61915a8c62c0e1d7594730399caa3feb73655dfe96c7bc17710e96cf AS build-stage

ARG BUILDPLATFORM
ARG TARGETARCH

WORKDIR /app

COPY vendor ./vendor
COPY go.mod go.sum ./
COPY cmd ./cmd
COPY pkg ./pkg

RUN CGO_ENABLED=0 GOOS=linux GOARCH="${TARGETARCH}" go build -ldflags="-w -s" -o /cloudflare-dyndns ./cmd/

#
# END build-stage
###############################################################################

###############################################################################
# BEGIN test-stage
# Run the tests in the container
FROM docker.io/library/golang:1.22.5@sha256:86a3c48a61915a8c62c0e1d7594730399caa3feb73655dfe96c7bc17710e96cf AS test-stage

WORKDIR /app

COPY --from=build-stage /app /app
# Not needed for testing, but needed for later stage
COPY --from=build-stage /cloudflare-dyndns /

RUN go test -v ./...

#
# END test-stage
###############################################################################

###############################################################################
# BEGIN combine-stage
# Combine all outputs, to enable single layer copy for the final image
FROM scratch AS combine-stage

COPY --from=test-stage /cloudflare-dyndns /

COPY --from=test-stage /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

#
# END combine-stage
###############################################################################

###############################################################################
# BEGIN final-stage
# Create final docker image
FROM scratch AS final-stage

WORKDIR /

COPY --from=combine-stage / /

EXPOSE 8080

USER 1001

ENTRYPOINT ["/cloudflare-dyndns"]

#
# END final-stage
###############################################################################
