.DEFAULT_GOAL := help
SHELL := /usr/bin/env bash

# Shared dev targets. Works under Git Bash (Windows), macOS, and Linux.

.PHONY: help setup dev backend-dev frontend-dev test backend-test frontend-test \
        lint fmt audit migrate db-up db-down db-reset mail-up mail-down clean

help: ## Show this help.
	@awk 'BEGIN {FS = ":.*##"; printf "Targets:\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

setup: ## Install backend + frontend dev dependencies.
	cd backend && cargo fetch
	cd frontend && pnpm install

dev: ## Run backend + frontend together (requires `concurrently` or two terminals).
	@echo "Run 'make backend-dev' and 'make frontend-dev' in separate terminals."

backend-dev: ## Run the API server with hot-reload (requires cargo-watch).
	cd backend && cargo watch -x 'run -p civitas-api --bin civitas-api'

frontend-dev: ## Run the SvelteKit dev server.
	cd frontend && pnpm dev

test: backend-test frontend-test ## Run all tests.

backend-test: ## Run Rust tests.
	cd backend && cargo test --workspace --all-features

frontend-test: ## Run frontend tests.
	cd frontend && pnpm test

lint: ## Run clippy + eslint.
	cd backend && cargo clippy --workspace --all-targets --all-features -- -D warnings
	cd frontend && pnpm lint

fmt: ## Format Rust + TS sources.
	cd backend && cargo fmt --all
	cd frontend && pnpm format

audit: ## Audit dependencies for known vulnerabilities.
	cd backend && cargo audit
	cd frontend && pnpm audit

migrate: ## Apply pending SQLx migrations.
	cd backend && sqlx migrate run

db-up: ## Start local Postgres via docker compose.
	docker compose up -d db

db-down: ## Stop local Postgres.
	docker compose stop db

db-reset: ## Drop, recreate, and re-migrate the dev database.
	cd backend && sqlx database drop -y && sqlx database create && sqlx migrate run

mail-up: ## Start the dev mail catcher (Mailpit, UI on http://localhost:8025).
	docker compose up -d mailpit

mail-down: ## Stop the dev mail catcher.
	docker compose stop mailpit

clean: ## Remove build artifacts.
	cd backend && cargo clean
	cd frontend && rm -rf .svelte-kit build node_modules/.vite
