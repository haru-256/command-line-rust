.DEFAULT_GOAL := help

.PHONY: test
test: ## Execute tests
	RUST_LOG=info cargo test

.PHONY: lint
lint: ## Run Linter
	cargo check
	cargo clippy

.PHONY: fmt
fmt: ## Run formatter
	cargo fmt

.PHONY: doc
doc: ## Generate documentation
	cargo doc --no-deps --open --document-private-items

.PHONY: help
help: ## Show options
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
