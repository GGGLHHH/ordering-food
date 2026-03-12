.DEFAULT_GOAL := help

COMPOSE := docker compose
SERVER_DIR := server
ROOT_ENV_FILE := .env
SQLX := cargo sqlx
SQLX_MIGRATIONS_DIR := crates/identity-infrastructure-sqlx/migrations
INFRA_SERVICES := postgres redis dbhub
FULL_STACK_SERVICES := $(INFRA_SERVICES) server frontend nginx autoheal
DATABASE_URL ?= $(shell awk -F= '/^DATABASE__URL=/{sub(/^DATABASE__URL=/, ""); print; exit}' $(ROOT_ENV_FILE))
GENERATED_API_DIR ?= $(shell awk -F= '/^GENERATED_API_DIR=/{sub(/^GENERATED_API_DIR=/, ""); print; exit}' $(ROOT_ENV_FILE))

.PHONY: help up compose-up down ps logs run dev export-ts migration-up migration-down migration-create migration-info fmt fmt-check clippy test check

help: ## Show available commands
	@awk 'BEGIN {FS = ":.*## "}; /^[a-zA-Z0-9_.-]+:.*## / {printf "  %-16s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

up: ## Start Postgres, Redis, and DBHub in the background
	$(COMPOSE) up -d $(INFRA_SERVICES)

compose-up: ## Build and start the full containerized stack including DBHub
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

export-ts: ## Export frontend TypeScript bindings from API contracts
	@test -n "$(GENERATED_API_DIR)" || (echo "GENERATED_API_DIR is required, e.g. GENERATED_API_DIR=../frontend/src/contracts/generated" && exit 1)
	cd $(SERVER_DIR) && GENERATED_API_DIR='$(GENERATED_API_DIR)' cargo run -p ordering-food-api --bin export-ts-bindings

migration-up: ## Apply pending database migrations with sqlx-cli
	cd $(SERVER_DIR) && DATABASE_URL='$(DATABASE_URL)' $(SQLX) migrate run --source $(SQLX_MIGRATIONS_DIR)

migration-down: ## Revert the latest database migration with sqlx-cli
	cd $(SERVER_DIR) && DATABASE_URL='$(DATABASE_URL)' $(SQLX) migrate revert --source $(SQLX_MIGRATIONS_DIR)

migration-create: ## Create a new reversible migration with NAME=<name>
	@test -n "$(NAME)" || (echo "NAME is required, e.g. make migration-create NAME=add_orders_table" && exit 1)
	cd $(SERVER_DIR) && $(SQLX) migrate add -r --source $(SQLX_MIGRATIONS_DIR) $(NAME)

migration-info: ## Show database migration status with sqlx-cli
	cd $(SERVER_DIR) && DATABASE_URL='$(DATABASE_URL)' $(SQLX) migrate info --source $(SQLX_MIGRATIONS_DIR)

fmt: ## Format Rust code
	cd $(SERVER_DIR) && cargo fmt --all

fmt-check: ## Check Rust formatting
	cd $(SERVER_DIR) && cargo fmt --all --check

clippy: ## Run clippy with warnings denied
	cd $(SERVER_DIR) && cargo clippy --workspace --all-targets --all-features -- -D warnings

test: ## Run Rust tests
	cd $(SERVER_DIR) && cargo test --workspace

check: fmt-check clippy test ## Run the full Rust validation suite
