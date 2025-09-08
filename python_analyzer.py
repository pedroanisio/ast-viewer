"""
Enhanced Python AST Analyzer using built-in ast module
Maintains high-quality Python analysis while conforming to universal interface
"""
import ast
from pathlib import Path
from typing import Dict, List, Optional, Any, Set
import logging

from language_detector import Language
from language_analyzer import LanguageAnalyzer, UniversalFileAnalysis, UniversalASTNode

# Import radon for complexity analysis
try:
    import radon.complexity as radon_cc
    import radon.metrics as radon_metrics
    RADON_AVAILABLE = True
except ImportError:
    RADON_AVAILABLE = False

logger = logging.getLogger(__name__)


class PythonASTAnalyzer(LanguageAnalyzer):
    """Enhanced Python analyzer using built-in ast module."""
    
    def __init__(self):
        super().__init__(Language.PYTHON)
    
    def analyze_file(self, file_path: Path, content: str) -> Optional[UniversalFileAnalysis]:
        """Analyze Python file using built-in ast module."""
        try:
            # Parse AST
            tree = ast.parse(content, filename=str(file_path))
            
            # Extract comprehensive information
            nodes = self.extract_nodes(tree, content, str(file_path))
            imports = self.extract_imports(tree, content)
            exports = self.extract_exports(tree, content)  # Python doesn't have explicit exports
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
            
        except SyntaxError as e:
            logger.warning(f"Python syntax error in {file_path}: {e}")
            return None
        except Exception as e:
            logger.error(f"Python analysis failed for {file_path}: {e}")
            return None
    
    def extract_nodes(self, tree: ast.AST, source_code: str, file_path: str) -> List[UniversalASTNode]:
        """Extract comprehensive AST nodes from Python AST."""
        nodes = []
        source_lines = source_code.split('\n')
        
        class NodeVisitor(ast.NodeVisitor):
            def __init__(self):
                self.node_counter = 0
            
            def visit(self, node):
                if hasattr(node, 'lineno') and hasattr(node, 'col_offset'):
                    universal_node = self._create_universal_node(node)
                    if universal_node:
                        nodes.append(universal_node)
                
                self.generic_visit(node)
            
            def _create_universal_node(self, node: ast.AST) -> Optional[UniversalASTNode]:
                """Convert Python AST node to universal node."""
                self.node_counter += 1
                
                # Determine node type and name
                node_type, name = self._get_node_info(node)
                
                if not node_type:
                    return None
                
                # Get end position if available (Python 3.8+)
                end_line = getattr(node, 'end_lineno', None)
                end_col = getattr(node, 'end_col_offset', None)
                
                # Calculate complexity for specific nodes
                complexity = self._calculate_node_complexity(node)
                
                # Extract additional properties
                properties = self._extract_node_properties(node)
                
                return UniversalASTNode(
                    id=f"{file_path}:{node.lineno}:{node.col_offset}:{self.node_counter}",
                    type=node_type,
                    name=name,
                    file=file_path,
                    line=node.lineno,
                    col=node.col_offset,
                    end_line=end_line,
                    end_col=end_col,
                    children=[],  # Will be populated if needed
                    properties=properties,
                    language=Language.PYTHON.value,
                    complexity=complexity
                )
            
            def _get_node_info(self, node: ast.AST) -> tuple[str, Optional[str]]:
                """Get node type and name from Python AST node."""
                if isinstance(node, ast.FunctionDef):
                    return 'function', node.name
                elif isinstance(node, ast.AsyncFunctionDef):
                    return 'async_function', node.name
                elif isinstance(node, ast.ClassDef):
                    return 'class', node.name
                elif isinstance(node, ast.Import):
                    names = [alias.name for alias in node.names]
                    return 'import', ', '.join(names)
                elif isinstance(node, ast.ImportFrom):
                    module = node.module or ''
                    names = [alias.name for alias in node.names]
                    return 'import_from', f"from {module} import {', '.join(names)}"
                elif isinstance(node, ast.Assign):
                    targets = []
                    for target in node.targets:
                        if isinstance(target, ast.Name):
                            targets.append(target.id)
                        elif isinstance(target, ast.Attribute):
                            targets.append(f"{self._get_attr_name(target)}")
                    return 'assignment', ', '.join(targets)
                elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
                    return 'annotated_assignment', node.target.id
                elif isinstance(node, ast.AugAssign) and isinstance(node.target, ast.Name):
                    return 'augmented_assignment', node.target.id
                elif isinstance(node, ast.If):
                    return 'if_statement', None
                elif isinstance(node, ast.For):
                    return 'for_loop', None
                elif isinstance(node, ast.While):
                    return 'while_loop', None
                elif isinstance(node, ast.With):
                    return 'with_statement', None
                elif isinstance(node, ast.Try):
                    return 'try_statement', None
                elif isinstance(node, ast.Lambda):
                    return 'lambda', None
                elif isinstance(node, ast.ListComp):
                    return 'list_comprehension', None
                elif isinstance(node, ast.DictComp):
                    return 'dict_comprehension', None
                elif isinstance(node, ast.SetComp):
                    return 'set_comprehension', None
                elif isinstance(node, ast.GeneratorExp):
                    return 'generator_expression', None
                else:
                    # Skip other node types for now
                    return None, None
            
            def _get_attr_name(self, node: ast.Attribute) -> str:
                """Get full attribute name like obj.attr."""
                if isinstance(node.value, ast.Name):
                    return f"{node.value.id}.{node.attr}"
                elif isinstance(node.value, ast.Attribute):
                    return f"{self._get_attr_name(node.value)}.{node.attr}"
                else:
                    return node.attr
            
            def _calculate_node_complexity(self, node: ast.AST) -> Optional[int]:
                """Calculate complexity for specific node types."""
                # Control flow nodes add complexity
                if isinstance(node, (ast.If, ast.For, ast.While, ast.Try, ast.ExceptHandler)):
                    return 1
                elif isinstance(node, ast.FunctionDef):
                    # Function complexity is handled separately
                    return None
                else:
                    return None
            
            def _extract_node_properties(self, node: ast.AST) -> Dict[str, Any]:
                """Extract additional properties from Python AST node."""
                properties = {
                    'ast_type': type(node).__name__,
                }
                
                # Add specific properties based on node type
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    properties.update({
                        'is_async': isinstance(node, ast.AsyncFunctionDef),
                        'arg_count': len(node.args.args),
                        'has_defaults': len(node.args.defaults) > 0,
                        'has_vararg': node.args.vararg is not None,
                        'has_kwarg': node.args.kwarg is not None,
                        'has_decorators': len(node.decorator_list) > 0,
                        'decorator_count': len(node.decorator_list),
                    })
                elif isinstance(node, ast.ClassDef):
                    properties.update({
                        'base_count': len(node.bases),
                        'has_decorators': len(node.decorator_list) > 0,
                        'decorator_count': len(node.decorator_list),
                    })
                elif isinstance(node, (ast.Import, ast.ImportFrom)):
                    properties.update({
                        'import_count': len(node.names),
                        'has_alias': any(alias.asname for alias in node.names),
                    })
                
                return properties
        
        # Visit all nodes
        visitor = NodeVisitor()
        visitor.visit(tree)
        
        return nodes
    
    def extract_imports(self, tree: ast.AST, source_code: str) -> List[str]:
        """Extract import statements from Python AST."""
        imports = []
        
        class ImportVisitor(ast.NodeVisitor):
            def visit_Import(self, node):
                for alias in node.names:
                    import_name = alias.asname if alias.asname else alias.name
                    imports.append(import_name)
            
            def visit_ImportFrom(self, node):
                module = node.module or ''
                for alias in node.names:
                    import_name = alias.asname if alias.asname else alias.name
                    full_name = f"{module}.{import_name}" if module else import_name
                    imports.append(full_name)
        
        visitor = ImportVisitor()
        visitor.visit(tree)
        return imports
    
    def extract_exports(self, tree: ast.AST, source_code: str) -> List[str]:
        """Extract exports (Python uses __all__ for explicit exports)."""
        exports = []
        
        class ExportVisitor(ast.NodeVisitor):
            def visit_Assign(self, node):
                # Look for __all__ assignments
                for target in node.targets:
                    if isinstance(target, ast.Name) and target.id == '__all__':
                        # Extract string literals from __all__ list
                        if isinstance(node.value, ast.List):
                            for elt in node.value.elts:
                                if isinstance(elt, ast.Constant) and isinstance(elt.value, str):
                                    exports.append(elt.value)
                                elif isinstance(elt, ast.Str):  # Python < 3.8
                                    exports.append(elt.s)
        
        visitor = ExportVisitor()
        visitor.visit(tree)
        return exports
    
    def extract_classes(self, tree: ast.AST, source_code: str) -> List[str]:
        """Extract class definitions from Python AST."""
        classes = []
        
        class ClassVisitor(ast.NodeVisitor):
            def visit_ClassDef(self, node):
                classes.append(node.name)
        
        visitor = ClassVisitor()
        visitor.visit(tree)
        return classes
    
    def extract_functions(self, tree: ast.AST, source_code: str) -> List[str]:
        """Extract function definitions from Python AST."""
        functions = []
        
        class FunctionVisitor(ast.NodeVisitor):
            def visit_FunctionDef(self, node):
                functions.append(node.name)
            
            def visit_AsyncFunctionDef(self, node):
                functions.append(node.name)
        
        visitor = FunctionVisitor()
        visitor.visit(tree)
        return functions
    
    def extract_variables(self, tree: ast.AST, source_code: str) -> List[str]:
        """Extract variable assignments from Python AST."""
        variables = set()  # Use set to avoid duplicates
        
        class VariableVisitor(ast.NodeVisitor):
            def visit_Assign(self, node):
                for target in node.targets:
                    if isinstance(target, ast.Name):
                        variables.add(target.id)
                    elif isinstance(target, ast.Tuple):
                        for elt in target.elts:
                            if isinstance(elt, ast.Name):
                                variables.add(elt.id)
            
            def visit_AnnAssign(self, node):
                if isinstance(node.target, ast.Name):
                    variables.add(node.target.id)
            
            def visit_AugAssign(self, node):
                if isinstance(node.target, ast.Name):
                    variables.add(node.target.id)
        
        visitor = VariableVisitor()
        visitor.visit(tree)
        return list(variables)
    
    def calculate_complexity(self, tree: ast.AST, source_code: str) -> float:
        """Calculate cyclomatic complexity using Radon or fallback method."""
        if RADON_AVAILABLE:
            try:
                # Use Radon for accurate complexity calculation
                complexity_data = radon_cc.cc_visit(tree)
                if complexity_data:
                    total_complexity = sum(item.complexity for item in complexity_data)
                    return float(total_complexity)
            except Exception as e:
                logger.warning(f"Radon complexity calculation failed: {e}")
        
        # Fallback: Manual complexity calculation
        complexity = 1  # Base complexity
        
        class ComplexityVisitor(ast.NodeVisitor):
            def visit_If(self, node):
                nonlocal complexity
                complexity += 1
                self.generic_visit(node)
            
            def visit_For(self, node):
                nonlocal complexity
                complexity += 1
                self.generic_visit(node)
            
            def visit_While(self, node):
                nonlocal complexity
                complexity += 1
                self.generic_visit(node)
            
            def visit_Try(self, node):
                nonlocal complexity
                complexity += 1
                self.generic_visit(node)
            
            def visit_ExceptHandler(self, node):
                nonlocal complexity
                complexity += 1
                self.generic_visit(node)
            
            def visit_Lambda(self, node):
                nonlocal complexity
                complexity += 1
                self.generic_visit(node)
            
            def visit_BoolOp(self, node):
                nonlocal complexity
                # Add complexity for boolean operators (and/or)
                complexity += len(node.values) - 1
                self.generic_visit(node)
        
        visitor = ComplexityVisitor()
        visitor.visit(tree)
        return float(complexity)
