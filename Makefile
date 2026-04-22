SHELL := bash

REPOSITORY ?= localhost
CONTAINER_NAME ?= cloudflare-dyndns
TAG ?= latest

# Build the binary
build:
	hack/build.sh

# Build all artifacts used for release, except the container images
release:
	hack/release.sh

# Build the container image
image:
	podman build --build-arg="CI_COMMIT_SHA=$$(git rev-parse HEAD)" -t $(REPOSITORY)/$(CONTAINER_NAME):$(TAG) .

# Run cargo test
test:
	cargo test

# Run e2e tests
test-e2e:
	cargo test --features e2e --test e2e-testsuite

# Generate cover profile
coverprofile:
	hack/coverprofile.sh

# Build the docs, fail on warnings
doc:
	RUSTDOCFLAGS='--deny warnings' cargo doc --no-deps

# Run linter (clippy)
lint:
	cargo clippy -- --deny warnings

# Lint the helm charts
lint-helm:
	helm lint manifests/helm/

# Format the code
fmt:
	cargo fmt

# Validate that all generated files are up to date
validate:
	hack/validate.sh

# Validate the appstream metainfo file
validate-metainfo:
	appstreamcli validate io.github.heathcliff26.cloudflare-dyndns.metainfo.xml

# Build rpm with code in current workdir using packit
packit:
	packit build locally

# Build rpm of upstream code using packit + mock
packit-mock:
	packit build in-mock --resultdir tmp
	rm *.src.rpm

# Clean build artifacts
clean:
	hack/clean.sh

# Show this help message
help:
	@echo "Available targets:"
	@echo ""
	@awk '/^#/{c=substr($$0,3);next}c&&/^[[:alpha:]][[:alnum:]_-]+:/{print substr($$1,1,index($$1,":")),c}1{c=0}' $(MAKEFILE_LIST) | column -s: -t
	@echo ""
	@echo "Run 'make <target>' to execute a specific target."

.PHONY: \
	build \
	release \
	image \
	test \
	test-e2e \
	coverprofile \
	doc \
	lint \
	lint-helm \
	fmt \
	validate \
	validate-metainfo \
	packit \
	packit-mock \
	clean \
	help \
	$(NULL)
