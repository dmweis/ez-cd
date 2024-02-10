TARGET_HOST ?= homepi
TARGET_USERNAME ?= $$USER
TARGET_HOST_USER ?= $(TARGET_USERNAME)@$(TARGET_HOST)

.PHONY: build
build:
	cargo build --release

.PHONY: build-deb
build-deb: build
	cargo deb --no-build

.PHONY: install-dependencies
install-dependencies:
	cargo install cargo-deb

.PHONY: install-cli
install-cli:
	cargo build --release --bin ez-cd-cli
	cargo deb --no-build --fast --variant cli
	sudo dpkg -i target/debian/ez-cd-cli*.deb

.PHONY: build-docker
build-docker:
	rm -rf docker_out
	mkdir docker_out
	DOCKER_BUILDKIT=1 docker build --tag ez-cd-builder --file Dockerfile --output type=local,dest=docker_out .

.PHONY: push-docker
push-docker: build-docker
	rsync -avz --delete docker_out/* $(TARGET_HOST_USER):/home/$(TARGET_USERNAME)/ez-cd-service
