# Makefile for Manga Manager API

# Variables
APP_NAME = manga-sync
IMAGE_NAME = manga-sync
DOCKER_PORT = 7783
LOCAL_PORT = 7783

.PHONY: help run test docker-build docker-run openapi-update clean

help: ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

run: ## Run the application locally
	cargo run

test: ## Run tests
	cargo test

docker-build: ## Build the Docker image
	docker build -t $(IMAGE_NAME) .

docker-run: ## Run the Docker container
	mkdir -p secret
	docker run -p $(DOCKER_PORT):$(LOCAL_PORT) -v $$(pwd)/secret:/app/secret $(IMAGE_NAME)

openapi-update: ## Update openapi.yml automatically from code
	cargo run --bin gen_openapi > openapi.yml

clean: ## Clean build artifacts
	cargo clean
