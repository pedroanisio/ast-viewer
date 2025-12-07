"""
Language Detection and Routing for Multi-Language AST Analysis
"""
from pathlib import Path
from typing import Optional, Set
from enum import Enum


class Language(Enum):
    """Supported programming languages."""
    PYTHON = "python"
    JAVASCRIPT = "javascript"
    TYPESCRIPT = "typescript"
    GO = "go"
    RUST = "rust"
    C = "c"
    CPP = "cpp"
    JAVA = "java"
    CSS = "css"
    HTML = "html"
    UNKNOWN = "unknown"


class LanguageDetector:
    """Detects programming language from file extensions."""
    
    # Language mapping by file extension
    EXTENSION_MAP = {
        # Python
        '.py': Language.PYTHON,
        '.pyw': Language.PYTHON,
        
        # JavaScript
        '.js': Language.JAVASCRIPT,
        '.mjs': Language.JAVASCRIPT,
        '.jsx': Language.JAVASCRIPT,  # React JSX
        
        # TypeScript
        '.ts': Language.TYPESCRIPT,
        '.tsx': Language.TYPESCRIPT,  # React TSX
        
        # Go
        '.go': Language.GO,
        
        # Rust
        '.rs': Language.RUST,
        
        # C
        '.c': Language.C,
        '.h': Language.C,
        
        # C++
        '.cpp': Language.CPP,
        '.cc': Language.CPP,
        '.cxx': Language.CPP,
        '.hpp': Language.CPP,
        '.hxx': Language.CPP,
        '.hh': Language.CPP,
        
        # Java
        '.java': Language.JAVA,
        
        # CSS
        '.css': Language.CSS,
        '.scss': Language.CSS,  # Sass
        '.sass': Language.CSS,  # Sass
        '.less': Language.CSS,  # Less
        
        # HTML
        '.html': Language.HTML,
        '.htm': Language.HTML,
        '.xhtml': Language.HTML,
    }
    
    # Languages that support full AST analysis
    AST_SUPPORTED = {
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
    
    # Languages that support syntax highlighting only
    SYNTAX_ONLY = {
        Language.UNKNOWN,
    }
    
    @classmethod
    def detect_language(cls, file_path: Path) -> Language:
        """
        Detect programming language from file extension.
        
        Args:
            file_path: Path to the file
            
        Returns:
            Detected language enum
        """
        extension = file_path.suffix.lower()
        return cls.EXTENSION_MAP.get(extension, Language.UNKNOWN)
    
    @classmethod
    def is_supported(cls, language: Language) -> bool:
        """Check if language is supported for AST analysis."""
        return language in cls.AST_SUPPORTED
    
    @classmethod
    def get_supported_extensions(cls) -> Set[str]:
        """Get all supported file extensions."""
        return set(cls.EXTENSION_MAP.keys())
    
    @classmethod
    def get_extensions_for_language(cls, language: Language) -> Set[str]:
        """Get file extensions for a specific language."""
        return {ext for ext, lang in cls.EXTENSION_MAP.items() if lang == language}
    
    @classmethod
    def should_analyze_file(cls, file_path: Path) -> bool:
        """
        Check if file should be analyzed based on language support.
        
        Args:
            file_path: Path to the file
            
        Returns:
            True if file should be analyzed
        """
        language = cls.detect_language(file_path)
        return cls.is_supported(language)
    
    @classmethod
    def get_language_info(cls, file_path: Path) -> dict:
        """
        Get comprehensive language information for a file.
        
        Args:
            file_path: Path to the file
            
        Returns:
            Dictionary with language information
        """
        language = cls.detect_language(file_path)
        return {
            'language': language.value,
            'supported': cls.is_supported(language),
            'extension': file_path.suffix.lower(),
            'analyzer_type': 'tree-sitter' if language != Language.PYTHON else 'python-ast',
            'syntax_highlighting': True  # All languages support syntax highlighting
        }


# Language-specific configuration
LANGUAGE_CONFIG = {
    Language.PYTHON: {
        'name': 'Python',
        'icon': '🐍',
        'analyzer': 'python-ast',  # Use built-in AST for better Python analysis
        'complexity_supported': True,
        'package_manager': ['pip', 'poetry', 'pipenv'],
    },
    Language.JAVASCRIPT: {
        'name': 'JavaScript',
        'icon': '🟨',
        'analyzer': 'tree-sitter',
        'complexity_supported': True,
        'package_manager': ['npm', 'yarn', 'pnpm'],
    },
    Language.TYPESCRIPT: {
        'name': 'TypeScript', 
        'icon': '🔷',
        'analyzer': 'tree-sitter',
        'complexity_supported': True,
        'package_manager': ['npm', 'yarn', 'pnpm'],
    },
    Language.GO: {
        'name': 'Go',
        'icon': '🐹',
        'analyzer': 'tree-sitter',
        'complexity_supported': True,
        'package_manager': ['go mod'],
    },
    Language.RUST: {
        'name': 'Rust',
        'icon': '🦀',
        'analyzer': 'tree-sitter', 
        'complexity_supported': True,
        'package_manager': ['cargo'],
    },
    Language.C: {
        'name': 'C',
        'icon': '⚡',
        'analyzer': 'tree-sitter',
        'complexity_supported': True,
        'package_manager': ['make', 'cmake'],
    },
    Language.CPP: {
        'name': 'C++',
        'icon': '⚡',
        'analyzer': 'tree-sitter',
        'complexity_supported': True,
        'package_manager': ['make', 'cmake', 'conan'],
    },
    Language.JAVA: {
        'name': 'Java',
        'icon': '☕',
        'analyzer': 'tree-sitter',
        'complexity_supported': True,
        'package_manager': ['maven', 'gradle'],
    },
    Language.CSS: {
        'name': 'CSS',
        'icon': '🎨',
        'analyzer': 'tree-sitter',
        'complexity_supported': False,
        'package_manager': ['npm', 'yarn'],
    },
    Language.HTML: {
        'name': 'HTML',
        'icon': '🌐',
        'analyzer': 'tree-sitter',
        'complexity_supported': False,
        'package_manager': None,
    },
}


def get_language_config(language: Language) -> dict:
    """Get configuration for a specific language."""
    return LANGUAGE_CONFIG.get(language, {
        'name': 'Unknown',
        'icon': '❓',
        'analyzer': 'none',
        'complexity_supported': False,
        'package_manager': None,
    })
