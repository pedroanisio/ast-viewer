#!/usr/bin/env python3
"""
Production startup script for AST Visualizer
"""
import os
import sys
import logging
from pathlib import Path

# Add current directory to Python path
sys.path.insert(0, str(Path(__file__).parent))

# Configure logging
logging.basicConfig(
    level=getattr(logging, os.getenv('LOG_LEVEL', 'INFO')),
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(),
        logging.FileHandler('ast_visualizer.log')
    ]
)

logger = logging.getLogger(__name__)


def check_requirements():
    """Check if all requirements are installed."""
    try:
        import flask
        import redis
        import git
        import plotly
        import networkx
        logger.info("All required packages are available")
        return True
    except ImportError as e:
        logger.error(f"Missing required package: {e}")
        logger.error("Please run: pip install -r requirements.txt")
        return False


def check_environment():
    """Check environment configuration."""
    required_vars = ['SECRET_KEY']
    missing_vars = []
    
    for var in required_vars:
        if not os.getenv(var):
            missing_vars.append(var)
    
    if missing_vars:
        logger.error(f"Missing required environment variables: {missing_vars}")
        logger.error("Please copy env.example to .env and configure values")
        return False
    
    logger.info("Environment configuration is valid")
    return True


def check_redis():
    """Check Redis connection (optional)."""
    try:
        import redis
        redis_url = os.getenv('REDIS_URL', 'redis://localhost:6379')
        client = redis.from_url(redis_url)
        client.ping()
        logger.info("Redis connection successful")
        return True
    except Exception as e:
        logger.warning(f"Redis not available: {e}")
        logger.warning("Using memory cache fallback")
        return False


def main():
    """Main startup function."""
    logger.info("Starting AST Visualizer...")
    
    # Check requirements
    if not check_requirements():
        sys.exit(1)
    
    # Check environment
    if not check_environment():
        sys.exit(1)
    
    # Check Redis (optional)
    check_redis()
    
    # Import and start application
    try:
        from app import app
        
        # Configuration
        debug_mode = os.getenv('FLASK_DEBUG', 'False').lower() == 'true'
        host = os.getenv('FLASK_HOST', '0.0.0.0')
        port = int(os.getenv('FLASK_PORT', 5000))
        
        if debug_mode:
            logger.warning("Running in debug mode - NOT for production!")
        
        logger.info(f"Starting server on {host}:{port}")
        
        app.run(
            debug=debug_mode,
            host=host,
            port=port,
            threaded=True
        )
        
    except Exception as e:
        logger.error(f"Failed to start application: {e}")
        sys.exit(1)


if __name__ == '__main__':
    main()
