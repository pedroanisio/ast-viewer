"""
Project Builder - Orchestrates multi-level analysis and builds hierarchical project graph
"""
from pathlib import Path
from typing import Dict, List, Optional, Set, Any
import logging
import hashlib
from concurrent.futures import ThreadPoolExecutor, as_completed

from models.project_model import ProjectGraph, CodeElement, CodeLocation, NodeType, Scope
from language_detector import LanguageDetector, Language
from language_analyzer import AnalyzerFactory, UniversalFileAnalysis
from cache_manager import CacheManager

logger = logging.getLogger(__name__)


class HierarchyAnalyzer:
    """Analyzes project hierarchy and package structure."""
    
    def analyze_file_hierarchy(self, files: List[Path], root_path: Path) -> Dict[str, CodeElement]:
        """Build file and package hierarchy."""
        elements = {}
        
        # Group files by directory to identify packages/modules
        dir_files = {}
        for file_path in files:
            dir_path = file_path.parent
            if dir_path not in dir_files:
                dir_files[dir_path] = []
            dir_files[dir_path].append(file_path)
        
        # Create directory/package elements
        for dir_path, dir_files_list in dir_files.items():
            if dir_path == root_path:
                continue
                
            # Determine if this is a package based on language conventions
            is_package = self._is_package_directory(dir_path, dir_files_list)
            
            if is_package:
                package_id = f"package:{dir_path.relative_to(root_path)}"
                package_element = CodeElement(
                    id=package_id,
                    name=dir_path.name,
                    type=NodeType.PACKAGE,
                    language="multi",  # Packages can contain multiple languages
                    location=CodeLocation(
                        file_path=str(dir_path),
                        line_start=1,
                        line_end=1,
                        col_start=0,
                        col_end=0
                    ),
                    parent_id=self._get_parent_package_id(dir_path, root_path),
                    lines_of_code=0,
                    properties={'directory_path': str(dir_path)}
                )
                elements[package_id] = package_element
        
        # Create file elements
        for file_path in files:
            file_id = f"file:{file_path.relative_to(root_path)}"
            language = LanguageDetector.detect_language(file_path)
            
            file_element = CodeElement(
                id=file_id,
                name=file_path.name,
                type=NodeType.FILE,
                language=language.value,
                location=CodeLocation(
                    file_path=str(file_path.relative_to(root_path)),
                    line_start=1,
                    line_end=1,  # Will be updated with actual line count
                    col_start=0,
                    col_end=0
                ),
                parent_id=self._get_parent_package_id(file_path.parent, root_path),
                properties={'absolute_path': str(file_path)}
            )
            elements[file_id] = file_element
        
        return elements
    
    def _is_package_directory(self, dir_path: Path, files: List[Path]) -> bool:
        """Determine if directory represents a package/module."""
        # Python: has __init__.py
        if any(f.name == '__init__.py' for f in files):
            return True
        
        # JavaScript/TypeScript: has package.json or index.js/ts
        if any(f.name in ['package.json', 'index.js', 'index.ts'] for f in files):
            return True
        
        # Go: has go.mod or multiple .go files
        if any(f.name == 'go.mod' for f in files) or len([f for f in files if f.suffix == '.go']) >= 2:
            return True
        
        # Rust: has Cargo.toml or lib.rs/main.rs
        if any(f.name in ['Cargo.toml', 'lib.rs', 'main.rs'] for f in files):
            return True
        
        # Java: has multiple .java files (package structure)
        if len([f for f in files if f.suffix == '.java']) >= 2:
            return True
        
        return False
    
    def _get_parent_package_id(self, dir_path: Path, root_path: Path) -> Optional[str]:
        """Get parent package ID for a directory."""
        if dir_path == root_path or dir_path.parent == root_path:
            return None
        return f"package:{dir_path.parent.relative_to(root_path)}"


class DependencyAnalyzer:
    """Analyzes dependencies between code elements."""
    
    def analyze_dependencies(self, elements: Dict[str, CodeElement], 
                           file_analyses: Dict[str, UniversalFileAnalysis]) -> None:
        """Analyze and populate dependency relationships."""
        
        # Build name-to-element mappings for faster lookup
        element_by_name = self._build_name_mappings(elements)
        
        for element in elements.values():
            if element.type == NodeType.FILE:
                # Analyze file-level dependencies
                file_analysis = file_analyses.get(element.location.file_path)
                if file_analysis:
                    self._analyze_file_dependencies(element, file_analysis, element_by_name)
            
            elif element.type in [NodeType.CLASS, NodeType.FUNCTION, NodeType.METHOD]:
                # Analyze code element dependencies
                self._analyze_code_element_dependencies(element, elements, element_by_name)
    
    def _build_name_mappings(self, elements: Dict[str, CodeElement]) -> Dict[str, List[str]]:
        """Build mappings from names to element IDs."""
        mappings = {}
        
        for element_id, element in elements.items():
            if element.name not in mappings:
                mappings[element.name] = []
            mappings[element.name].append(element_id)
        
        return mappings
    
    def _analyze_file_dependencies(self, file_element: CodeElement, 
                                 file_analysis: UniversalFileAnalysis,
                                 element_by_name: Dict[str, List[str]]) -> None:
        """Analyze dependencies for a file element."""
        for import_stmt in file_analysis.imports:
            # Extract module/file name from import statement
            imported_names = self._extract_imported_names(import_stmt, file_analysis.language)
            
            for name in imported_names:
                # Find corresponding elements
                if name in element_by_name:
                    for dep_id in element_by_name[name]:
                        dep_element = file_element  # This would need proper resolution
                        if dep_element and dep_element.type == NodeType.FILE:
                            file_element.add_dependency(dep_id)
    
    def _extract_imported_names(self, import_stmt: str, language: str) -> List[str]:
        """Extract imported names from import statement."""
        names = []
        
        if language == 'python':
            # Handle Python imports: "import module", "from module import name"
            if import_stmt.startswith('from ') and ' import ' in import_stmt:
                module_part = import_stmt.split(' import ')[0].replace('from ', '')
                import_part = import_stmt.split(' import ')[1]
                names.append(module_part.strip())
                names.extend([n.strip() for n in import_part.split(',')])
            elif import_stmt.startswith('import '):
                module_names = import_stmt.replace('import ', '').split(',')
                names.extend([n.strip() for n in module_names])
        
        elif language in ['javascript', 'typescript']:
            # Handle JS/TS imports: "import name from 'module'", "require('module')"
            if 'from ' in import_stmt:
                module_part = import_stmt.split('from ')[-1].strip().strip("'\"")
                names.append(module_part)
        
        # Add more language-specific import parsing as needed
        
        return names
    
    def _analyze_code_element_dependencies(self, element: CodeElement, 
                                         all_elements: Dict[str, CodeElement],
                                         element_by_name: Dict[str, List[str]]) -> None:
        """Analyze dependencies for classes and functions."""
        # This would involve analyzing the actual code content for function calls,
        # class instantiations, variable references, etc.
        # For now, we'll implement basic name-based matching
        pass


class MetricsAnalyzer:
    """Analyzes code metrics at various levels."""
    
    def calculate_metrics(self, elements: Dict[str, CodeElement], 
                         file_analyses: Dict[str, UniversalFileAnalysis]) -> None:
        """Calculate comprehensive metrics for all elements."""
        
        # Calculate file-level metrics
        for element in elements.values():
            if element.type == NodeType.FILE:
                file_analysis = file_analyses.get(element.location.file_path)
                if file_analysis:
                    element.lines_of_code = file_analysis.lines
                    element.complexity = file_analysis.complexity
        
        # Calculate package-level metrics (aggregate from files)
        for element in elements.values():
            if element.type == NodeType.PACKAGE:
                self._calculate_package_metrics(element, elements)
        
        # Calculate class and function metrics
        for element in elements.values():
            if element.type in [NodeType.CLASS, NodeType.FUNCTION, NodeType.METHOD]:
                self._calculate_element_metrics(element, elements)
    
    def _calculate_package_metrics(self, package_element: CodeElement, 
                                  all_elements: Dict[str, CodeElement]) -> None:
        """Calculate aggregated metrics for a package."""
        child_files = [e for e in all_elements.values() 
                      if e.parent_id == package_element.id and e.type == NodeType.FILE]
        
        package_element.lines_of_code = sum(f.lines_of_code for f in child_files)
        complexities = [f.complexity for f in child_files if f.complexity]
        package_element.complexity = sum(complexities) / len(complexities) if complexities else 0
    
    def _calculate_element_metrics(self, element: CodeElement, 
                                  all_elements: Dict[str, CodeElement]) -> None:
        """Calculate metrics for individual code elements."""
        # Metrics would be calculated based on AST analysis
        # For now, set default values
        if not element.complexity:
            element.complexity = 1.0
        if not element.lines_of_code:
            element.lines_of_code = 1


class ProjectBuilder:
    """Main orchestrator for building hierarchical project representation."""
    
    def __init__(self, cache_manager=None, max_workers: Optional[int] = None):
        self.cache_manager = cache_manager
        self.cache = cache_manager  # Keep backwards compatibility
        self.max_workers = max_workers or 4
        self.hierarchy_analyzer = HierarchyAnalyzer()
        self.dependency_analyzer = DependencyAnalyzer()
        self.metrics_analyzer = MetricsAnalyzer()
    
    def build_project(self, project_name: str, root_path: Path, 
                     analysis_id: str) -> ProjectGraph:
        """Build complete hierarchical project representation."""
        logger.info(f"Building project graph for {project_name}")
        
        # Initialize project graph
        project_graph = ProjectGraph(
            project_id=analysis_id,
            name=project_name,
            root_path=str(root_path)
        )
        
        # 1. Discover and filter files
        files = self._discover_files(root_path)
        logger.info(f"Discovered {len(files)} files")
        
        # 2. Retrieve existing file analyses from cache (avoid re-analysis)
        file_analyses = self._retrieve_cached_analyses(files, analysis_id, root_path)
        logger.info(f"Retrieved {len(file_analyses)} cached file analyses")
        
        # 3. Build hierarchy (files and packages)
        hierarchy_elements = self.hierarchy_analyzer.analyze_file_hierarchy(files, root_path)
        
        # 4. Extract code elements from file analyses
        code_elements = self._extract_code_elements(file_analyses, hierarchy_elements, root_path)
        
        # 5. Combine all elements
        all_elements = {**hierarchy_elements, **code_elements}
        
        # 6. Analyze dependencies
        self.dependency_analyzer.analyze_dependencies(all_elements, file_analyses)
        
        # 7. Calculate metrics
        self.metrics_analyzer.calculate_metrics(all_elements, file_analyses)
        
        # 8. Populate project graph
        for element in all_elements.values():
            project_graph.add_element(element)
        
        logger.info(f"Project graph built with {len(all_elements)} elements")
        return project_graph
    
    def _discover_files(self, root_path: Path) -> List[Path]:
        """Discover all analyzable files in the project."""
        files = []
        excluded_dirs = {'.git', '__pycache__', 'node_modules', '.venv', 'venv', 'dist', 'build'}
        
        for file_path in root_path.rglob('*'):
            if file_path.is_file():
                # Skip if in excluded directory
                if any(part in excluded_dirs for part in file_path.parts):
                    continue
                
                # Check if file is analyzable
                if LanguageDetector.should_analyze_file(file_path):
                    files.append(file_path)
        
        return files
    
    def _retrieve_cached_analyses(self, files: List[Path], analysis_id: str, root_path: Path) -> Dict[str, Dict]:
        """Retrieve existing file analyses from cache instead of re-analyzing."""
        analyses = {}
        missing_files = []
        
        if not self.cache_manager:
            logger.warning("No cache manager available, falling back to re-analysis")
            return self._analyze_files_parallel_fallback(files, analysis_id)
        
        # First, try to get cached analyses
        for file_path in files:
            cache_key = f"file:{analysis_id}:{file_path.name}"
            
            cached_analysis = self.cache_manager.get(cache_key)
            if cached_analysis:
                # Convert cached dict back to analysis format
                analyses[str(file_path)] = self._convert_cached_to_analysis(cached_analysis, file_path)
            else:
                missing_files.append(file_path)
        
        # If we have missing files, try to analyze them
        if missing_files:
            logger.info(f"Re-analyzing {len(missing_files)} missing files")
            fallback_analyses = self._analyze_files_parallel_fallback(missing_files, analysis_id)
            analyses.update(fallback_analyses)
        
        return analyses
    
    def _convert_cached_to_analysis(self, cached_data: Dict, file_path: Path) -> Dict:
        """Convert cached analysis data to expected format."""
        # Handle both old FileAnalysis format and new UniversalFileAnalysis format
        if 'ast_nodes' in cached_data:
            # Old FileAnalysis format
            return {
                'language': LanguageDetector.detect_language(file_path).value,
                'classes': cached_data.get('classes', []),
                'functions': cached_data.get('functions', []),
                'variables': cached_data.get('variables', []),
                'imports': cached_data.get('imports', []),
                'complexity': cached_data.get('complexity', 1.0),
                'lines_of_code': cached_data.get('lines_of_code', cached_data.get('lines', 0)),  # Handle both names
                'ast_nodes': cached_data.get('ast_nodes', []),
                'file_path': str(file_path)
            }
        else:
            # New UniversalFileAnalysis format or direct dict
            return {
                'language': cached_data.get('language', LanguageDetector.detect_language(file_path).value),
                'classes': cached_data.get('classes', []),
                'functions': cached_data.get('functions', []), 
                'variables': cached_data.get('variables', []),
                'imports': cached_data.get('imports', []),
                'complexity': cached_data.get('complexity', 1.0),
                'lines_of_code': cached_data.get('lines_of_code', cached_data.get('lines', 0)),  # Handle both names
                'file_path': str(file_path)
            }
    
    def _analyze_files_parallel_fallback(self, files: List[Path], analysis_id: str) -> Dict[str, Dict]:
        """Fallback method to analyze files when cache is unavailable."""
        analyses = {}
        
        def analyze_single_file(file_path: Path) -> Optional[Dict]:
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                # Try universal analyzer first
                analysis = AnalyzerFactory.analyze_file_auto(file_path, content)
                if analysis:
                    return {
                        'language': analysis.language,
                        'classes': analysis.classes,
                        'functions': analysis.functions,
                        'variables': analysis.variables,
                        'imports': analysis.imports,
                        'complexity': analysis.complexity,
                        'lines_of_code': analysis.lines_of_code,
                        'file_path': str(file_path)
                    }
                else:
                    # Basic fallback for unsupported languages
                    return {
                        'language': LanguageDetector.detect_language(file_path).value,
                        'classes': [],
                        'functions': [],
                        'variables': [],
                        'imports': [],
                        'complexity': 1.0,
                        'lines_of_code': len(content.splitlines()),
                        'file_path': str(file_path)
                    }
            except Exception as e:
                logger.warning(f"Failed to analyze {file_path}: {e}")
                return None
        
        # Use ThreadPoolExecutor for parallel analysis
        with ThreadPoolExecutor(max_workers=4) as executor:
            future_to_file = {executor.submit(analyze_single_file, file_path): file_path 
                            for file_path in files}
            
            for future in as_completed(future_to_file):
                file_path = future_to_file[future]
                try:
                    analysis = future.result()
                    if analysis:
                        analyses[str(file_path)] = analysis
                except Exception as e:
                    logger.error(f"Error analyzing {file_path}: {e}")
        
        return analyses
    
    def _extract_code_elements(self, file_analyses: Dict[str, Dict], 
                              hierarchy_elements: Dict[str, CodeElement],
                              root_path: Path) -> Dict[str, CodeElement]:
        """Extract code elements (classes, functions, etc.) from file analyses."""
        elements = {}
        
        for file_path, analysis in file_analyses.items():
            file_id = f"file:{file_path}"
            
            # Create class elements
            for class_name in analysis.classes:
                class_id = f"class:{file_path}:{class_name}"
                class_element = CodeElement(
                    id=class_id,
                    name=class_name,
                    type=NodeType.CLASS,
                    language=analysis.language,
                    location=CodeLocation(
                        file_path=file_path,
                        line_start=1,  # Would need AST analysis for exact location
                        line_end=1,
                        col_start=0,
                        col_end=0
                    ),
                    parent_id=file_id,
                    scope=Scope.PUBLIC,  # Default, would need analysis
                    properties={'file_analysis_id': analysis.hash}
                )
                elements[class_id] = class_element
            
            # Create function elements
            for function_name in analysis.functions:
                function_id = f"function:{file_path}:{function_name}"
                
                # Determine if it's a method (belongs to a class)
                parent_id = file_id
                function_type = NodeType.FUNCTION
                
                # Simple heuristic: if there are classes in the file, assume methods
                if analysis.classes and function_name != '__init__':
                    # Find the most likely parent class (simple heuristic)
                    if analysis.classes:
                        parent_class = analysis.classes[0]  # Simplified
                        parent_id = f"class:{file_path}:{parent_class}"
                        function_type = NodeType.METHOD
                
                function_element = CodeElement(
                    id=function_id,
                    name=function_name,
                    type=function_type,
                    language=analysis.language,
                    location=CodeLocation(
                        file_path=file_path,
                        line_start=1,  # Would need AST analysis
                        line_end=1,
                        col_start=0,
                        col_end=0
                    ),
                    parent_id=parent_id,
                    scope=Scope.PUBLIC,
                    properties={'file_analysis_id': analysis.hash}
                )
                elements[function_id] = function_element
            
            # Create variable elements
            for variable_name in analysis.variables:
                variable_id = f"variable:{file_path}:{variable_name}"
                variable_element = CodeElement(
                    id=variable_id,
                    name=variable_name,
                    type=NodeType.VARIABLE,
                    language=analysis.language,
                    location=CodeLocation(
                        file_path=file_path,
                        line_start=1,  # Would need AST analysis
                        line_end=1,
                        col_start=0,
                        col_end=0
                    ),
                    parent_id=file_id,
                    scope=Scope.LOCAL,  # Default
                    properties={'file_analysis_id': analysis.hash}
                )
                elements[variable_id] = variable_element
        
        return elements
