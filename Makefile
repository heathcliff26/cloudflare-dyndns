SHELL := bash

REPOSITORY ?= localhost
CONTAINER_NAME ?= cloudflare-dyndns
TAG ?= latest

build: ## Build the binary
	hack/build.sh

image: ## Build the container image
	podman build -t $(REPOSITORY)/$(CONTAINER_NAME):$(TAG) .

test: ## Run unit-tests
	go test -v -race -coverprofile=coverprofile.out ./...

update-deps: ## Update dependencies
	hack/update-deps.sh

coverprofile: ## Generate cover profile
	hack/coverprofile.sh

lint: ## Run linter
	golangci-lint run -v

fmt: ## Format code
	gofmt -s -w ./cmd ./pkg

validate: ## Validate that all generated files are up to date
	hack/validate.sh

package-openwrt: ## Build Package for OpenWRT
	hack/build-package-openwrt.sh

gosec: ## Scan code for vulnerabilities using gosec
	gosec ./...

clean: ## Clean build artifacts
	rm -rf bin coverprofiles coverprofile.out packages/openwrt/*.tar.gz packages/openwrt/control/control

help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "%-20s %s\n", $$1, $$2}'
	@echo ""
	@echo "Run 'make <target>' to execute a specific target."

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
	gosec \
	clean \
	help \
	$(NULL)
