"""
Modern AST Analysis Engine for Multi-Language Repositories
Provides comprehensive analysis with security and performance optimizations
Supports Python, JavaScript, TypeScript, Go, Rust, C/C++, Java, CSS, HTML
"""
import ast
import os
import sys
from pathlib import Path
from typing import Dict, List, Optional, Any, Set, Tuple
from dataclasses import dataclass, asdict
from collections import defaultdict
import json
import hashlib
import git
import tempfile
import shutil
from concurrent.futures import ThreadPoolExecutor, as_completed
import threading
import logging
import time
from urllib.parse import urlparse

# Multi-language support
try:
    from language_detector import LanguageDetector, Language
    from language_analyzer import AnalyzerFactory, UniversalFileAnalysis
    MULTI_LANGUAGE_AVAILABLE = True
except ImportError:
    MULTI_LANGUAGE_AVAILABLE = False
    logging.warning("Multi-language analyzers not available, falling back to Python-only")

# Import radon safely
try:
    import radon.complexity as radon_cc
    import radon.metrics as radon_metrics
    RADON_AVAILABLE = True
except ImportError:
    RADON_AVAILABLE = False
    logging.warning("Radon not available, complexity analysis will be simplified")

logger = logging.getLogger(__name__)


@dataclass
class ASTNode:
    """Represents an AST node with metadata."""
    id: str
    type: str
    name: Optional[str]
    file: str
    line: int
    col: int
    children: List[str]
    properties: Dict[str, Any]
    complexity: Optional[int] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return asdict(self)


@dataclass
class FileAnalysis:
    """Analysis results for a single file."""
    path: str
    nodes: List[ASTNode]
    imports: List[str]
    classes: List[str]
    functions: List[str]
    complexity: float
    lines: int
    hash: str
    size_bytes: int
    encoding: str = 'utf-8'
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            'path': self.path,
            'nodes': [node.to_dict() for node in self.nodes],
            'imports': self.imports,
            'classes': self.classes,
            'functions': self.functions,
            'complexity': self.complexity,
            'lines': self.lines,
            'hash': self.hash,
            'size_bytes': self.size_bytes,
            'encoding': self.encoding
        }


class SecurityError(Exception):
    """Raised when security validation fails."""
    pass


class RepositoryAnalyzer:
    """Modern AST analyzer for Python repositories with security and performance optimizations."""
    
    # Security: Restrict file sizes and paths
    MAX_FILE_SIZE = 10 * 1024 * 1024  # 10MB per file
    MAX_TOTAL_SIZE = 500 * 1024 * 1024  # 500MB total
    MAX_FILES = 10000  # Maximum files to process
    
    # Excluded directories for security and performance
    EXCLUDED_DIRS = {
        '.git', '__pycache__', 'venv', 'env', '.env', 'node_modules',
        '.venv', 'site-packages', 'dist', 'build', '.pytest_cache',
        '.mypy_cache', '.tox', 'htmlcov', '.coverage'
    }
    
    # Allowed file extensions (now supports multiple languages)
    ALLOWED_EXTENSIONS = {
        # Python
        '.py', '.pyw',
        # JavaScript/TypeScript
        '.js', '.mjs', '.jsx', '.ts', '.tsx',
        # Systems languages
        '.go', '.rs', '.c', '.h', '.cpp', '.cc', '.cxx', '.hpp', '.hxx', '.hh',
        # Enterprise languages  
        '.java',
        # Web languages
        '.css', '.scss', '.sass', '.less', '.html', '.htm', '.xhtml'
    }
    
    def __init__(self, cache_manager=None, max_workers: Optional[int] = None):
        """
        Initialize repository analyzer.
        
        Args:
            cache_manager: Cache manager instance
            max_workers: Maximum worker threads (defaults to CPU count / 2)
        """
        self.cache = cache_manager
        # MEMORY OPTIMIZATION: Use WeakValueDictionary to allow garbage collection
        import weakref
        self.node_registry: weakref.WeakValueDictionary = weakref.WeakValueDictionary()
        self.file_registry: Dict[str, FileAnalysis] = {}
        self._lock = threading.Lock()
        self.max_workers = max_workers or max(1, os.cpu_count() // 2)
        
        # MEMORY OPTIMIZATION: Interning frequently used strings
        self._string_cache = {}
        self._node_type_cache = {}
        
    def _validate_path(self, path: Path) -> None:
        """
        Validate path for security issues.
        
        Args:
            path: Path to validate
            
        Raises:
            SecurityError: If path is invalid or unsafe
        """
        try:
            # Resolve path to prevent traversal attacks
            resolved_path = path.resolve()
            
            # Check if path exists
            if not resolved_path.exists():
                raise SecurityError(f"Path does not exist: {path}")
            
            # Check for suspicious path components
            parts = resolved_path.parts
            suspicious = {'.', '..', '~'}
            if any(part in suspicious for part in parts):
                raise SecurityError(f"Suspicious path components: {path}")
                
            # Ensure it's within reasonable bounds (not system directories)
            system_dirs = {'/bin', '/sbin', '/etc', '/sys', '/proc', '/dev'}
            if any(str(resolved_path).startswith(sdir) for sdir in system_dirs):
                raise SecurityError(f"System directory access denied: {path}")
                
        except Exception as e:
            if isinstance(e, SecurityError):
                raise
            raise SecurityError(f"Path validation failed: {e}")
    
    def _validate_url(self, url: str) -> None:
        """
        Validate Git URL for security.
        
        Args:
            url: Git URL to validate
            
        Raises:
            SecurityError: If URL is invalid or unsafe
        """
        if not url or not isinstance(url, str):
            raise SecurityError("URL must be a non-empty string")
            
        # Check for common malformed URLs
        if url.startswith('https:/') and not url.startswith('https://'):
            raise SecurityError("Malformed URL: missing '//' after protocol (should be https://)")
        if url.startswith('http:/') and not url.startswith('http://'):
            raise SecurityError("Malformed URL: missing '//' after protocol (should be http://)")
            
        try:
            parsed = urlparse(url)
            
            # Check for valid scheme
            allowed_schemes = {'http', 'https', 'git'}
            if not parsed.scheme:
                raise SecurityError("URL must include a protocol (http://, https://, or git://)")
            if parsed.scheme not in allowed_schemes:
                raise SecurityError(f"Unsupported URL scheme: {parsed.scheme}. Use http, https, or git")
            
            # Check for valid hostname
            if not parsed.hostname:
                raise SecurityError("URL must include a valid hostname")
                
            # Block local/private IPs (basic check)
            if parsed.hostname in {'localhost', '127.0.0.1', '0.0.0.0'}:
                raise SecurityError("Local URLs not allowed")
                
            # Basic Git repository URL patterns
            if not (url.endswith('.git') or 'github.com' in url or 'gitlab.com' in url or 'bitbucket.org' in url):
                logger.warning(f"URL doesn't look like a Git repository: {url}")
                    
        except SecurityError:
            raise
        except Exception as e:
            raise SecurityError(f"URL validation failed: {str(e)}")
    
    def analyze_from_url(self, repo_url: str, analysis_id: str, timeout: int = 300) -> Dict:
        """
        Clone and analyze repository from URL with security validation.
        
        Args:
            repo_url: Git repository URL
            analysis_id: Unique analysis identifier
            timeout: Timeout in seconds for git operations (handled by subprocess)
            
        Returns:
            Analysis results dictionary
        """
        self._validate_url(repo_url)
        
        with tempfile.TemporaryDirectory() as tmpdir:
            try:
                # Clone repository with depth limit
                repo_path = Path(tmpdir) / 'repo'
                logger.info(f"Cloning repository: {repo_url}")
                
                # Use shallow clone for performance and security
                # Note: GitPython doesn't support timeout directly, but git operations
                # will typically timeout on their own after a reasonable time
                repo = git.Repo.clone_from(
                    repo_url, 
                    repo_path,
                    depth=1,  # Shallow clone
                    single_branch=True  # Only clone the default branch
                )
                
                # Mark this as a temporary clone for source caching
                self._temp_repo_path = str(repo_path)
                self._repo_url = repo_url
                
                try:
                    result = self.analyze_local(str(repo_path), analysis_id)
                    
                    # Cache source files before the temporary directory is cleaned up
                    self._cache_source_files(repo_path, analysis_id)
                    
                    return result
                finally:
                    # Clean up attributes
                    if hasattr(self, '_temp_repo_path'):
                        delattr(self, '_temp_repo_path')
                    if hasattr(self, '_repo_url'):
                        delattr(self, '_repo_url')
                
            except git.exc.GitError as e:
                logger.error(f"Git clone failed: {e}")
                raise ValueError(f"Failed to clone repository: {e}")
            except Exception as e:
                logger.error(f"Repository analysis failed: {e}")
                raise
    
    def analyze_local(self, repo_path: str, analysis_id: str) -> Dict:
        """
        Analyze local repository with security and size validation.
        
        Args:
            repo_path: Path to local repository
            analysis_id: Unique analysis identifier
            
        Returns:
            Analysis results dictionary
        """
        start_time = time.time()
        repo_path = Path(repo_path)
        
        # Security validation
        self._validate_path(repo_path)
        
        # Find all Python files with size validation
        python_files = self._find_python_files(repo_path)
        
        if not python_files:
            raise ValueError("No Python files found in repository")
        
        logger.info(f"Found {len(python_files)} Python files to analyze")
        
        # Analyze files in parallel
        results = self._parallel_analyze(python_files, analysis_id)
        
        # Generate summary and metrics
        summary = self._generate_summary(results)
        metrics = self._calculate_metrics(results)
        
        # Cache results - MEMORY OPTIMIZATION: Stream data to cache in chunks
        if self.cache:
            # Store summary and metadata first
            summary_data = {
                'repo_path': str(repo_path),
                'summary': summary,
                'metrics': metrics,
                'analysis_time': time.time() - start_time,
                'timestamp': time.time(),
                'file_count': len(results),
                'is_temporary_clone': hasattr(self, '_temp_repo_path')
            }
            self.cache.set(f"analysis:{analysis_id}", summary_data, expire=7200)
            
            # MEMORY OPTIMIZATION: Cache files in smaller chunks to reduce memory pressure
            chunk_size = 50  # Files per chunk
            for i in range(0, len(results), chunk_size):
                chunk = results[i:i + chunk_size]
                chunk_data = [self._file_to_compact_dict(f) for f in chunk]
                self.cache.set(f"files:{analysis_id}:{i//chunk_size}", chunk_data, expire=7200)
            
            # MEMORY OPTIMIZATION: Only cache essential node data, not full objects
            essential_nodes = self._extract_essential_nodes()
            if essential_nodes:
                self.cache.set(f"nodes:{analysis_id}", essential_nodes, expire=7200)
            
            # Clear registries to free memory after caching
            self.node_registry.clear()
            self.file_registry.clear()
        
        logger.info(f"Analysis completed in {time.time() - start_time:.2f} seconds")
        
        return {
            'summary': summary,
            'files': [f.path for f in results],
            'metrics': metrics,
            'analysis_time': time.time() - start_time
        }
    
    def _find_python_files(self, repo_path: Path) -> List[Path]:
        """
        Find Python files with security and size validation.
        
        Args:
            repo_path: Repository root path
            
        Returns:
            List of valid Python file paths
        """
        python_files = []
        total_size = 0
        
        try:
            for root, dirs, files in os.walk(repo_path):
                # Remove excluded directories
                dirs[:] = [d for d in dirs if d not in self.EXCLUDED_DIRS]
                
                for file in files:
                    file_path = Path(root) / file
                    
                    # Check file extension
                    if file_path.suffix not in self.ALLOWED_EXTENSIONS:
                        continue
                    
                    try:
                        # Check file size
                        file_size = file_path.stat().st_size
                        if file_size > self.MAX_FILE_SIZE:
                            logger.warning(f"Skipping large file: {file_path} ({file_size} bytes)")
                            continue
                        
                        total_size += file_size
                        if total_size > self.MAX_TOTAL_SIZE:
                            logger.warning(f"Total size limit reached, stopping file discovery")
                            break
                        
                        python_files.append(file_path)
                        
                        if len(python_files) >= self.MAX_FILES:
                            logger.warning(f"File count limit reached: {self.MAX_FILES}")
                            break
                            
                    except OSError as e:
                        logger.warning(f"Cannot access file {file_path}: {e}")
                        continue
                
                if len(python_files) >= self.MAX_FILES or total_size > self.MAX_TOTAL_SIZE:
                    break
                    
        except Exception as e:
            logger.error(f"Error finding Python files: {e}")
            raise
        
        return python_files
    
    def _parallel_analyze(self, files: List[Path], analysis_id: str) -> List[FileAnalysis]:
        """
        Analyze multiple files in parallel with error handling.
        
        Args:
            files: List of file paths to analyze
            analysis_id: Analysis identifier
            
        Returns:
            List of file analysis results
        """
        results = []
        failed_files = []
        
        with ThreadPoolExecutor(max_workers=self.max_workers) as executor:
            # Submit all tasks
            future_to_file = {
                executor.submit(self._analyze_file, f, analysis_id): f 
                for f in files
            }
            
            # Collect results
            for future in as_completed(future_to_file):
                file_path = future_to_file[future]
                try:
                    result = future.result(timeout=60)  # 60 second timeout per file
                    if result:
                        results.append(result)
                except Exception as e:
                    failed_files.append((file_path, str(e)))
                    logger.error(f"Failed to analyze {file_path}: {e}")
        
        if failed_files:
            logger.warning(f"Failed to analyze {len(failed_files)} files")
        
        return results
    
    def _analyze_file(self, file_path: Path, analysis_id: str) -> Optional[FileAnalysis]:
        """
        Analyze a single file with multi-language support.
        
        Args:
            file_path: Path to file (any supported language)
            analysis_id: Analysis identifier
            
        Returns:
            File analysis result or None if failed
        """
        try:
            # Read file with encoding detection
            content, encoding = self._read_file_safely(file_path)
            if content is None:
                return None
            
            # Cache file content for source display
            if self.cache:
                cache_key = f"source:{analysis_id}:{file_path.name}"
                self.cache.set(cache_key, {
                    'content': content,
                    'encoding': encoding,
                    'path': str(file_path)
                }, expire=3600)
            
            # Use universal analyzer if available
            if MULTI_LANGUAGE_AVAILABLE:
                universal_result = AnalyzerFactory.analyze_file_auto(file_path, content)
                if universal_result:
                    legacy_result = self._convert_universal_to_legacy(universal_result)
                    
                    # Cache file analysis
                    if self.cache:
                        self.cache.set(
                            f"file:{analysis_id}:{file_path.name}", 
                            legacy_result.to_dict(),
                            expire=3600
                        )
                    
                    return legacy_result
            
            # Fallback to Python-only analysis
            if file_path.suffix.lower() in {'.py', '.pyw'}:
                return self._analyze_python_file_legacy(file_path, content, analysis_id, encoding)
            else:
                logger.debug(f"Unsupported file type for legacy analysis: {file_path}")
                return None
                
        except Exception as e:
            logger.error(f"Error processing {file_path}: {e}")
            return None
    
    def _analyze_python_file_legacy(self, file_path: Path, content: str, analysis_id: str, encoding: str) -> Optional[FileAnalysis]:
        """Legacy Python file analysis for fallback."""
        try:
            # Parse AST
            tree = ast.parse(content, filename=str(file_path))
            
            # Calculate file hash and metadata
            file_hash = hashlib.md5(content.encode()).hexdigest()
            file_size = file_path.stat().st_size
            
            # Extract AST information
            nodes = []
            imports = []
            classes = []
            functions = []
            
            # Visit all nodes
            for node in ast.walk(tree):
                node_data = self._process_node(node, str(file_path))
                if node_data:
                    with self._lock:
                        self.node_registry[node_data.id] = node_data
                    nodes.append(node_data)
                    
                    # Categorize nodes
                    if isinstance(node, ast.Import):
                        imports.extend(alias.name for alias in node.names)
                    elif isinstance(node, ast.ImportFrom):
                        module_name = node.module or ''
                        if module_name:
                            imports.append(module_name)
                    elif isinstance(node, ast.ClassDef):
                        classes.append(node.name)
                    elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                        functions.append(node.name)
            
            # Calculate complexity
            complexity = self._calculate_complexity(content)
            
            analysis = FileAnalysis(
                path=str(file_path),
                nodes=nodes,
                imports=self._dedupe_list(imports),
                classes=classes,
                functions=functions,
                complexity=complexity,
                lines=len(content.splitlines()),
                hash=file_hash,
                size_bytes=file_size,
                encoding=encoding
            )
            
            # Cache file analysis
            if self.cache:
                self.cache.set(
                    f"file:{analysis_id}:{file_path.name}", 
                    analysis.to_dict(),
                    expire=3600
                )
            
            return analysis
            
        except SyntaxError as e:
            logger.warning(f"Python syntax error in {file_path}: {e}")
            return None
        except Exception as e:
            logger.error(f"Legacy Python analysis failed for {file_path}: {e}")
            return None
    
    def _convert_universal_to_legacy(self, universal: UniversalFileAnalysis) -> FileAnalysis:
        """Convert UniversalFileAnalysis to legacy FileAnalysis format."""
        # Convert universal nodes to legacy ASTNode format
        legacy_nodes = []
        for universal_node in universal.nodes:
            legacy_node = ASTNode(
                id=universal_node.id,
                type=universal_node.type,
                name=universal_node.name,
                file=universal_node.file,
                line=universal_node.line,
                col=universal_node.col,
                children=universal_node.children,
                properties=universal_node.properties or {},
                complexity=universal_node.complexity
            )
            legacy_nodes.append(legacy_node)
            
            # Register in node registry for compatibility
            with self._lock:
                self.node_registry[legacy_node.id] = legacy_node
        
        return FileAnalysis(
            path=universal.path,
            nodes=legacy_nodes,
            imports=universal.imports,
            classes=universal.classes,
            functions=universal.functions,
            complexity=universal.complexity,
            lines=universal.lines,
            hash=universal.hash,
            size_bytes=universal.size_bytes,
            encoding=universal.encoding
        )
    
    def _read_file_safely(self, file_path: Path) -> Tuple[Optional[str], str]:
        """
        Safely read file with encoding detection.
        
        Args:
            file_path: Path to file
            
        Returns:
            Tuple of (content, encoding) or (None, '') if failed
        """
        encodings = ['utf-8', 'utf-8-sig', 'latin1', 'cp1252']
        
        for encoding in encodings:
            try:
                with open(file_path, 'r', encoding=encoding) as f:
                    content = f.read()
                return content, encoding
            except UnicodeDecodeError:
                continue
            except Exception as e:
                logger.error(f"Error reading {file_path}: {e}")
                break
        
        return None, ''
    
    def _process_node(self, node: ast.AST, file_path: str) -> Optional[ASTNode]:
        """
        Process individual AST node with comprehensive property extraction.
        
        Args:
            node: AST node
            file_path: File path containing the node
            
        Returns:
            Processed AST node or None
        """
        if not hasattr(node, 'lineno'):
            return None
        
        # MEMORY OPTIMIZATION: Use interned strings and efficient ID generation
        class_name = self._intern_string(node.__class__.__name__)
        col_offset = getattr(node, 'col_offset', 0)
        
        # More memory-efficient node ID generation
        node_id_parts = (file_path, str(node.lineno), str(col_offset), class_name)
        node_id_str = ':'.join(node_id_parts)
        node_id = hashlib.md5(node_id_str.encode('utf-8')).hexdigest()[:12]
        
        # MEMORY OPTIMIZATION: Extract only essential properties to reduce memory usage
        properties = {}
        essential_fields = {'name', 'id', 'arg', 'attr', 'value'}  # Only store essential fields
        for field in node._fields:
            if field in essential_fields:
                try:
                    value = getattr(node, field, None)
                    if value is not None and not isinstance(value, (ast.AST, list)):
                        # Intern string values to save memory
                        properties[field] = self._intern_string(str(value))
                except Exception:
                    continue
        
        # MEMORY OPTIMIZATION: Get node name efficiently with interning
        name = None
        for attr in ['name', 'id', 'arg']:
            if hasattr(node, attr):
                name = self._intern_string(str(getattr(node, attr)))
                break
        
        # Calculate complexity for functions/methods
        complexity = None
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            complexity = self._get_node_complexity(node)
        
        return ASTNode(
            id=node_id,
            type=node.__class__.__name__,
            name=name,
            file=file_path,
            line=node.lineno,
            col=getattr(node, 'col_offset', 0),
            children=[],  # Will be populated later if needed
            properties=properties,
            complexity=complexity
        )
    
    def _calculate_complexity(self, code: str) -> float:
        """
        Calculate cyclomatic complexity using radon if available.
        
        Args:
            code: Source code string
            
        Returns:
            Average complexity score
        """
        if not RADON_AVAILABLE:
            return self._simple_complexity(code)
        
        try:
            cc_results = radon_cc.cc_visit(code)
            if cc_results:
                total_complexity = sum(item.complexity for item in cc_results)
                return total_complexity / len(cc_results)
            return 1.0
        except Exception:
            return self._simple_complexity(code)
    
    def _simple_complexity(self, code: str) -> float:
        """
        Simple complexity calculation fallback.
        
        Args:
            code: Source code string
            
        Returns:
            Estimated complexity score
        """
        try:
            tree = ast.parse(code)
            complexity = 0
            
            for node in ast.walk(tree):
                if isinstance(node, (ast.If, ast.While, ast.For, ast.ExceptHandler,
                                   ast.With, ast.Assert, ast.Try)):
                    complexity += 1
                elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                    complexity += 1
                    
            return max(1.0, complexity / max(1, len(list(ast.walk(tree)))))
        except:
            return 1.0
    
    def _get_node_complexity(self, node: ast.AST) -> int:
        """
        Get complexity for a specific AST node.
        
        Args:
            node: AST node
            
        Returns:
            Complexity score for the node
        """
        complexity = 1  # Base complexity
        
        for child in ast.walk(node):
            if isinstance(child, (ast.If, ast.While, ast.For, ast.ExceptHandler)):
                complexity += 1
            elif isinstance(child, (ast.BoolOp, ast.Compare)):
                complexity += 1
            elif isinstance(child, ast.Try):
                complexity += len(child.handlers)
        
        return complexity
    
    def _generate_summary(self, results: List[FileAnalysis]) -> Dict:
        """
        Generate comprehensive repository summary.
        
        Args:
            results: List of file analysis results
            
        Returns:
            Summary dictionary
        """
        if not results:
            return {}
        
        total_lines = sum(f.lines for f in results)
        total_classes = sum(len(f.classes) for f in results)
        total_functions = sum(len(f.functions) for f in results)
        complexities = [f.complexity for f in results if f.complexity > 0]
        
        # MEMORY OPTIMIZATION: Efficiently collect unique imports without intermediate sets
        import_counter = {}
        for f in results:
            for imp in f.imports:
                import_counter[imp] = import_counter.get(imp, 0) + 1
        
        return {
            'total_files': len(results),
            'total_lines': total_lines,
            'total_classes': total_classes,
            'total_functions': total_functions,
            'average_complexity': sum(complexities) / len(complexities) if complexities else 0,
            'imports': sorted(import_counter.keys())[:100],  # Limit for performance
            'file_size_distribution': self._get_size_distribution(results),
            'top_complex_files': sorted(
                [(f.path, f.complexity) for f in results], 
                key=lambda x: x[1], 
                reverse=True
            )[:10]
        }
    
    def _get_size_distribution(self, results: List[FileAnalysis]) -> Dict[str, int]:
        """Get file size distribution statistics."""
        sizes = [f.lines for f in results]
        return {
            'small_files': len([s for s in sizes if s < 100]),
            'medium_files': len([s for s in sizes if 100 <= s < 500]),
            'large_files': len([s for s in sizes if s >= 500])
        }
    
    def _calculate_metrics(self, results: List[FileAnalysis]) -> Dict:
        """
        Calculate detailed repository metrics.
        
        Args:
            results: List of file analysis results
            
        Returns:
            Metrics dictionary
        """
        if not results:
            return {}
        
        metrics = {
            'files_by_complexity': [],
            'largest_files': [],
            'most_imports': [],
            'class_distribution': defaultdict(int),
            'function_distribution': defaultdict(int),
            'import_graph': self._build_import_graph(results)
        }
        
        # Sort by complexity
        complexity_sorted = sorted(results, key=lambda x: x.complexity, reverse=True)[:10]
        metrics['files_by_complexity'] = [
            {'file': Path(f.path).name, 'complexity': f.complexity} 
            for f in complexity_sorted
        ]
        
        # Sort by size
        size_sorted = sorted(results, key=lambda x: x.lines, reverse=True)[:10]
        metrics['largest_files'] = [
            {'file': Path(f.path).name, 'lines': f.lines} 
            for f in size_sorted
        ]
        
        # Sort by imports
        import_sorted = sorted(results, key=lambda x: len(x.imports), reverse=True)[:10]
        metrics['most_imports'] = [
            {'file': Path(f.path).name, 'imports': len(f.imports)} 
            for f in import_sorted
        ]
        
        return metrics
    
    def _build_import_graph(self, results: List[FileAnalysis]) -> Dict[str, List[str]]:
        """
        Build import dependency graph.
        
        Args:
            results: List of file analysis results
            
        Returns:
            Import graph as adjacency list
        """
        graph = {}
        file_modules = {Path(f.path).stem for f in results}
        
        for file_analysis in results:
            file_stem = Path(file_analysis.path).stem
            dependencies = []
            
            for imp in file_analysis.imports:
                # Check if import is a local module
                if imp in file_modules:
                    dependencies.append(imp)
                # Handle relative imports
                elif '.' in imp:
                    base_module = imp.split('.')[0]
                    if base_module in file_modules:
                        dependencies.append(base_module)
            
            graph[file_stem] = dependencies
        
        return graph
    
    def prepare_visualization(self, analysis_data: Dict, analysis_id: str = None) -> Dict:
        """
        Prepare data for web visualization.
        
        Args:
            analysis_data: Raw analysis data
            analysis_id: Analysis identifier for retrieving chunked file data
            
        Returns:
            Visualization-ready data
        """
        nodes = []
        edges = []
        
        # Create nodes for files - handle both direct files array and chunked storage
        files_data = []
        
        # First try to get files directly from analysis_data (backward compatibility)
        if 'files' in analysis_data:
            files_data = analysis_data['files']
        else:
            # Reconstruct files from chunks if we have cache and analysis_id
            if self.cache and analysis_id:
                chunk_index = 0
                while True:
                    chunk_key = f"files:{analysis_id}:{chunk_index}"
                    chunk_data = self.cache.get(chunk_key)
                    if not chunk_data:
                        break
                    files_data.extend(chunk_data)
                    chunk_index += 1
                    
                    # Safety limit to prevent infinite loops
                    if chunk_index > 100:  # Max 5000 files (50 per chunk * 100 chunks)
                        logger.warning(f"Reached maximum chunk limit for analysis {analysis_id}")
                        break
        
        # Create nodes for files
        for file_data in files_data:
            file_path = file_data['path']
            file_name = Path(file_path).name
            
            # Handle both list and integer formats for classes/functions
            classes_data = file_data.get('classes', [])
            functions_data = file_data.get('functions', [])
            
            # If they're already integers (from compact storage), use them directly
            # Otherwise, get the length of the list
            classes_count = classes_data if isinstance(classes_data, int) else len(classes_data)
            functions_count = functions_data if isinstance(functions_data, int) else len(functions_data)
            
            nodes.append({
                'id': file_path,
                'label': file_name,
                'type': 'file',
                'metrics': {
                    'lines': file_data['lines'],
                    'complexity': file_data['complexity'],
                    'classes': classes_count,
                    'functions': functions_count,
                    'size_bytes': file_data.get('size_bytes', 0)
                }
            })
        
        # Create edges for imports
        import_graph = analysis_data.get('metrics', {}).get('import_graph', {})
        for source, targets in import_graph.items():
            for target in targets:
                edges.append({
                    'source': source,
                    'target': target,
                    'type': 'import'
                })
        
        logger.info(f"Prepared visualization with {len(nodes)} nodes and {len(edges)} edges")
        
        return {
            'nodes': nodes,
            'edges': edges,
            'summary': analysis_data.get('summary', {}),
            'metrics': analysis_data.get('metrics', {})
        }
    
    def _cache_source_files(self, repo_path: Path, analysis_id: str):
        """Cache source files for temporary Git repositories."""
        if not self.cache:
            return
            
        logger.info(f"Caching source files for temporary repository: {analysis_id}")
        
        # Find all Python files and cache their content
        for file_path in repo_path.rglob('*.py'):
            if self._should_analyze_file(file_path):
                try:
                    relative_path = file_path.relative_to(repo_path)
                    
                    # Read file content with encoding handling
                    encodings = ['utf-8', 'latin-1', 'cp1252']
                    content = None
                    encoding_used = 'utf-8'
                    
                    for encoding in encodings:
                        try:
                            with open(file_path, 'r', encoding=encoding) as f:
                                content = f.read()
                                encoding_used = encoding
                                break
                        except UnicodeDecodeError:
                            continue
                    
                    if content is not None:
                        source_data = {
                            'source': content,
                            'encoding': encoding_used,
                            'lines': content.count('\n') + 1,
                            'size': file_path.stat().st_size,
                            'path': str(relative_path)
                        }
                        
                        # Cache with multiple key patterns for easy retrieval
                        cache_keys = [
                            f"source:{analysis_id}:{relative_path}",
                            f"source:{analysis_id}:{relative_path.name}",
                            f"source:{analysis_id}:{file_path}",  # Full original path
                        ]
                        
                        for cache_key in cache_keys:
                            self.cache.set(cache_key, source_data, expire=7200)
                            
                except Exception as e:
                    logger.warning(f"Failed to cache source for {file_path}: {e}")
                    
        logger.info(f"Source file caching completed for analysis: {analysis_id}")
    
    def _should_analyze_file(self, file_path: Path) -> bool:
        """Check if a file should be analyzed based on extension and size."""
        # Check file extension
        if file_path.suffix not in self.ALLOWED_EXTENSIONS:
            return False
            
        # Check file size (skip very large files)
        try:
            if file_path.stat().st_size > 10 * 1024 * 1024:  # 10MB limit
                return False
        except OSError:
            return False
            
        # Check if it's in excluded directories
        for part in file_path.parts:
            if part in self.EXCLUDED_DIRS:
                return False
                
        return True
    
    def search_nodes(self, analysis_id: str, query: str, node_type: str = '') -> List[Dict]:
        """
        Search for specific nodes in the analysis.
        
        Args:
            analysis_id: Analysis identifier
            query: Search query
            node_type: Optional node type filter
            
        Returns:
            List of matching nodes
        """
        if not self.cache:
            return []
        
        # Get analysis data
        data = self.cache.get(f"analysis:{analysis_id}")
        if not data:
            return []
        
        results = []
        query_lower = query.lower() if query else ""
        
        # Search through nodes
        for node_id, node_data in data.get('nodes', {}).items():
            # Skip if node_type filter doesn't match
            if node_type and node_data.get('type') != node_type:
                continue
            
            # Check if query matches
            if not query or self._node_matches_query(node_data, query_lower):
                results.append(node_data)
                
            # Limit results for performance
            if len(results) >= 100:
                break
        
        return results
    
    def _node_matches_query(self, node_data: Dict, query: str) -> bool:
        """Check if node matches search query."""
        searchable_fields = [
            str(node_data.get('name', '')).lower(),
            node_data.get('type', '').lower(),
            str(node_data.get('properties', {})).lower()
        ]
        
        return any(query in field for field in searchable_fields if field)
    
    # MEMORY OPTIMIZATION HELPER METHODS
    
    def _intern_string(self, s: str) -> str:
        """Intern frequently used strings to save memory."""
        if s not in self._string_cache:
            self._string_cache[s] = sys.intern(s) if len(s) < 100 else s
        return self._string_cache[s]
    
    def _file_to_compact_dict(self, file_analysis: FileAnalysis) -> Dict[str, Any]:
        """Convert file analysis to compact dictionary representation."""
        return {
            'path': file_analysis.path,
            'lines': file_analysis.lines,
            'complexity': round(file_analysis.complexity, 2),
            'classes': len(file_analysis.classes),
            'functions': len(file_analysis.functions),
            'imports': len(file_analysis.imports),
            'size_bytes': file_analysis.size_bytes,
            'hash': file_analysis.hash[:8]  # Shortened hash
        }
    
    def _extract_essential_nodes(self) -> Dict[str, Dict]:
        """Extract only essential node data for caching."""
        essential_nodes = {}
        
        # Only keep nodes that are likely to be searched or displayed
        important_types = {'FunctionDef', 'ClassDef', 'AsyncFunctionDef', 'Import', 'ImportFrom'}
        
        for node_id, node in list(self.node_registry.items()):
            if node and node.type in important_types:
                essential_nodes[node_id] = {
                    'type': node.type,
                    'name': node.name,
                    'file': node.file,
                    'line': node.line,
                    'complexity': node.complexity
                }
                
                # Limit to prevent memory bloat
                if len(essential_nodes) >= 1000:
                    break
        
        return essential_nodes
    
    def _dedupe_list(self, items: List[str]) -> List[str]:
        """Efficiently remove duplicates while preserving order."""
        if not items:
            return []
        
        seen = set()
        result = []
        for item in items:
            if item and item not in seen:
                seen.add(item)
                result.append(item)
        return result
