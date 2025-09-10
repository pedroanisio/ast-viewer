"""
Unified Language Analyzer Interface for Multi-Language AST Support
"""
from abc import ABC, abstractmethod
from pathlib import Path
from typing import Dict, List, Optional, Any, Set, Tuple
from dataclasses import dataclass
import logging
import hashlib

from language_detector import Language, LanguageDetector

logger = logging.getLogger(__name__)


@dataclass 
class UniversalASTNode:
    """Universal AST node representation across all languages."""
    id: str
    type: str  # function, class, variable, import, etc.
    name: Optional[str]
    file: str
    line: int
    col: int
    end_line: Optional[int] = None
    end_col: Optional[int] = None
    children: List[str] = None
    properties: Dict[str, Any] = None
    language: str = None
    complexity: Optional[int] = None
    
    def __post_init__(self):
        if self.children is None:
            self.children = []
        if self.properties is None:
            self.properties = {}
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            'id': self.id,
            'type': self.type,
            'name': self.name,
            'file': self.file,
            'line': self.line,
            'col': self.col,
            'end_line': self.end_line,
            'end_col': self.end_col,
            'children': self.children,
            'properties': self.properties,
            'language': self.language,
            'complexity': self.complexity
        }


@dataclass
class UniversalFileAnalysis:
    """Universal file analysis result across all languages."""
    path: str
    language: str
    nodes: List[UniversalASTNode]
    imports: List[str]
    exports: List[str]  # For languages that support exports
    classes: List[str]
    functions: List[str]
    variables: List[str]
    complexity: float
    lines: int
    hash: str
    size_bytes: int
    encoding: str = 'utf-8'
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            'path': self.path,
            'language': self.language,
            'nodes': [node.to_dict() for node in self.nodes],
            'imports': self.imports,
            'exports': self.exports,
            'classes': self.classes,
            'functions': self.functions,
            'variables': self.variables,
            'complexity': self.complexity,
            'lines': self.lines,
            'hash': self.hash,
            'size_bytes': self.size_bytes,
            'encoding': self.encoding
        }


class LanguageAnalyzer(ABC):
    """Abstract base class for language-specific analyzers."""
    
    def __init__(self, language: Language):
        self.language = language
        self.supported_extensions = LanguageDetector.get_extensions_for_language(language)
    
    @abstractmethod
    def analyze_file(self, file_path: Path, content: str) -> Optional[UniversalFileAnalysis]:
        """
        Analyze a single file and return universal analysis result.
        
        Args:
            file_path: Path to the file
            content: File content as string
            
        Returns:
            Universal file analysis result or None if failed
        """
        pass
    
    @abstractmethod
    def extract_nodes(self, tree: Any, source_code: str, file_path: str) -> List[UniversalASTNode]:
        """
        Extract AST nodes from parse tree.
        
        Args:
            tree: Language-specific parse tree
            source_code: Original source code
            file_path: Path to source file
            
        Returns:
            List of universal AST nodes
        """
        pass
    
    def calculate_complexity(self, tree: Any, source_code: str) -> float:
        """
        Calculate code complexity (default implementation).
        
        Args:
            tree: Language-specific parse tree  
            source_code: Original source code
            
        Returns:
            Complexity score
        """
        # Default: simple line-based complexity
        lines = source_code.count('\n') + 1
        return max(1.0, lines / 10.0)
    
    def extract_imports(self, tree: Any, source_code: str) -> List[str]:
        """Extract import statements (to be overridden by subclasses)."""
        return []
    
    def extract_exports(self, tree: Any, source_code: str) -> List[str]:
        """Extract export statements (to be overridden by subclasses).""" 
        return []
    
    def extract_classes(self, tree: Any, source_code: str) -> List[str]:
        """Extract class definitions (to be overridden by subclasses)."""
        return []
    
    def extract_functions(self, tree: Any, source_code: str) -> List[str]:
        """Extract function definitions (to be overridden by subclasses)."""
        return []
    
    def extract_variables(self, tree: Any, source_code: str) -> List[str]:
        """Extract variable declarations (to be overridden by subclasses)."""
        return []
    
    def is_supported_file(self, file_path: Path) -> bool:
        """Check if file extension is supported by this analyzer."""
        return file_path.suffix.lower() in self.supported_extensions
    
    def create_file_hash(self, content: str) -> str:
        """Create file content hash."""
        return hashlib.md5(content.encode()).hexdigest()


class AnalyzerFactory:
    """Factory for creating language-specific analyzers."""
    
    _analyzers: Dict[Language, LanguageAnalyzer] = {}
    
    @classmethod
    def get_analyzer(cls, language: Language) -> Optional[LanguageAnalyzer]:
        """
        Get analyzer for specific language.
        
        Args:
            language: Programming language
            
        Returns:
            Language analyzer instance or None if not supported
        """
        if language not in cls._analyzers:
            cls._analyzers[language] = cls._create_analyzer(language)
        
        return cls._analyzers[language]
    
    @classmethod
    def _create_analyzer(cls, language: Language) -> Optional[LanguageAnalyzer]:
        """Create analyzer instance for language."""
        try:
            if language == Language.PYTHON:
                from python_analyzer import PythonASTAnalyzer
                return PythonASTAnalyzer()
            elif language in [Language.JAVASCRIPT, Language.TYPESCRIPT]:
                from tree_sitter_analyzer import TreeSitterAnalyzer
                return TreeSitterAnalyzer(language)
            elif language in [Language.GO, Language.RUST, Language.C, Language.CPP, Language.JAVA]:
                from tree_sitter_analyzer import TreeSitterAnalyzer
                return TreeSitterAnalyzer(language)
            elif language in [Language.CSS, Language.HTML]:
                from tree_sitter_analyzer import TreeSitterAnalyzer
                return TreeSitterAnalyzer(language)
            else:
                logger.warning(f"No analyzer available for language: {language}")
                return None
                
        except ImportError as e:
            logger.error(f"Failed to import analyzer for {language}: {e}")
            return None
    
    @classmethod
    def get_supported_languages(cls) -> Set[Language]:
        """Get all supported languages."""
        return {
            Language.PYTHON,
            Language.JAVASCRIPT,
            Language.TYPESCRIPT,
            Language.GO,
            Language.RUST,
            Language.C,
            Language.CPP,
            Language.JAVA,
            Language.CSS,
            Language.HTML,
        }
    
    @classmethod
    def analyze_file_auto(cls, file_path: Path, content: str) -> Optional[UniversalFileAnalysis]:
        """
        Automatically detect language and analyze file.
        
        Args:
            file_path: Path to file
            content: File content
            
        Returns:
            Analysis result or None if unsupported/failed
        """
        language = LanguageDetector.detect_language(file_path)
        
        if not LanguageDetector.is_supported(language):
            logger.debug(f"Language {language} not supported for file: {file_path}")
            return None
        
        analyzer = cls.get_analyzer(language)
        if not analyzer:
            logger.warning(f"No analyzer available for {language}: {file_path}")
            return None
        
        try:
            return analyzer.analyze_file(file_path, content)
        except Exception as e:
            logger.error(f"Analysis failed for {file_path}: {e}")
            return None


# Utility functions for backward compatibility and easy integration
def analyze_file_universal(file_path: Path, content: str) -> Optional[UniversalFileAnalysis]:
    """Universal file analysis function."""
    return AnalyzerFactory.analyze_file_auto(file_path, content)


def get_supported_extensions() -> Set[str]:
    """Get all supported file extensions."""
    return LanguageDetector.get_supported_extensions()


def is_file_supported(file_path: Path) -> bool:
    """Check if file is supported for analysis."""
    return LanguageDetector.should_analyze_file(file_path)
