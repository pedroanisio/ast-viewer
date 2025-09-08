# AST Visualizer Makefile

.PHONY: help install test run clean docker lint format check-env

# Colors for terminal output
RED=\033[0;31m
GREEN=\033[0;32m
YELLOW=\033[1;33m
NC=\033[0m # No Color

help: ## Show this help message
	@echo "$(GREEN)AST Visualizer Development Commands$(NC)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(YELLOW)%-15s$(NC) %s\n", $$1, $$2}'

install: ## Install dependencies
	@echo "$(GREEN)Installing dependencies...$(NC)"
	pip install -r requirements.txt
	@echo "$(GREEN)✓ Dependencies installed$(NC)"

check-env: ## Check environment configuration
	@echo "$(GREEN)Checking environment...$(NC)"
	@if [ -z "$$SECRET_KEY" ]; then \
		echo "$(RED)✗ SECRET_KEY not set$(NC)"; \
		echo "  Please set: export SECRET_KEY='your-secret-key'"; \
		exit 1; \
	else \
		echo "$(GREEN)✓ SECRET_KEY is set$(NC)"; \
	fi
	@python -c "import redis; print('$(GREEN)✓ Redis library available$(NC)')" 2>/dev/null || echo "$(YELLOW)⚠ Redis library not available$(NC)"

test: ## Run tests
	@echo "$(GREEN)Running tests...$(NC)"
	pytest -v
	@echo "$(GREEN)✓ Tests completed$(NC)"

test-cov: ## Run tests with coverage
	@echo "$(GREEN)Running tests with coverage...$(NC)"
	pytest --cov=. --cov-report=html --cov-report=term
	@echo "$(GREEN)✓ Coverage report generated$(NC)"

lint: ## Run linting
	@echo "$(GREEN)Running linting...$(NC)"
	pylint *.py || true
	@echo "$(GREEN)✓ Linting completed$(NC)"

format: ## Format code
	@echo "$(GREEN)Formatting code...$(NC)"
	black *.py
	@echo "$(GREEN)✓ Code formatted$(NC)"

run: check-env ## Run the application
	@echo "$(GREEN)Starting AST Visualizer...$(NC)"
	python run.py

dev: check-env ## Run in development mode
	@echo "$(GREEN)Starting in development mode...$(NC)"
	FLASK_DEBUG=true python run.py

docker: ## Build Docker image
	@echo "$(GREEN)Building Docker image...$(NC)"
	docker build -t ast-visualizer .
	@echo "$(GREEN)✓ Docker image built$(NC)"

docker-run: docker ## Run Docker container
	@echo "$(GREEN)Running Docker container...$(NC)"
	docker run -p 5000:5000 -e SECRET_KEY="dev-secret-key" ast-visualizer

docker-dev: ## Run with docker-compose for development
	@echo "$(GREEN)Starting development environment...$(NC)"
	docker-compose up -d
	@echo "$(GREEN)✓ Development environment started$(NC)"
	@echo "  Access at: http://localhost:5000"

docker-stop: ## Stop docker-compose services
	@echo "$(GREEN)Stopping development environment...$(NC)"
	docker-compose down
	@echo "$(GREEN)✓ Development environment stopped$(NC)"

clean: ## Clean up temporary files
	@echo "$(GREEN)Cleaning up...$(NC)"
	find . -type f -name "*.pyc" -delete
	find . -type d -name "__pycache__" -delete
	rm -rf .pytest_cache
	rm -rf htmlcov
	rm -rf .coverage
	rm -f ast_visualizer.log
	@echo "$(GREEN)✓ Cleanup completed$(NC)"

init: ## Initialize development environment
	@echo "$(GREEN)Initializing development environment...$(NC)"
	@if [ ! -f .env ]; then \
		cp env.example .env; \
		echo "$(YELLOW)Created .env file from template$(NC)"; \
		echo "$(YELLOW)Please edit .env with your configuration$(NC)"; \
	fi
	make install
	@echo "$(GREEN)✓ Development environment initialized$(NC)"
	@echo ""
	@echo "$(YELLOW)Next steps:$(NC)"
	@echo "  1. Edit .env file with your settings"
	@echo "  2. Export SECRET_KEY: export SECRET_KEY='your-secret-key'"
	@echo "  3. Run: make dev"

# Default target
.DEFAULT_GOAL := help
