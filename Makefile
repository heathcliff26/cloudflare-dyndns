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
	go test -v -race -coverprofile=coverprofile.out ./...

update-deps:
	hack/update-deps.sh

coverprofile:
	hack/coverprofile.sh

lint:
	golangci-lint run -v

fmt:
	gofmt -s -w ./cmd ./pkg

validate:
	hack/validate.sh

package-openwrt:
	hack/build-package-openwrt.sh

clean:
	rm -rf bin coverprofiles coverprofile.out packages/openwrt/*.tar.gz packages/openwrt/control/control

.PHONY: \
	default \
	build \
	image \
	test \
	update-deps \
	coverprofile \
	lint \
	fmt \
	validate \
	package-openwrt \
	clean \
	$(NULL)
