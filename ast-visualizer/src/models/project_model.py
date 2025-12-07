"""
Project Model - Core data structures for hierarchical code navigation
"""
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Set, Any, Union
from enum import Enum
from pathlib import Path
import json


class NodeType(Enum):
    """Types of code elements in the project hierarchy."""
    PROJECT = "project"
    PACKAGE = "package"
    MODULE = "module"  
    FILE = "file"
    CLASS = "class"
    INTERFACE = "interface"
    FUNCTION = "function"
    METHOD = "method"
    VARIABLE = "variable"
    CONSTANT = "constant"
    IMPORT = "import"
    EXPORT = "export"


class Scope(Enum):
    """Visibility/scope levels."""
    PUBLIC = "public"
    PRIVATE = "private"
    PROTECTED = "protected"
    INTERNAL = "internal"
    GLOBAL = "global"
    LOCAL = "local"


@dataclass
class CodeLocation:
    """Precise location of code element."""
    file_path: str
    line_start: int
    line_end: int
    col_start: int
    col_end: int
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'file_path': self.file_path,
            'line_start': self.line_start,
            'line_end': self.line_end,
            'col_start': self.col_start,
            'col_end': self.col_end
        }


@dataclass
class CodeElement:
    """Universal code element representing any node in the project hierarchy."""
    id: str                              # Unique identifier
    name: str                           # Element name
    type: NodeType                      # Element type
    language: str                       # Programming language
    location: CodeLocation              # Source location
    parent_id: Optional[str] = None     # Parent element ID
    children_ids: Set[str] = field(default_factory=set)  # Child element IDs
    
    # Element-specific properties
    scope: Optional[Scope] = None       # Visibility scope
    is_static: bool = False            # Static/class-level element
    is_async: bool = False             # Async function/method
    is_abstract: bool = False          # Abstract class/method
    return_type: Optional[str] = None   # Function return type
    parameters: List[Dict[str, Any]] = field(default_factory=list)  # Function parameters
    
    # Code metrics
    complexity: Optional[float] = None  # Cyclomatic complexity
    lines_of_code: int = 0             # Lines of code
    
    # Documentation and annotations
    docstring: Optional[str] = None    # Documentation string
    annotations: Dict[str, Any] = field(default_factory=dict)  # Type annotations, decorators
    
    # Relationships
    depends_on: Set[str] = field(default_factory=set)     # Direct dependencies
    used_by: Set[str] = field(default_factory=set)        # Elements that use this
    implements: Set[str] = field(default_factory=set)     # Interfaces implemented
    extends: Optional[str] = None                          # Parent class/interface
    
    # Custom properties by language
    properties: Dict[str, Any] = field(default_factory=dict)
    
    def add_child(self, child_id: str):
        """Add a child element."""
        self.children_ids.add(child_id)
    
    def add_dependency(self, dependency_id: str):
        """Add a dependency relationship."""
        self.depends_on.add(dependency_id)
    
    def add_usage(self, user_id: str):
        """Add a usage relationship."""
        self.used_by.add(user_id)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            'id': self.id,
            'name': self.name,
            'type': self.type.value,
            'language': self.language,
            'location': self.location.to_dict(),
            'parent_id': self.parent_id,
            'children_ids': list(self.children_ids),
            'scope': self.scope.value if self.scope else None,
            'is_static': self.is_static,
            'is_async': self.is_async,
            'is_abstract': self.is_abstract,
            'return_type': self.return_type,
            'parameters': self.parameters,
            'complexity': self.complexity,
            'lines_of_code': self.lines_of_code,
            'docstring': self.docstring,
            'annotations': self.annotations,
            'depends_on': list(self.depends_on),
            'used_by': list(self.used_by),
            'implements': list(self.implements),
            'extends': self.extends,
            'properties': self.properties
        }


@dataclass
class ProjectGraph:
    """Complete project representation with hierarchical and dependency graphs."""
    project_id: str
    name: str
    root_path: str
    languages: Set[str] = field(default_factory=set)
    
    # Core data structures
    elements: Dict[str, CodeElement] = field(default_factory=dict)  # All code elements
    
    # Hierarchical indexes for fast navigation
    files: Dict[str, str] = field(default_factory=dict)           # file_path -> element_id
    packages: Dict[str, str] = field(default_factory=dict)        # package_name -> element_id
    classes: Dict[str, List[str]] = field(default_factory=dict)   # class_name -> [element_ids]
    functions: Dict[str, List[str]] = field(default_factory=dict) # function_name -> [element_ids]
    variables: Dict[str, List[str]] = field(default_factory=dict) # variable_name -> [element_ids]
    
    # Type-based indexes
    by_type: Dict[NodeType, Set[str]] = field(default_factory=dict)     # type -> element_ids
    by_language: Dict[str, Set[str]] = field(default_factory=dict)      # language -> element_ids
    by_file: Dict[str, Set[str]] = field(default_factory=dict)          # file_path -> element_ids
    
    # Dependency graphs
    dependency_graph: Dict[str, Set[str]] = field(default_factory=dict) # element_id -> dependencies
    usage_graph: Dict[str, Set[str]] = field(default_factory=dict)      # element_id -> usages
    
    # File tree structure
    file_tree: Dict[str, Any] = field(default_factory=dict)             # Hierarchical file tree
    
    def add_element(self, element: CodeElement):
        """Add a code element to the project graph."""
        self.elements[element.id] = element
        self.languages.add(element.language)
        
        # Update hierarchical indexes
        if element.type == NodeType.FILE:
            self.files[element.location.file_path] = element.id
        elif element.type == NodeType.PACKAGE:
            self.packages[element.name] = element.id
        elif element.type == NodeType.CLASS:
            if element.name not in self.classes:
                self.classes[element.name] = []
            self.classes[element.name].append(element.id)
        elif element.type in [NodeType.FUNCTION, NodeType.METHOD]:
            if element.name not in self.functions:
                self.functions[element.name] = []
            self.functions[element.name].append(element.id)
        elif element.type in [NodeType.VARIABLE, NodeType.CONSTANT]:
            if element.name not in self.variables:
                self.variables[element.name] = []
            self.variables[element.name].append(element.id)
        
        # Update type-based indexes
        if element.type not in self.by_type:
            self.by_type[element.type] = set()
        self.by_type[element.type].add(element.id)
        
        if element.language not in self.by_language:
            self.by_language[element.language] = set()
        self.by_language[element.language].add(element.id)
        
        if element.location.file_path not in self.by_file:
            self.by_file[element.location.file_path] = set()
        self.by_file[element.location.file_path].add(element.id)
        
        # Update dependency graphs
        self.dependency_graph[element.id] = element.depends_on.copy()
        self.usage_graph[element.id] = element.used_by.copy()
        
        # Update parent-child relationships
        if element.parent_id:
            parent = self.elements.get(element.parent_id)
            if parent:
                parent.add_child(element.id)
    
    def get_element(self, element_id: str) -> Optional[CodeElement]:
        """Get element by ID."""
        return self.elements.get(element_id)
    
    def get_children(self, element_id: str) -> List[CodeElement]:
        """Get direct children of an element."""
        element = self.get_element(element_id)
        if not element:
            return []
        return [self.elements[child_id] for child_id in element.children_ids if child_id in self.elements]
    
    def get_descendants(self, element_id: str) -> List[CodeElement]:
        """Get all descendants of an element (recursive)."""
        descendants = []
        
        def collect_descendants(elem_id: str):
            children = self.get_children(elem_id)
            descendants.extend(children)
            for child in children:
                collect_descendants(child.id)
        
        collect_descendants(element_id)
        return descendants
    
    def get_dependencies(self, element_id: str, recursive: bool = False) -> List[CodeElement]:
        """Get dependencies of an element."""
        if not recursive:
            dep_ids = self.dependency_graph.get(element_id, set())
            return [self.elements[dep_id] for dep_id in dep_ids if dep_id in self.elements]
        
        # Recursive dependencies (transitive closure)
        visited = set()
        dependencies = []
        
        def collect_deps(elem_id: str):
            if elem_id in visited:
                return
            visited.add(elem_id)
            
            for dep_id in self.dependency_graph.get(elem_id, set()):
                if dep_id in self.elements and dep_id not in visited:
                    dependencies.append(self.elements[dep_id])
                    collect_deps(dep_id)
        
        collect_deps(element_id)
        return dependencies
    
    def get_usages(self, element_id: str, recursive: bool = False) -> List[CodeElement]:
        """Get elements that use this element."""
        if not recursive:
            usage_ids = self.usage_graph.get(element_id, set())
            return [self.elements[usage_id] for usage_id in usage_ids if usage_id in self.elements]
        
        # Recursive usages
        visited = set()
        usages = []
        
        def collect_usages(elem_id: str):
            if elem_id in visited:
                return
            visited.add(elem_id)
            
            for usage_id in self.usage_graph.get(elem_id, set()):
                if usage_id in self.elements and usage_id not in visited:
                    usages.append(self.elements[usage_id])
                    collect_usages(usage_id)
        
        collect_usages(element_id)
        return usages
    
    def search(self, query: str, element_types: Optional[List[NodeType]] = None, 
               languages: Optional[List[str]] = None) -> List[CodeElement]:
        """Search for elements by name or other criteria."""
        results = []
        
        for element in self.elements.values():
            # Filter by type
            if element_types and element.type not in element_types:
                continue
            
            # Filter by language
            if languages and element.language not in languages:
                continue
            
            # Search in name and docstring
            if (query.lower() in element.name.lower() or 
                (element.docstring and query.lower() in element.docstring.lower())):
                results.append(element)
        
        return results
    
    def get_file_tree(self) -> Dict[str, Any]:
        """Build hierarchical file tree structure."""
        if self.file_tree:
            return self.file_tree
        
        tree = {}
        
        for file_path in self.files.keys():
            parts = Path(file_path).parts
            current = tree
            
            for part in parts[:-1]:  # Directory parts
                if part not in current:
                    current[part] = {'type': 'directory', 'children': {}}
                current = current[part]['children']
            
            # File part
            filename = parts[-1]
            element_id = self.files[file_path]
            element = self.elements[element_id]
            
            current[filename] = {
                'type': 'file',
                'element_id': element_id,
                'language': element.language,
                'lines_of_code': element.lines_of_code,
                'complexity': element.complexity
            }
        
        self.file_tree = tree
        return tree
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            'project_id': self.project_id,
            'name': self.name,
            'root_path': self.root_path,
            'languages': list(self.languages),
            'elements': {k: v.to_dict() for k, v in self.elements.items()},
            'files': self.files,
            'packages': self.packages,
            'classes': self.classes,
            'functions': self.functions,
            'variables': self.variables,
            'by_type': {k.value: list(v) for k, v in self.by_type.items()},
            'by_language': self.by_language,
            'by_file': self.by_file,
            'dependency_graph': {k: list(v) for k, v in self.dependency_graph.items()},
            'usage_graph': {k: list(v) for k, v in self.usage_graph.items()},
            'file_tree': self.get_file_tree()
        }
    
    def get_metrics(self) -> Dict[str, Any]:
        """Get project-level metrics."""
        return {
            'total_files': len(self.files),
            'total_elements': len(self.elements),
            'languages': list(self.languages),
            'by_type': {k.value: len(v) for k, v in self.by_type.items()},
            'by_language': {k: len(v) for k, v in self.by_language.items()},
            'total_lines_of_code': sum(e.lines_of_code for e in self.elements.values()),
            'average_complexity': sum(e.complexity or 0 for e in self.elements.values()) / len(self.elements) if self.elements else 0,
            'dependency_density': len(self.dependency_graph) / len(self.elements) if self.elements else 0
        }
