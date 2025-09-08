"""
Flask Application for AST Repository Visualizer
Production-ready web application with security and performance optimizations
"""
from flask import Flask, render_template, jsonify, request, session
from flask_cors import CORS
import os
import json
from pathlib import Path
from typing import Dict, List, Optional
import hashlib
from datetime import datetime
import uuid
import logging
import traceback
from functools import wraps
import time

from ast_analyzer import RepositoryAnalyzer, SecurityError
from cache_manager import CacheManager

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Initialize Flask app
app = Flask(__name__)

# Security: Require SECRET_KEY in environment
secret_key = os.environ.get('SECRET_KEY')
if not secret_key:
    raise ValueError("SECRET_KEY environment variable is required")
app.secret_key = secret_key

# CORS configuration - restrictive for production
cors_origins = os.environ.get('CORS_ORIGINS', 'http://localhost:5000').split(',')
CORS(app, origins=cors_origins)

# Initialize components
cache = CacheManager()
analyzer = RepositoryAnalyzer(cache)

# Configuration
UPLOAD_FOLDER = Path('uploads')
UPLOAD_FOLDER.mkdir(exist_ok=True)
MAX_FILE_SIZE = 100 * 1024 * 1024  # 100MB
REQUEST_TIMEOUT = 300  # 5 minutes
RATE_LIMIT_REQUESTS = int(os.environ.get('RATE_LIMIT_REQUESTS', '10'))
RATE_LIMIT_WINDOW = int(os.environ.get('RATE_LIMIT_WINDOW', '60'))  # seconds

# In-memory rate limiting (use Redis in production)
request_counts = {}


def rate_limit(max_requests: int = RATE_LIMIT_REQUESTS, window: int = RATE_LIMIT_WINDOW):
    """Simple rate limiting decorator."""
    def decorator(f):
        @wraps(f)
        def decorated_function(*args, **kwargs):
            client_ip = request.environ.get('HTTP_X_FORWARDED_FOR', request.remote_addr)
            current_time = int(time.time())
            window_start = current_time - window
            
            # Clean old entries
            if client_ip in request_counts:
                request_counts[client_ip] = [
                    req_time for req_time in request_counts[client_ip] 
                    if req_time > window_start
                ]
            else:
                request_counts[client_ip] = []
            
            # Check rate limit
            if len(request_counts[client_ip]) >= max_requests:
                return jsonify({'error': 'Rate limit exceeded'}), 429
            
            # Add current request
            request_counts[client_ip].append(current_time)
            
            return f(*args, **kwargs)
        return decorated_function
    return decorator


def validate_request_data(required_fields: List[str]):
    """Validate request JSON data."""
    def decorator(f):
        @wraps(f)
        def decorated_function(*args, **kwargs):
            if not request.is_json:
                return jsonify({'error': 'Request must be JSON'}), 400
            
            data = request.get_json()
            if not data:
                return jsonify({'error': 'No JSON data provided'}), 400
            
            # Check required fields
            missing_fields = [field for field in required_fields if field not in data]
            if missing_fields:
                return jsonify({
                    'error': f'Missing required fields: {", ".join(missing_fields)}'
                }), 400
            
            return f(data, *args, **kwargs)
        return decorated_function
    return decorator


@app.errorhandler(404)
def not_found(error):
    """Handle 404 errors."""
    return jsonify({'error': 'Resource not found'}), 404


@app.errorhandler(500)
def internal_error(error):
    """Handle 500 errors."""
    logger.error(f"Internal server error: {error}")
    return jsonify({'error': 'Internal server error'}), 500


@app.errorhandler(Exception)
def handle_exception(e):
    """Handle uncaught exceptions."""
    logger.error(f"Unhandled exception: {e}\n{traceback.format_exc()}")
    return jsonify({'error': 'An unexpected error occurred'}), 500


@app.before_request
def before_request():
    """Log requests and check content length."""
    # Log request
    logger.info(f"{request.method} {request.path} from {request.remote_addr}")
    
    # Check content length
    if request.content_length and request.content_length > MAX_FILE_SIZE:
        return jsonify({'error': 'Request too large'}), 413


@app.after_request
def after_request(response):
    """Add security headers and log response."""
    # Security headers
    response.headers['X-Content-Type-Options'] = 'nosniff'
    response.headers['X-Frame-Options'] = 'DENY'
    response.headers['X-XSS-Protection'] = '1; mode=block'
    response.headers['Strict-Transport-Security'] = 'max-age=31536000; includeSubDomains'
    
    # Log response
    logger.info(f"Response {response.status_code} for {request.method} {request.path}")
    
    return response


@app.route('/')
def index():
    """Main page with repository input."""
    return render_template('index.html')


@app.route('/health')
def health_check():
    """Health check endpoint."""
    cache_status = cache.health_check()
    
    return jsonify({
        'status': 'healthy',
        'timestamp': datetime.utcnow().isoformat(),
        'cache': cache_status,
        'version': '1.0.0'
    })


@app.route('/api/analyze', methods=['POST'])
@rate_limit(max_requests=5, window=300)  # Stricter limit for analysis
def analyze_repository():
    """Analyze a repository and return AST data."""
    try:
        data = request.get_json()
        if not data:
            return jsonify({'error': 'No JSON data provided'}), 400
        
        repo_path = data.get('path')
        repo_url = data.get('url')
        
        if not repo_path and not repo_url:
            return jsonify({'error': 'Please provide repository path or URL'}), 400
        
        if repo_path and repo_url:
            return jsonify({'error': 'Provide either path or URL, not both'}), 400
        
        # Generate analysis ID
        analysis_id = str(uuid.uuid4())
        
        # Store analysis start in session
        session['current_analysis'] = analysis_id
        session['analysis_start'] = time.time()
        
        logger.info(f"Starting analysis {analysis_id}")
        
        # Analyze repository
        try:
            if repo_url:
                result = analyzer.analyze_from_url(repo_url, analysis_id)
            else:
                result = analyzer.analyze_local(repo_path, analysis_id)
        except SecurityError as e:
            logger.warning(f"Security error in analysis {analysis_id}: {e}")
            return jsonify({'error': f'Security validation failed: {str(e)}'}), 400
        except ValueError as e:
            logger.warning(f"Validation error in analysis {analysis_id}: {e}")
            return jsonify({'error': str(e)}), 400
        except Exception as e:
            logger.error(f"Analysis error {analysis_id}: {e}")
            return jsonify({'error': 'Analysis failed due to internal error'}), 500
        
        logger.info(f"Analysis {analysis_id} completed successfully")
        
        return jsonify({
            'analysis_id': analysis_id,
            'summary': result['summary'],
            'files': result['files'][:100],  # Limit for performance
            'metrics': result['metrics'],
            'analysis_time': result.get('analysis_time', 0)
        })
    
    except Exception as e:
        logger.error(f"Unexpected error in analyze_repository: {e}")
        return jsonify({'error': 'Internal server error'}), 500


@app.route('/api/visualize/<analysis_id>')
@rate_limit()
def get_visualization(analysis_id: str):
    """Get visualization data for specific analysis."""
    try:
        # Validate analysis_id format
        try:
            uuid.UUID(analysis_id)
        except ValueError:
            return jsonify({'error': 'Invalid analysis ID format'}), 400
        
        # Get data from cache
        data = cache.get(f"analysis:{analysis_id}")
        if not data:
            return jsonify({'error': 'Analysis not found or expired'}), 404
        
        # Convert to visualization format - pass analysis_id for chunked file retrieval
        viz_data = analyzer.prepare_visualization(data, analysis_id)
        
        return jsonify(viz_data)
    
    except Exception as e:
        logger.error(f"Visualization error for {analysis_id}: {e}")
        return jsonify({'error': 'Failed to generate visualization'}), 500


@app.route('/api/file/<analysis_id>/<path:file_path>')
@rate_limit()
def get_file_ast(analysis_id: str, file_path: str):
    """Get detailed AST for a specific file."""
    try:
        # Validate analysis_id format
        try:
            uuid.UUID(analysis_id)
        except ValueError:
            return jsonify({'error': 'Invalid analysis ID format'}), 400
        
        # Sanitize file_path
        if '..' in file_path or file_path.startswith('/'):
            return jsonify({'error': 'Invalid file path'}), 400
        
        # Decode URL-encoded file path
        from urllib.parse import unquote
        decoded_file_path = unquote(file_path)
        
        # Try multiple cache key patterns
        cache_keys = [
            f"file:{analysis_id}:{decoded_file_path}",
            f"file:{analysis_id}:{Path(decoded_file_path).name}",
            f"file:{analysis_id}:{file_path}",
            f"file:{analysis_id}:{Path(file_path).name}"
        ]
        
        file_data = None
        for cache_key in cache_keys:
            file_data = cache.get(cache_key)
            if file_data:
                logger.info(f"Found file data with cache key: {cache_key}")
                break
        
        if not file_data:
            # Try to find by searching all cached files for this analysis
            all_files = cache.get(f"files:{analysis_id}")
            if all_files:
                # Search by filename
                target_filename = Path(decoded_file_path).name
                for cached_file in all_files:
                    if cached_file.get('path', '').endswith(target_filename):
                        file_data = cached_file
                        logger.info(f"Found file data by filename search: {target_filename}")
                        break
        
        if not file_data:
            logger.warning(f"File not found: {decoded_file_path} (tried keys: {cache_keys})")
            return jsonify({'error': f'File not found: {decoded_file_path}'}), 404
        
        return jsonify(file_data)
    
    except Exception as e:
        logger.error(f"File AST error for {analysis_id}/{file_path}: {e}")
        return jsonify({'error': 'Failed to retrieve file data'}), 500


@app.route('/api/source/<analysis_id>/<path:file_path>')
@rate_limit()
def get_file_source(analysis_id: str, file_path: str):
    """Get source code for a specific file."""
    try:
        # Validate analysis_id format
        try:
            uuid.UUID(analysis_id)
        except ValueError:
            return jsonify({'error': 'Invalid analysis ID format'}), 400
        
        # Sanitize file_path
        if '..' in file_path or file_path.startswith('/'):
            return jsonify({'error': 'Invalid file path'}), 400
        
        # Decode URL-encoded file path
        from urllib.parse import unquote
        decoded_file_path = unquote(file_path)
        
        # Get analysis data from cache to find the original repository path
        analysis_data = cache.get(f"analysis:{analysis_id}")
        if not analysis_data:
            return jsonify({'error': 'Analysis not found'}), 404
        
        # Check if this is a temporary clone (Git repository)
        is_temporary_clone = analysis_data.get('is_temporary_clone', False)
        
        if is_temporary_clone:
            # For temporary clones, try to get source from cache first
            cache_keys = [
                f"source:{analysis_id}:{decoded_file_path}",
                f"source:{analysis_id}:{Path(decoded_file_path).name}",
            ]
            
            for cache_key in cache_keys:
                cached_source = cache.get(cache_key)
                if cached_source:
                    logger.info(f"Found cached source with key: {cache_key}")
                    return jsonify(cached_source)
            
            # If not in cache, return error since temp files are gone
            return jsonify({
                'error': 'Source file not available (temporary repository)',
                'source': f'# Source file not available: {decoded_file_path}\n# This was from a temporary Git repository clone\n# The source files were not cached during analysis',
                'encoding': 'error',
                'lines': 0
            })
        
        # For local repositories, use the original file reading logic
        original_repo_path = analysis_data.get('repo_path')
        if not original_repo_path:
            return jsonify({'error': 'Repository path not found in analysis'}), 404
        
        # Construct full file path
        full_file_path = Path(original_repo_path) / decoded_file_path
        
        # Security check: ensure the file is within the repository
        try:
            full_file_path = full_file_path.resolve()
            repo_path = Path(original_repo_path).resolve()
            if not str(full_file_path).startswith(str(repo_path)):
                return jsonify({'error': 'File path outside repository'}), 400
        except Exception:
            return jsonify({'error': 'Invalid file path'}), 400
        
        # Try to read the actual source file
        try:
            if not full_file_path.exists():
                return jsonify({'error': 'Source file not found'}), 404
            
            # Check file size (limit to 1MB for web display)
            if full_file_path.stat().st_size > 1024 * 1024:
                return jsonify({
                    'error': 'File too large to display',
                    'size': full_file_path.stat().st_size,
                    'source': f'# File too large to display: {decoded_file_path}\n# Size: {full_file_path.stat().st_size} bytes\n# Please view this file locally',
                    'encoding': 'utf-8',
                    'lines': 0
                })
            
            # Try different encodings
            encodings = ['utf-8', 'latin-1', 'cp1252']
            source_content = None
            encoding_used = 'utf-8'
            
            for encoding in encodings:
                try:
                    with open(full_file_path, 'r', encoding=encoding) as f:
                        source_content = f.read()
                        encoding_used = encoding
                        break
                except UnicodeDecodeError:
                    continue
            
            if source_content is None:
                return jsonify({
                    'error': 'Unable to decode file',
                    'source': f'# Unable to decode file: {decoded_file_path}\n# File may contain binary data or use an unsupported encoding',
                    'encoding': 'error',
                    'lines': 0
                })
            
            lines = source_content.count('\n') + 1
            
            return jsonify({
                'source': source_content,
                'encoding': encoding_used,
                'lines': lines,
                'size': full_file_path.stat().st_size,
                'path': decoded_file_path
            })
            
        except PermissionError:
            return jsonify({'error': 'Permission denied reading source file'}), 403
        except Exception as e:
            logger.error(f"Error reading source file {full_file_path}: {e}")
            return jsonify({
                'error': 'Failed to read source file',
                'source': f'# Error reading file: {decoded_file_path}\n# {str(e)}',
                'encoding': 'error',
                'lines': 0
            })
    
    except Exception as e:
        logger.error(f"Source code error for {analysis_id}/{file_path}: {e}")
        return jsonify({'error': 'Failed to retrieve source code'}), 500


@app.route('/api/search/<analysis_id>')
@rate_limit()
def search_nodes(analysis_id: str):
    """Search for specific AST nodes."""
    try:
        # Validate analysis_id format
        try:
            uuid.UUID(analysis_id)
        except ValueError:
            return jsonify({'error': 'Invalid analysis ID format'}), 400
        
        query = request.args.get('q', '').strip()
        node_type = request.args.get('type', '').strip()
        
        # Validate query length
        if len(query) > 100:
            return jsonify({'error': 'Query too long'}), 400
        
        # Validate node_type
        allowed_types = {
            'FunctionDef', 'ClassDef', 'Import', 'ImportFrom', 
            'If', 'For', 'While', 'Try', 'With'
        }
        if node_type and node_type not in allowed_types:
            return jsonify({'error': f'Invalid node type. Allowed: {sorted(allowed_types)}'}), 400
        
        results = analyzer.search_nodes(analysis_id, query, node_type)
        
        return jsonify({
            'results': results,
            'query': query,
            'node_type': node_type,
            'count': len(results)
        })
    
    except Exception as e:
        logger.error(f"Search error for {analysis_id}: {e}")
        return jsonify({'error': 'Search failed'}), 500


@app.route('/api/analysis/<analysis_id>')
@rate_limit()
def get_analysis_info(analysis_id: str):
    """Get basic analysis information."""
    try:
        # Validate analysis_id format
        try:
            uuid.UUID(analysis_id)
        except ValueError:
            return jsonify({'error': 'Invalid analysis ID format'}), 400
        
        data = cache.get(f"analysis:{analysis_id}")
        if not data:
            return jsonify({'error': 'Analysis not found'}), 404
        
        # Return summary information only
        return jsonify({
            'analysis_id': analysis_id,
            'summary': data.get('summary', {}),
            'timestamp': data.get('timestamp'),
            'analysis_time': data.get('analysis_time'),
            'file_count': len(data.get('files', []))
        })
    
    except Exception as e:
        logger.error(f"Analysis info error for {analysis_id}: {e}")
        return jsonify({'error': 'Failed to retrieve analysis info'}), 500


@app.route('/api/export/<analysis_id>')
@rate_limit(max_requests=3, window=300)  # Very restrictive for exports
def export_analysis(analysis_id: str):
    """Export analysis data in JSON format."""
    try:
        # Validate analysis_id format
        try:
            uuid.UUID(analysis_id)
        except ValueError:
            return jsonify({'error': 'Invalid analysis ID format'}), 400
        
        data = cache.get(f"analysis:{analysis_id}")
        if not data:
            return jsonify({'error': 'Analysis not found'}), 404
        
        # Prepare export data (excluding internal cache keys)
        export_data = {
            'analysis_id': analysis_id,
            'summary': data.get('summary', {}),
            'metrics': data.get('metrics', {}),
            'timestamp': data.get('timestamp'),
            'export_timestamp': time.time(),
            'version': '1.0.0'
        }
        
        response = jsonify(export_data)
        response.headers['Content-Disposition'] = f'attachment; filename=ast_analysis_{analysis_id}.json'
        
        return response
    
    except Exception as e:
        logger.error(f"Export error for {analysis_id}: {e}")
        return jsonify({'error': 'Export failed'}), 500


@app.route('/visualization/<analysis_id>')
def visualization_page(analysis_id: str):
    """Visualization page for analysis results."""
    try:
        # Validate analysis_id format
        try:
            uuid.UUID(analysis_id)
        except ValueError:
            return render_template('error.html', 
                                 error='Invalid analysis ID format'), 400
        
        # Check if analysis exists
        data = cache.get(f"analysis:{analysis_id}")
        if not data:
            return render_template('error.html', 
                                 error='Analysis not found or expired'), 404
        
        return render_template('visualization.html', 
                             analysis_id=analysis_id,
                             summary=data.get('summary', {}))
    
    except Exception as e:
        logger.error(f"Visualization page error for {analysis_id}: {e}")
        return render_template('error.html', 
                             error='Failed to load visualization'), 500


@app.route('/api/status')
@rate_limit()
def get_status():
    """Get application status and statistics."""
    try:
        cache_status = cache.health_check()
        
        # Get current session info
        session_info = {}
        if 'current_analysis' in session:
            session_info = {
                'current_analysis': session['current_analysis'],
                'analysis_start': session.get('analysis_start')
            }
        
        return jsonify({
            'status': 'running',
            'cache': cache_status,
            'session': session_info,
            'configuration': {
                'max_file_size_mb': MAX_FILE_SIZE // (1024 * 1024),
                'request_timeout_seconds': REQUEST_TIMEOUT,
                'rate_limit_requests': RATE_LIMIT_REQUESTS,
                'rate_limit_window': RATE_LIMIT_WINDOW
            }
        })
    
    except Exception as e:
        logger.error(f"Status endpoint error: {e}")
        return jsonify({'error': 'Failed to get status'}), 500


if __name__ == '__main__':
    # Production server configuration
    debug_mode = os.environ.get('FLASK_DEBUG', 'False').lower() == 'true'
    host = os.environ.get('FLASK_HOST', '0.0.0.0')
    port = int(os.environ.get('FLASK_PORT', 5000))
    
    if debug_mode:
        logger.warning("Running in debug mode - NOT for production!")
    
    logger.info(f"Starting AST Visualizer on {host}:{port}")
    
    try:
        app.run(
            debug=debug_mode, 
            host=host, 
            port=port,
            threaded=True
        )
    except KeyboardInterrupt:
        logger.info("Shutting down gracefully...")
        cache.close()
    except Exception as e:
        logger.error(f"Failed to start application: {e}")
        raise
