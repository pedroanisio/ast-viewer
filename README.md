# AST Repository Visualizer

A modern web-based AST (Abstract Syntax Tree) visualization system for Python repositories with interactive graphs and comprehensive analysis.

## 🚀 Features

- **Interactive Visualizations**: Network graphs, treemaps, and complexity charts
- **File AST Explorer**: Click on any file to explore its detailed AST structure with hierarchical navigation, search, and node details
- **Comprehensive Analysis**: AST parsing, complexity metrics, import dependencies
- **Security First**: Input validation, rate limiting, path traversal protection
- **High Performance**: Parallel processing, Redis caching, optimized rendering
- **Modern UI**: Responsive design with real-time updates
- **Export Capabilities**: JSON export for further analysis and individual file AST export

## 🛠 Installation

### Prerequisites

- Python 3.11+
- Git
- Redis (optional - will use memory cache if unavailable)

### Quick Start

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd ast-viewer
   ```

2. **Install dependencies**
   ```bash
   pip install -r requirements.txt
   ```

3. **Configure environment**
   ```bash
   cp env.example .env
   # Edit .env file with your configuration
   ```

4. **Set required environment variables**
   ```bash
   export SECRET_KEY="your-super-secret-key"
   ```

5. **Run the application**
   ```bash
   python run.py
   ```

6. **Access the web interface**
   ```
   http://localhost:5000
   ```

### Docker Setup (Recommended)

1. **Using Docker Compose**
   ```bash
   docker-compose up -d
   ```

2. **Using Docker only**
   ```bash
   docker build -t ast-visualizer .
   docker run -p 5000:5000 -e SECRET_KEY="your-secret" ast-visualizer
   ```

## 📖 Usage

### Web Interface

1. **Navigate to the main page** at `http://localhost:5000`
2. **Choose input method**:
   - **Local Repository**: Enter absolute path to Python project
   - **Git URL**: Enter public repository URL (GitHub, GitLab, etc.)
3. **Configure analysis options**:
   - Include test files
   - Deep complexity analysis
   - Cache results
   - Generate summary report
4. **Click "Analyze Repository"**
5. **View interactive visualizations**

### API Endpoints

#### Analyze Repository
```bash
POST /api/analyze
Content-Type: application/json

{
  "path": "/path/to/repository"
  // OR
  "url": "https://github.com/user/repo.git"
}
```

#### Get Visualization Data
```bash
GET /api/visualize/{analysis_id}
```

#### Search Nodes
```bash
GET /api/search/{analysis_id}?q=function_name&type=FunctionDef
```

#### Export Analysis
```bash
GET /api/export/{analysis_id}
```

#### Health Check
```bash
GET /health
```

## 🏗 Architecture

### Components

- **`app.py`**: Flask web application with REST API
- **`ast_analyzer.py`**: Core AST analysis engine
- **`cache_manager.py`**: Redis/memory caching system
- **`templates/`**: HTML templates for web interface
- **`tests/`**: Comprehensive test suite

### Security Features

- **Input validation**: Path traversal prevention, URL validation
- **Rate limiting**: Configurable request limits
- **Security headers**: CSRF, XSS, and clickjacking protection
- **Environment isolation**: Secure configuration management

### Performance Optimizations

- **Parallel processing**: Multi-threaded file analysis
- **Smart caching**: Redis with memory fallback
- **Lazy loading**: Progressive visualization rendering
- **Size limits**: File and repository size restrictions

## 🧪 Testing

### Run All Tests
```bash
pytest
```

### Run Specific Test Categories
```bash
# Unit tests
pytest tests/test_cache_manager.py
pytest tests/test_ast_analyzer.py

# Integration tests
pytest tests/test_app.py

# With coverage
pytest --cov=. --cov-report=html
```

### Test Coverage
The project maintains comprehensive test coverage:
- Cache manager functionality
- AST analysis engine
- Flask API endpoints
- Integration workflows

## ⚙️ Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `SECRET_KEY` | Yes | - | Flask session secret key |
| `FLASK_HOST` | No | `0.0.0.0` | Server host |
| `FLASK_PORT` | No | `5000` | Server port |
| `FLASK_DEBUG` | No | `False` | Debug mode |
| `REDIS_URL` | No | `redis://localhost:6379` | Redis connection |
| `CORS_ORIGINS` | No | `http://localhost:5000` | Allowed origins |
| `RATE_LIMIT_REQUESTS` | No | `10` | Requests per window |
| `RATE_LIMIT_WINDOW` | No | `60` | Rate limit window (seconds) |

### Security Configuration

- Set strong `SECRET_KEY` in production
- Configure appropriate `CORS_ORIGINS`
- Adjust rate limits based on usage
- Enable HTTPS in production

## 📊 Visualization Types

### Network Graph
- **Interactive node-link diagram**
- File dependencies and imports
- Complexity-based coloring
- Zoom, pan, and selection

### Treemap
- **Hierarchical file size visualization**
- Lines of code proportional sizing
- Complexity-based coloring
- Interactive exploration

### Complexity Charts
- **Bar charts of file complexity**
- Sortable metrics
- Drill-down capabilities
- Export functionality

### Metrics Dashboard
- **Summary statistics**
- Top complex files
- Largest files
- Import relationships
- Size distribution

## 🔧 Development

### Project Structure
```
ast-viewer/
├── app.py                 # Flask application
├── ast_analyzer.py        # AST analysis engine
├── cache_manager.py       # Caching system
├── run.py                 # Production startup
├── requirements.txt       # Dependencies
├── Dockerfile            # Container config
├── docker-compose.yml    # Development setup
├── env.example           # Environment template
├── templates/            # HTML templates
│   ├── index.html
│   ├── visualization.html
│   └── error.html
├── tests/                # Test suite
│   ├── test_app.py
│   ├── test_ast_analyzer.py
│   └── test_cache_manager.py
├── uploads/              # File uploads
└── logs/                 # Application logs
```

### Development Setup
```bash
# Install development dependencies
pip install -r requirements.txt

# Install pre-commit hooks (if using)
pre-commit install

# Run in development mode
export FLASK_DEBUG=true
python run.py
```

## 📈 Performance

### Optimization Features
- **Multi-threading**: Parallel file processing
- **Caching**: Redis-based result caching
- **Streaming**: Large dataset handling
- **Pagination**: UI performance optimization

### Limits
- Max file size: 10MB per file
- Max repository size: 500MB total
- Max files: 10,000 files
- Request timeout: 5 minutes

## 🐛 Troubleshooting

### Common Issues

1. **"SECRET_KEY environment variable is required"**
   ```bash
   export SECRET_KEY="your-secret-key"
   ```

2. **Redis connection errors**
   - Check Redis is running: `redis-cli ping`
   - Application will use memory cache fallback

3. **Git clone failures**
   - Ensure git is installed and accessible
   - Check repository URL is valid and accessible
   - Private repositories require authentication setup

4. **Large repository timeouts**
   - Git operations may timeout naturally for very large repos
   - Use smaller repositories for testing
   - Consider using shallow clones (enabled by default)

5. **Permission errors**
   - Ensure read permissions on repository paths
   - Check file ownership and access rights

### Logging
- Application logs to `ast_visualizer.log`
- Set `LOG_LEVEL=DEBUG` for detailed logging
- Check browser console for frontend errors

## 🤝 Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Run tests (`pytest`)
4. Commit changes (`git commit -m 'Add amazing feature'`)
5. Push to branch (`git push origin feature/amazing-feature`)
6. Open Pull Request

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🚀 Deployment

### Production Checklist

- [ ] Set strong `SECRET_KEY`
- [ ] Configure Redis for caching
- [ ] Set appropriate rate limits
- [ ] Enable HTTPS
- [ ] Configure logging
- [ ] Set up monitoring
- [ ] Configure backup strategy

### Docker Deployment
```bash
# Build production image
docker build -t ast-visualizer:latest .

# Run with production settings
docker run -d \
  -p 5000:5000 \
  -e SECRET_KEY="production-secret-key" \
  -e FLASK_DEBUG=false \
  -e REDIS_URL="redis://production-redis:6379" \
  --name ast-visualizer \
  ast-visualizer:latest
```

## 🆘 Support

For support and questions:
- Check the [troubleshooting section](#troubleshooting)
- Review application logs
- Open an issue on the repository

---

**AST Repository Visualizer** - Visualize your Python code structure with modern web technology! 🐍📊
