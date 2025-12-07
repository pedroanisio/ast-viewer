"""
Tree-sitter based Universal Language Analyzer
"""
from pathlib import Path
from typing import Dict, List, Optional, Any, Set
import logging
import re

from language_detector import Language
from language_analyzer import LanguageAnalyzer, UniversalFileAnalysis, UniversalASTNode

logger = logging.getLogger(__name__)

# Tree-sitter imports with fallback
try:
    import tree_sitter
    from tree_sitter import Language as TSLanguage, Parser
    TREE_SITTER_AVAILABLE = True
except ImportError:
    TREE_SITTER_AVAILABLE = False
    logger.warning("Tree-sitter not available, multi-language support disabled")


class TreeSitterAnalyzer(LanguageAnalyzer):
    """Universal analyzer using Tree-sitter for multiple languages."""
    
    # Tree-sitter language loading map
    LANGUAGE_FILES = {
        Language.JAVASCRIPT: 'tree-sitter-javascript',
        Language.TYPESCRIPT: 'tree-sitter-typescript',
        Language.GO: 'tree-sitter-go', 
        Language.RUST: 'tree-sitter-rust',
        Language.C: 'tree-sitter-c',
        Language.CPP: 'tree-sitter-cpp',
        Language.JAVA: 'tree-sitter-java',
        Language.CSS: 'tree-sitter-css',
        Language.HTML: 'tree-sitter-html',
    }
    
    # Node type mappings for different languages
    NODE_TYPE_MAP = {
        # JavaScript/TypeScript
        Language.JAVASCRIPT: {
            'function_declaration': 'function',
            'function_expression': 'function',
            'arrow_function': 'function',
            'method_definition': 'method',
            'class_declaration': 'class',
            'variable_declaration': 'variable',
            'import_statement': 'import',
            'export_statement': 'export',
        },
        Language.TYPESCRIPT: {
            'function_declaration': 'function',
            'function_expression': 'function', 
            'arrow_function': 'function',
            'method_definition': 'method',
            'class_declaration': 'class',
            'interface_declaration': 'interface',
            'type_alias_declaration': 'type',
            'variable_declaration': 'variable',
            'import_statement': 'import',
            'export_statement': 'export',
        },
        # Go
        Language.GO: {
            'function_declaration': 'function',
            'method_declaration': 'method',
            'type_declaration': 'type',
            'var_declaration': 'variable',
            'const_declaration': 'constant',
            'import_declaration': 'import',
            'package_clause': 'package',
        },
        # Rust
        Language.RUST: {
            'function_item': 'function',
            'impl_item': 'implementation',
            'struct_item': 'struct',
            'enum_item': 'enum',
            'trait_item': 'trait',
            'let_declaration': 'variable',
            'use_declaration': 'import',
            'mod_item': 'module',
        },
        # C/C++
        Language.C: {
            'function_definition': 'function',
            'declaration': 'declaration',
            'struct_specifier': 'struct',
            'enum_specifier': 'enum',
            'typedef_declaration': 'typedef',
            'preproc_include': 'include',
        },
        Language.CPP: {
            'function_definition': 'function',
            'declaration': 'declaration',
            'class_specifier': 'class',
            'struct_specifier': 'struct',
            'enum_specifier': 'enum',
            'namespace_definition': 'namespace',
            'template_declaration': 'template',
            'preproc_include': 'include',
        },
        # Java
        Language.JAVA: {
            'method_declaration': 'method',
            'class_declaration': 'class',
            'interface_declaration': 'interface',
            'variable_declaration': 'variable',
            'import_declaration': 'import',
            'package_declaration': 'package',
        },
        # CSS
        Language.CSS: {
            'rule_set': 'rule',
            'at_rule': 'at_rule',
            'declaration': 'property',
            'import_statement': 'import',
        },
        # HTML
        Language.HTML: {
            'element': 'element',
            'start_tag': 'start_tag',
            'end_tag': 'end_tag',
            'attribute': 'attribute',
            'text': 'text',
        },
    }
    
    def __init__(self, language: Language):
        super().__init__(language)
        self.parser = None
        self.ts_language = None
        
        if TREE_SITTER_AVAILABLE:
            self._initialize_parser()
    
    def _initialize_parser(self):
        """Initialize Tree-sitter parser for the language."""
        try:
            if self.language not in self.LANGUAGE_FILES:
                logger.error(f"No Tree-sitter support for language: {self.language}")
                return
            
            # Load language library from Python package
            language_file = self.LANGUAGE_FILES[self.language]
            
            try:
                # Import the language module
                import importlib
                module_name = language_file.replace('-', '_').replace('/', '.')
                module = importlib.import_module(module_name)
                
                # Get the language object - handle PyCapsule properly
                language_capsule = module.language()
                
                # Create Language object from capsule
                from tree_sitter import Language
                self.ts_language = Language(language_capsule)
                
                # Create parser and set language
                self.parser = Parser()
                self.parser.language = self.ts_language
                
                logger.info(f"Tree-sitter parser initialized for {self.language}")
                
            except Exception as e:
                logger.warning(f"Failed to load Tree-sitter language {language_file}: {e}")
                self.parser = None
                self.ts_language = None
            
        except Exception as e:
            logger.error(f"Failed to initialize Tree-sitter parser for {self.language}: {e}")
            self.parser = None
            self.ts_language = None
    
    def analyze_file(self, file_path: Path, content: str) -> Optional[UniversalFileAnalysis]:
        """Analyze file using Tree-sitter."""
        if not self.parser:
            logger.warning(f"No parser available for {self.language}")
            return None
        
        try:
            # Parse the content
            tree = self.parser.parse(content.encode('utf-8'))
            
            if not tree.root_node:
                logger.warning(f"Failed to parse {file_path}")
                return None
            
            # Extract information
            nodes = self.extract_nodes(tree, content, str(file_path))
            imports = self.extract_imports(tree, content)
            exports = self.extract_exports(tree, content)
            classes = self.extract_classes(tree, content)
            functions = self.extract_functions(tree, content)
            variables = self.extract_variables(tree, content)
            complexity = self.calculate_complexity(tree, content)
            
            # File metadata
            lines = content.count('\n') + 1
            file_hash = self.create_file_hash(content)
            size_bytes = len(content.encode('utf-8'))
            
            return UniversalFileAnalysis(
                path=str(file_path),
                language=self.language.value,
                nodes=nodes,
                imports=imports,
                exports=exports,
                classes=classes,
                functions=functions,
                variables=variables,
                complexity=complexity,
                lines=lines,
                hash=file_hash,
                size_bytes=size_bytes,
                encoding='utf-8'
            )
            
        except Exception as e:
            logger.error(f"Tree-sitter analysis failed for {file_path}: {e}")
            return None
    
    def extract_nodes(self, tree: Any, source_code: str, file_path: str) -> List[UniversalASTNode]:
        """Extract AST nodes from Tree-sitter parse tree."""
        nodes = []
        source_lines = source_code.split('\n')
        
        def traverse_node(node, parent_id=None):
            # Get node text
            node_text = source_code[node.start_byte:node.end_byte]
            
            # Map Tree-sitter node type to universal type
            node_type_map = self.NODE_TYPE_MAP.get(self.language, {})
            universal_type = node_type_map.get(node.type, node.type)
            
            # Extract name (first identifier child usually)
            name = self._extract_node_name(node, source_code)
            
            # Create universal node
            universal_node = UniversalASTNode(
                id=f"{file_path}:{node.start_point.row}:{node.start_point.column}",
                type=universal_type,
                name=name,
                file=file_path,
                line=node.start_point.row + 1,  # Tree-sitter uses 0-based indexing
                col=node.start_point.column,
                end_line=node.end_point.row + 1,
                end_col=node.end_point.column,
                children=[],
                properties={
                    'raw_type': node.type,
                    'text_length': len(node_text),
                    'has_children': len(node.children) > 0,
                },
                language=self.language.value
            )
            
            nodes.append(universal_node)
            
            # Process children
            for child in node.children:
                child_id = traverse_node(child, universal_node.id)
                if child_id:
                    universal_node.children.append(child_id)
            
            return universal_node.id
        
        # Start traversal from root
        if tree.root_node:
            traverse_node(tree.root_node)
        
        return nodes
    
    def _extract_node_name(self, node, source_code: str) -> Optional[str]:
        """Extract name from Tree-sitter node."""
        # Language-specific name extraction logic
        if self.language in [Language.JAVASCRIPT, Language.TYPESCRIPT]:
            return self._extract_js_name(node, source_code)
        elif self.language == Language.GO:
            return self._extract_go_name(node, source_code)
        elif self.language == Language.RUST:
            return self._extract_rust_name(node, source_code)
        elif self.language in [Language.C, Language.CPP]:
            return self._extract_c_name(node, source_code)
        elif self.language == Language.JAVA:
            return self._extract_java_name(node, source_code)
        else:
            # Generic name extraction
            for child in node.children:
                if child.type == 'identifier':
                    return source_code[child.start_byte:child.end_byte]
            return None
    
    def _extract_js_name(self, node, source_code: str) -> Optional[str]:
        """Extract name from JavaScript/TypeScript node."""
        if node.type in ['function_declaration', 'class_declaration']:
            for child in node.children:
                if child.type == 'identifier':
                    return source_code[child.start_byte:child.end_byte]
        return None
    
    def _extract_go_name(self, node, source_code: str) -> Optional[str]:
        """Extract name from Go node.""" 
        if node.type in ['function_declaration', 'type_declaration']:
            for child in node.children:
                if child.type == 'identifier':
                    return source_code[child.start_byte:child.end_byte]
        return None
    
    def _extract_rust_name(self, node, source_code: str) -> Optional[str]:
        """Extract name from Rust node."""
        if node.type.endswith('_item'):
            for child in node.children:
                if child.type == 'identifier':
                    return source_code[child.start_byte:child.end_byte]
        return None
    
    def _extract_c_name(self, node, source_code: str) -> Optional[str]:
        """Extract name from C/C++ node."""
        if node.type in ['function_definition', 'declaration']:
            # C/C++ naming is more complex, need to find the right identifier
            for child in node.children:
                if child.type == 'identifier':
                    return source_code[child.start_byte:child.end_byte]
        return None
    
    def _extract_java_name(self, node, source_code: str) -> Optional[str]:
        """Extract name from Java node."""
        if node.type in ['method_declaration', 'class_declaration']:
            for child in node.children:
                if child.type == 'identifier':
                    return source_code[child.start_byte:child.end_byte]
        return None
    
    def extract_imports(self, tree: Any, source_code: str) -> List[str]:
        """Extract import statements."""
        imports = []
        
        def find_imports(node):
            if node.type in ['import_statement', 'import_declaration', 'use_declaration', 'preproc_include']:
                import_text = source_code[node.start_byte:node.end_byte].strip()
                imports.append(import_text)
            
            for child in node.children:
                find_imports(child)
        
        find_imports(tree.root_node)
        return imports
    
    def extract_exports(self, tree: Any, source_code: str) -> List[str]:
        """Extract export statements."""
        exports = []
        
        def find_exports(node):
            if node.type in ['export_statement', 'export_declaration']:
                export_text = source_code[node.start_byte:node.end_byte].strip()
                exports.append(export_text)
            
            for child in node.children:
                find_exports(child)
        
        find_exports(tree.root_node)
        return exports
    
    def extract_classes(self, tree: Any, source_code: str) -> List[str]:
        """Extract class definitions."""
        classes = []
        
        def find_classes(node):
            if node.type in ['class_declaration', 'class_specifier', 'struct_item']:
                name = self._extract_node_name(node, source_code)
                if name:
                    classes.append(name)
            
            for child in node.children:
                find_classes(child)
        
        find_classes(tree.root_node)
        return classes
    
    def extract_functions(self, tree: Any, source_code: str) -> List[str]:
        """Extract function definitions."""
        functions = []
        
        def find_functions(node):
            if node.type in ['function_declaration', 'function_definition', 'function_item', 'method_declaration']:
                name = self._extract_node_name(node, source_code)
                if name:
                    functions.append(name)
            
            for child in node.children:
                find_functions(child)
        
        find_functions(tree.root_node)
        return functions
    
    def extract_variables(self, tree: Any, source_code: str) -> List[str]:
        """Extract variable declarations."""
        variables = []
        
        def find_variables(node):
            if node.type in ['variable_declaration', 'var_declaration', 'let_declaration']:
                # Extract variable names (might be multiple in one declaration)
                for child in node.children:
                    if child.type == 'identifier':
                        var_name = source_code[child.start_byte:child.end_byte]
                        variables.append(var_name)
            
            for child in node.children:
                find_variables(child)
        
        find_variables(tree.root_node)
        return variables
    
    def calculate_complexity(self, tree: Any, source_code: str) -> float:
        """Calculate cyclomatic complexity using Tree-sitter."""
        complexity = 1  # Base complexity
        
        # Language-specific complexity nodes
        complexity_nodes = {
            Language.JAVASCRIPT: ['if_statement', 'while_statement', 'for_statement', 'switch_statement', 'catch_clause'],
            Language.TYPESCRIPT: ['if_statement', 'while_statement', 'for_statement', 'switch_statement', 'catch_clause'],
            Language.GO: ['if_statement', 'for_statement', 'switch_statement', 'type_switch_statement'],
            Language.RUST: ['if_expression', 'while_expression', 'for_expression', 'match_expression'],
            Language.C: ['if_statement', 'while_statement', 'for_statement', 'switch_statement'],
            Language.CPP: ['if_statement', 'while_statement', 'for_statement', 'switch_statement', 'try_statement'],
            Language.JAVA: ['if_statement', 'while_statement', 'for_statement', 'switch_statement', 'catch_clause'],
        }
        
        target_nodes = complexity_nodes.get(self.language, [])
        
        def count_complexity(node):
            nonlocal complexity
            if node.type in target_nodes:
                complexity += 1
            
            for child in node.children:
                count_complexity(child)
        
        count_complexity(tree.root_node)
        return float(complexity)
