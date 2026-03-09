.DEFAULT_GOAL := help

COMPOSE := docker compose
SERVER_DIR := server
ROOT_ENV_FILE := .env
SQLX := cargo sqlx
INFRA_SERVICES := postgres redis
FULL_STACK_SERVICES := postgres redis server autoheal
DATABASE_URL ?= $(shell awk -F= '/^DATABASE__URL=/{sub(/^DATABASE__URL=/, ""); print; exit}' $(ROOT_ENV_FILE))

.PHONY: help up compose-up down ps logs run dev migration-up migration-down migration-create migration-info fmt fmt-check clippy test check

help: ## Show available commands
	@awk 'BEGIN {FS = ":.*## "}; /^[a-zA-Z0-9_.-]+:.*## / {printf "  %-16s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

up: ## Start Postgres and Redis in the background
	$(COMPOSE) up -d $(INFRA_SERVICES)

compose-up: ## Build and start the full containerized stack
	$(COMPOSE) up -d --build $(FULL_STACK_SERVICES)

down: ## Stop local development containers
	$(COMPOSE) down

ps: ## Show container status
	$(COMPOSE) ps

logs: ## Follow container logs
	$(COMPOSE) logs -f

run: ## Run the Rust API with Bacon auto-reload
	cd $(SERVER_DIR) && bacon

dev: up run ## Start dependencies and enter the Bacon hot-reload loop

migration-up: ## Apply pending database migrations with sqlx-cli
	cd $(SERVER_DIR) && DATABASE_URL='$(DATABASE_URL)' $(SQLX) migrate run

migration-down: ## Revert the latest database migration with sqlx-cli
	cd $(SERVER_DIR) && DATABASE_URL='$(DATABASE_URL)' $(SQLX) migrate revert

migration-create: ## Create a new reversible migration with NAME=<name>
	@test -n "$(NAME)" || (echo "NAME is required, e.g. make migration-create NAME=add_orders_table" && exit 1)
	cd $(SERVER_DIR) && $(SQLX) migrate add -r $(NAME)

migration-info: ## Show database migration status with sqlx-cli
	cd $(SERVER_DIR) && DATABASE_URL='$(DATABASE_URL)' $(SQLX) migrate info

fmt: ## Format Rust code
	cd $(SERVER_DIR) && cargo fmt --all

fmt-check: ## Check Rust formatting
	cd $(SERVER_DIR) && cargo fmt --check

clippy: ## Run clippy with warnings denied
	cd $(SERVER_DIR) && cargo clippy --workspace --all-targets --all-features -- -D warnings

test: ## Run Rust tests
	cd $(SERVER_DIR) && cargo test --workspace

check: fmt-check clippy test ## Run the full Rust validation suite
