# MetaForge Codebase

This repository contains two distinct projects:

## 1. MetaForge Engine (Rust)
Located in `metaforge-engine/`.
A Rust-based semantic code analysis and migration engine.
- **Build**: `cargo build` (requires Rust) or via Docker.
- **Docker**: `docker compose build engine`

## 2. AST Visualizer (Python)
Located in `ast-visualizer/`.
A Python Flask application for visualizing Abstract Syntax Trees.
- **Run**: `python src/run.py` (requires dependencies) or via Docker.
- **Docker**: `docker compose up visualizer`

## Running with Docker Compose
To run the entire stack (Engine, Visualizer, Redis, Postgres):
```bash
docker compose up --build
```
