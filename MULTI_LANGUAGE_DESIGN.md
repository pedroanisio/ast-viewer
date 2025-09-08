# Multi-Language AST Analyzer Design

## 🎯 Objective
Extend the AST viewer to support multiple programming languages using Tree-sitter for universal AST parsing.

## 🌍 Target Languages
- **Go** - `.go`
- **Rust** - `.rs`
- **C** - `.c`, `.h`
- **C++** - `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hxx`
- **Java** - `.java`
- **JavaScript** - `.js`, `.mjs`
- **TypeScript** - `.ts`
- **React/JSX** - `.jsx`, `.tsx`
- **CSS** - `.css`
- **HTML** - `.html`, `.htm`
- **Python** - `.py`, `.pyw` (existing)

## 🏗️ Architecture Design

### 1. Language Detection & Routing
```python
class LanguageDetector:
    LANGUAGE_MAP = {
        '.py': 'python',
        '.pyw': 'python',
        '.js': 'javascript',
        '.mjs': 'javascript',
        '.ts': 'typescript',
        '.tsx': 'typescript',
        '.jsx': 'javascript',
        '.go': 'go',
        '.rs': 'rust',
        '.c': 'c',
        '.h': 'c',
        '.cpp': 'cpp',
        '.cc': 'cpp',
        '.cxx': 'cpp',
        '.hpp': 'cpp',
        '.hxx': 'cpp',
        '.java': 'java',
        '.css': 'css',
        '.html': 'html',
        '.htm': 'html'
    }
```

### 2. Unified Analyzer Interface
```python
class LanguageAnalyzer:
    """Abstract base class for language-specific analyzers"""
    
    def analyze_file(self, file_path: Path, content: str) -> FileAnalysis:
        pass
    
    def extract_nodes(self, tree, source_code: str) -> List[ASTNode]:
        pass
    
    def get_complexity(self, tree, source_code: str) -> float:
        pass
```

### 3. Tree-sitter Integration
```python
class TreeSitterAnalyzer(LanguageAnalyzer):
    """Universal analyzer using Tree-sitter"""
    
    def __init__(self, language: str):
        self.language = language
        self.parser = self._get_parser(language)
    
    def _get_parser(self, language: str):
        # Load appropriate Tree-sitter parser
        pass
```

### 4. Fallback to Python AST
```python
class PythonASTAnalyzer(LanguageAnalyzer):
    """Enhanced Python analyzer using built-in ast module"""
    # Keep existing Python-specific logic for better analysis
```

## 📦 Dependencies to Add

### Core Tree-sitter
```bash
pip install tree-sitter
```

### Language Parsers
```bash
# JavaScript/TypeScript/JSX
pip install tree-sitter-javascript

# Go
pip install tree-sitter-go

# Rust  
pip install tree-sitter-rust

# C/C++
pip install tree-sitter-c tree-sitter-cpp

# Java
pip install tree-sitter-java

# CSS/HTML
pip install tree-sitter-css tree-sitter-html
```

## 🔄 Migration Strategy

### Phase 1: Foundation
1. ✅ Install Tree-sitter and language parsers
2. ✅ Create unified analyzer interface
3. ✅ Implement language detection
4. ✅ Basic Tree-sitter integration

### Phase 2: Core Languages
1. ✅ JavaScript/TypeScript support
2. ✅ Go support  
3. ✅ Rust support
4. ✅ C/C++ support

### Phase 3: Additional Languages
1. ✅ Java support
2. ✅ CSS/HTML support
3. ✅ Enhanced React/JSX support

### Phase 4: Optimization
1. ✅ Performance tuning
2. ✅ Caching improvements
3. ✅ Error handling
4. ✅ Frontend enhancements

## 🎨 Frontend Enhancements

### Extended Syntax Highlighting
Already partially implemented for:
- ✅ Python
- ✅ JavaScript/TypeScript/JSX/TSX  
- ⚠️ Generic (basic)

Need to add:
- 🔲 Go syntax highlighting
- 🔲 Rust syntax highlighting  
- 🔲 C/C++ syntax highlighting
- 🔲 Java syntax highlighting
- 🔲 CSS syntax highlighting
- 🔲 HTML syntax highlighting

### Language-Specific UI Features
- 🔲 Go package visualization
- 🔲 Rust module system
- 🔲 C/C++ header dependencies
- 🔲 Java class hierarchies
- 🔲 CSS selector analysis
- 🔲 HTML DOM structure

## 🚀 Benefits

1. **Universal Support** - Analyze any codebase regardless of language mix
2. **Consistent Interface** - Same UI for all languages
3. **Extensible** - Easy to add new languages
4. **Performance** - Tree-sitter is fast and incremental
5. **Accuracy** - Language-native parsers vs. regex-based

## ⚡ Quick Start Implementation

1. Add Tree-sitter dependencies to requirements.txt
2. Create `language_detector.py` 
3. Create `tree_sitter_analyzer.py`
4. Modify `ast_analyzer.py` to route by language
5. Update frontend syntax highlighting
6. Test with multi-language repositories

This design enables the AST viewer to become a truly universal code analysis tool! 🌟
