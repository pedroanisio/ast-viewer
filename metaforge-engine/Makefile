# MetaForge Makefile

.PHONY: help up down logs clean clean-light clean-deep restart status db-shell redis-shell pgadmin

# Default target
help:
	@echo "MetaForge Docker Management"
	@echo "=========================="
	@echo "Available commands:"
	@echo "  up          - Start all services"
	@echo "  down        - Stop all services"
	@echo "  restart     - Restart all services"
	@echo "  logs        - Show logs for all services"
	@echo "  status      - Show status of all services"
	@echo "  clean-light - Clean Python artifacts and cache"
	@echo "  clean       - Clean containers, volumes, and artifacts"
	@echo "  clean-deep  - Deep clean (removes everything including images)"
	@echo "  db-shell    - Connect to PostgreSQL shell"
	@echo "  redis-shell - Connect to Redis shell"
	@echo "  pgadmin     - Start pgAdmin (with tools profile)"
	@echo "  setup       - Initial setup (copy env file)"

# Start services
up:
	docker compose up -d

# Stop services
down:
	docker compose down

# Restart services
restart:
	docker compose restart

# Show logs
logs:
	docker compose logs -f

# Show status
status:
	docker compose ps

# Light cleanup - Python artifacts and cache (safe)
clean-light:
	@echo "🧹 Cleaning Python artifacts and cache..."
	find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete 2>/dev/null || true
	find . -type f -name "*.pyo" -delete 2>/dev/null || true
	find . -type d -name "*.egg-info" -exec rm -rf {} + 2>/dev/null || true
	rm -rf .pytest_cache
	rm -rf .coverage
	rm -rf htmlcov/
	rm -rf .mypy_cache
	rm -rf .ruff_cache
	rm -rf build/
	rm -rf dist/
	@echo "✅ Light cleanup completed"

# Standard cleanup - containers, volumes, and artifacts
clean:
	@echo "🧹 Starting standard cleanup..."
	@$(MAKE) clean-light
	@echo "🐳 Stopping and removing containers and volumes..."
	docker compose down -v --remove-orphans
	@echo "⚠️  This will remove unused Docker resources (containers, networks, images)."
	@echo "💡 Use 'make clean-deep' for more aggressive cleanup including images."
	@read -p "Continue with Docker system prune? (y/N): " confirm && [ "$$confirm" = "y" ] || [ "$$confirm" = "Y" ] || (echo "❌ Cleanup cancelled"; exit 1)
	docker system prune -f
	@echo "✅ Standard cleanup completed"

# Deep cleanup - removes everything including Docker images
clean-deep:
	@echo "🧹 Starting deep cleanup..."
	@$(MAKE) clean-light
	@echo "🐳 Stopping and removing all containers, volumes, and networks..."
	docker compose down -v --remove-orphans
	@echo "⚠️  WARNING: This will remove ALL unused Docker resources including:"
	@echo "   - All stopped containers"
	@echo "   - All unused networks"
	@echo "   - All unused images (not just dangling)"
	@echo "   - All unused build cache"
	@echo ""
	@read -p "Are you absolutely sure? This action cannot be undone! (y/N): " confirm && [ "$$confirm" = "y" ] || [ "$$confirm" = "Y" ] || (echo "❌ Deep cleanup cancelled"; exit 1)
	docker system prune -af --volumes
	@echo "✅ Deep cleanup completed - all Docker resources removed"

# Database shell
db-shell:
	docker compose exec postgres psql -U metaforge -d metaforge

# Redis shell
redis-shell:
	docker compose exec redis redis-cli

# Start pgAdmin
pgadmin:
	docker compose --profile tools up -d pgadmin

# Initial setup
setup:
	@if [ ! -f .env ]; then \
		cp env.example .env; \
		echo "Created .env file from env.example"; \
		echo "Please edit .env with your configuration"; \
	else \
		echo ".env file already exists"; \
	fi

# Development setup
dev-setup: setup up
	@echo "Waiting for database to be ready..."
	@sleep 10
	@echo "Development environment ready!"
	@echo "Database: postgresql://metaforge:metaforge_password@localhost:5432/metaforge"
	@echo "Redis: redis://localhost:6379/0"
	@echo "pgAdmin: http://localhost:8080 (admin@metaforge.local / admin_password)"

# Production setup
prod-setup: setup
	@echo "Production setup requires manual configuration of .env file"
	@echo "Please review and update .env with production values"
