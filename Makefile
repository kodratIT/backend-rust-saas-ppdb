.PHONY: help dev build test clean migrate migrate-up migrate-down seed fmt check

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

dev: ## Run development server with auto-reload
	cargo watch -x run

build: ## Build release binary
	cargo build --release

test: ## Run tests
	cargo test

clean: ## Clean build artifacts
	cargo clean

migrate: ## Run database migrations
	sqlx migrate run

migrate-up: ## Run database migrations (alias)
	sqlx migrate run

migrate-down: ## Revert last migration
	sqlx migrate revert

seed: ## Seed database with sample data
	psql $(DATABASE_URL) -f migrations/010_seed_data.sql

fmt: ## Format code
	cargo fmt

check: ## Check code (fmt + clippy)
	cargo fmt --check
	cargo clippy -- -D warnings

install-tools: ## Install development tools
	cargo install cargo-watch
	cargo install sqlx-cli --no-default-features --features postgres

setup: ## Setup project (copy .env, install tools)
	cp .env.example .env
	@echo "Please edit .env file with your configuration"
	@echo "Then run: make install-tools"
