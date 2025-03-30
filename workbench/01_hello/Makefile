.DEFAULT_GOAL := help

.PHONY: lint
lint: ## Run Linter
	cargo check
	cargo fmt -- --check
	cargo clippy

.PHONY: fmt
fmt: ## Run formatter
	cargo fmt

.PHONY: test
test: ## Run tests
	cargo test

.PHONY: setup
setup: rustup component add rust-analyzer

.PHONY: help
help: ## Show options
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
