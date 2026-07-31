.DEFAULT_GOAL := help

.PHONY: help fmt fmt-check clippy test check build release install-local tag

BINS := cc-switch cx-switch
PREFIX ?= $(HOME)/.local
VERSION := $(shell cargo metadata --no-deps --format-version 1 | sed -n 's/.*"version":"\([^"]*\)".*/\1/p')

help:
	@printf '%s\n' 'Available targets:'
	@printf '  %-14s %s\n' 'fmt' 'Format Rust sources'
	@printf '  %-14s %s\n' 'fmt-check' 'Check Rust formatting'
	@printf '  %-14s %s\n' 'clippy' 'Run clippy with warnings as errors'
	@printf '  %-14s %s\n' 'test' 'Run tests'
	@printf '  %-14s %s\n' 'check' 'Run fmt-check, clippy, and test'
	@printf '  %-14s %s\n' 'build' 'Build debug binary'
	@printf '  %-14s %s\n' 'release' 'Build release binary'
	@printf '  %-14s %s\n' 'install-local' 'Build and install to $(PREFIX)/bin'
	@printf '  %-14s %s\n' 'tag' 'Create local git tag v$(VERSION)'

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test

check: fmt-check clippy test

build:
	cargo build

release:
	cargo build --release
	@for bin in $(BINS); do printf 'Built %s\n' "target/release/$$bin"; done

install-local: release
	install -d '$(PREFIX)/bin'
	@for bin in $(BINS); do \
		install -m 0755 "target/release/$$bin" "$(PREFIX)/bin/$$bin"; \
		printf 'Installed %s\n' "$(PREFIX)/bin/$$bin"; \
	done

tag:
	git tag 'v$(VERSION)'
