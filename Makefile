.DEFAULT_GOAL := help

COMPOSE := docker compose
SERVER_DIR := server

.PHONY: help up down ps logs run dev fmt fmt-check clippy test check

help: ## Show available commands
	@awk 'BEGIN {FS = ":.*## "}; /^[a-zA-Z0-9_.-]+:.*## / {printf "  %-10s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

up: ## Start Postgres and Redis in the background
	$(COMPOSE) up -d

down: ## Stop local development containers
	$(COMPOSE) down

ps: ## Show container status
	$(COMPOSE) ps

logs: ## Follow container logs
	$(COMPOSE) logs -f

run: ## Run the Rust API with Bacon auto-reload
	cd $(SERVER_DIR) && bacon

dev: up run ## Start dependencies and enter the Bacon hot-reload loop

fmt: ## Format Rust code
	cd $(SERVER_DIR) && cargo fmt --all

fmt-check: ## Check Rust formatting
	cd $(SERVER_DIR) && cargo fmt --check

clippy: ## Run clippy with warnings denied
	cd $(SERVER_DIR) && cargo clippy --all-targets --all-features -- -D warnings

test: ## Run Rust tests
	cd $(SERVER_DIR) && cargo test

check: fmt-check clippy test ## Run the full Rust validation suite
