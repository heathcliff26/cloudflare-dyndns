SHELL := bash

REPOSITORY ?= localhost
CONTAINER_NAME ?= cloudflare-dyndns
TAG ?= latest

default: build

build:
	hack/build.sh

image:
	podman build -t $(REPOSITORY)/$(CONTAINER_NAME):$(TAG) .

test:
	go test -v -race ./...

update-deps:
	hack/update-deps.sh

coverprofile:
	hack/coverprofile.sh

lint:
	golangci-lint run -v

package-openwrt:
	hack/build-package-openwrt.sh

clean:
	rm -rf bin coverprofiles packages/openwrt/*.tar.gz packages/openwrt/control/control

.PHONY: \
	default \
	build \
	image \
	test \
	update-deps \
	coverprofile \
	lint \
	package-openwrt \
	clean \
	$(NULL)
