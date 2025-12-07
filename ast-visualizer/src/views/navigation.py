"""
Navigation and Query Interface for Multi-Level Project Exploration
"""
from typing import Dict, List, Optional, Set, Any, Union
from dataclasses import dataclass
from enum import Enum

from models.project_model import ProjectGraph, CodeElement, NodeType


class ViewLevel(Enum):
    """Different levels of project visualization."""
    PROJECT = "project"         # Overall project overview
    PACKAGE = "package"         # Package/module level
    FILE = "file"              # File level
    CLASS = "class"            # Class level
    FUNCTION = "function"      # Function/method level
    DETAILS = "details"        # Individual element details


@dataclass
class NavigationContext:
    """Current navigation context and focus."""
    current_level: ViewLevel
    current_element_id: Optional[str] = None
    breadcrumb: List[str] = None  # Navigation path
    filters: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.breadcrumb is None:
            self.breadcrumb = []
        if self.filters is None:
            self.filters = {}


@dataclass 
class ViewData:
    """Data structure for visualization at a specific level."""
    level: ViewLevel
    title: str
    elements: List[Dict[str, Any]]
    metrics: Dict[str, Any]
    relationships: Dict[str, List[str]]  # element_id -> related_element_ids
    navigation_options: List[Dict[str, Any]]
    context: NavigationContext


class ProjectNavigator:
    """Provides navigation and querying capabilities for project graph."""
    
    def __init__(self, project_graph: ProjectGraph):
        self.graph = project_graph
        self.current_context = NavigationContext(current_level=ViewLevel.PROJECT)
    
    def get_project_overview(self) -> ViewData:
        """Get high-level project overview."""
        metrics = self.graph.get_metrics()
        
        # Top-level packages and standalone files
        top_level_elements = []
        for element in self.graph.elements.values():
            if element.parent_id is None and element.type in [NodeType.PACKAGE, NodeType.FILE]:
                top_level_elements.append(self._element_to_view_dict(element))
        
        navigation_options = [
            {'type': 'packages', 'label': f"Packages ({len(self.graph.packages)})", 'action': 'view_packages'},
            {'type': 'files', 'label': f"Files ({len(self.graph.files)})", 'action': 'view_files'},
            {'type': 'classes', 'label': f"Classes ({len(self.graph.classes)})", 'action': 'view_classes'},
            {'type': 'functions', 'label': f"Functions ({len(self.graph.functions)})", 'action': 'view_functions'},
            {'type': 'dependencies', 'label': "Dependencies", 'action': 'view_dependencies'}
        ]
        
        return ViewData(
            level=ViewLevel.PROJECT,
            title=f"Project: {self.graph.name}",
            elements=top_level_elements,
            metrics=metrics,
            relationships={},
            navigation_options=navigation_options,
            context=self.current_context
        )
    
    def get_package_view(self, package_id: Optional[str] = None) -> ViewData:
        """Get package-level view."""
        if package_id:
            package = self.graph.get_element(package_id)
            if not package:
                raise ValueError(f"Package {package_id} not found")
            
            # Get package contents
            children = self.graph.get_children(package_id)
            elements = [self._element_to_view_dict(child) for child in children]
            
            title = f"Package: {package.name}"
            metrics = self._get_element_metrics(package, children)
            
        else:
            # Show all packages
            package_elements = [self.graph.elements[pid] for pid in self.graph.packages.values()]
            elements = [self._element_to_view_dict(pkg) for pkg in package_elements]
            
            title = "All Packages"
            metrics = {
                'total_packages': len(package_elements),
                'by_language': self._group_by_language(package_elements)
            }
        
        navigation_options = [
            {'type': 'back', 'label': "← Back to Project", 'action': 'view_project'},
            {'type': 'files', 'label': "View Files", 'action': 'view_files'},
        ]
        
        return ViewData(
            level=ViewLevel.PACKAGE,
            title=title,
            elements=elements,
            metrics=metrics,
            relationships=self._get_package_relationships(package_id) if package_id else {},
            navigation_options=navigation_options,
            context=self.current_context
        )
    
    def get_file_view(self, file_id: Optional[str] = None) -> ViewData:
        """Get file-level view."""
        if file_id:
            file_element = self.graph.get_element(file_id)
            if not file_element:
                raise ValueError(f"File {file_id} not found")
            
            # Get file contents (classes, functions, variables)
            children = self.graph.get_children(file_id)
            elements = [self._element_to_view_dict(child) for child in children]
            
            title = f"File: {file_element.name}"
            metrics = self._get_element_metrics(file_element, children)
            relationships = self._get_file_relationships(file_id)
            
        else:
            # Show all files
            file_elements = [self.graph.elements[fid] for fid in self.graph.files.values()]
            elements = [self._element_to_view_dict(f) for f in file_elements]
            
            title = "All Files"
            metrics = {
                'total_files': len(file_elements),
                'by_language': self._group_by_language(file_elements),
                'total_lines': sum(f.lines_of_code for f in file_elements)
            }
            relationships = {}
        
        navigation_options = [
            {'type': 'back', 'label': "← Back to Project", 'action': 'view_project'},
            {'type': 'tree', 'label': "File Tree View", 'action': 'view_file_tree'},
            {'type': 'dependencies', 'label': "File Dependencies", 'action': 'view_file_dependencies'},
        ]
        
        return ViewData(
            level=ViewLevel.FILE,
            title=title,
            elements=elements,
            metrics=metrics,
            relationships=relationships,
            navigation_options=navigation_options,
            context=self.current_context
        )
    
    def get_class_view(self, class_filter: Optional[str] = None) -> ViewData:
        """Get class-level view."""
        if class_filter:
            # Show specific class and its members
            class_ids = self.graph.classes.get(class_filter, [])
            if not class_ids:
                raise ValueError(f"Class {class_filter} not found")
            
            # For multiple classes with same name, show the first one
            class_element = self.graph.get_element(class_ids[0])
            children = self.graph.get_children(class_ids[0])
            elements = [self._element_to_view_dict(child) for child in children]
            
            title = f"Class: {class_filter}"
            metrics = self._get_element_metrics(class_element, children)
            relationships = self._get_class_relationships(class_ids[0])
            
        else:
            # Show all classes
            all_classes = []
            for class_name, class_ids in self.graph.classes.items():
                for class_id in class_ids:
                    class_element = self.graph.get_element(class_id)
                    if class_element:
                        all_classes.append(class_element)
            
            elements = [self._element_to_view_dict(cls) for cls in all_classes]
            title = "All Classes"
            metrics = {
                'total_classes': len(all_classes),
                'by_language': self._group_by_language(all_classes),
                'inheritance_depth': self._calculate_inheritance_metrics(all_classes)
            }
            relationships = {}
        
        navigation_options = [
            {'type': 'back', 'label': "← Back to Project", 'action': 'view_project'},
            {'type': 'inheritance', 'label': "Class Hierarchy", 'action': 'view_inheritance'},
            {'type': 'methods', 'label': "All Methods", 'action': 'view_functions'},
        ]
        
        return ViewData(
            level=ViewLevel.CLASS,
            title=title,
            elements=elements,
            metrics=metrics,
            relationships=relationships,
            navigation_options=navigation_options,
            context=self.current_context
        )
    
    def get_function_view(self, function_filter: Optional[str] = None) -> ViewData:
        """Get function-level view."""
        if function_filter:
            # Show specific function details
            function_ids = self.graph.functions.get(function_filter, [])
            if not function_ids:
                raise ValueError(f"Function {function_filter} not found")
            
            function_elements = [self.graph.get_element(fid) for fid in function_ids if self.graph.get_element(fid)]
            elements = [self._element_to_view_dict(func) for func in function_elements]
            
            title = f"Function: {function_filter}"
            metrics = self._get_function_metrics(function_elements)
            relationships = self._get_function_relationships(function_ids[0])
            
        else:
            # Show all functions
            all_functions = []
            for function_name, function_ids in self.graph.functions.items():
                for function_id in function_ids:
                    function_element = self.graph.get_element(function_id)
                    if function_element:
                        all_functions.append(function_element)
            
            elements = [self._element_to_view_dict(func) for func in all_functions]
            title = "All Functions"
            metrics = {
                'total_functions': len(all_functions),
                'by_language': self._group_by_language(all_functions),
                'complexity_distribution': self._calculate_complexity_distribution(all_functions)
            }
            relationships = {}
        
        navigation_options = [
            {'type': 'back', 'label': "← Back to Project", 'action': 'view_project'},
            {'type': 'complexity', 'label': "Complexity Analysis", 'action': 'view_complexity'},
            {'type': 'call_graph', 'label': "Call Graph", 'action': 'view_call_graph'},
        ]
        
        return ViewData(
            level=ViewLevel.FUNCTION,
            title=title,
            elements=elements,
            metrics=metrics,
            relationships=relationships,
            navigation_options=navigation_options,
            context=self.current_context
        )
    
    def get_dependency_view(self, element_id: Optional[str] = None, 
                           dependency_type: str = 'imports') -> ViewData:
        """Get dependency analysis view."""
        if element_id:
            element = self.graph.get_element(element_id)
            if not element:
                raise ValueError(f"Element {element_id} not found")
            
            if dependency_type == 'imports':
                deps = self.graph.get_dependencies(element_id)
            elif dependency_type == 'usages':
                deps = self.graph.get_usages(element_id)
            else:
                deps = []
            
            elements = [self._element_to_view_dict(dep) for dep in deps]
            title = f"{dependency_type.title()} for {element.name}"
            metrics = {'count': len(deps), 'types': self._group_by_type(deps)}
            relationships = {element_id: [dep.id for dep in deps]}
            
        else:
            # Global dependency overview
            elements = []
            title = "Project Dependencies"
            metrics = {
                'total_dependencies': sum(len(deps) for deps in self.graph.dependency_graph.values()),
                'circular_dependencies': self._detect_circular_dependencies()
            }
            relationships = dict(self.graph.dependency_graph)
        
        navigation_options = [
            {'type': 'back', 'label': "← Back", 'action': 'go_back'},
            {'type': 'circular', 'label': "Circular Dependencies", 'action': 'view_circular_deps'},
            {'type': 'graph', 'label': "Dependency Graph", 'action': 'view_dep_graph'},
        ]
        
        return ViewData(
            level=ViewLevel.DETAILS,
            title=title,
            elements=elements,
            metrics=metrics,
            relationships=relationships,
            navigation_options=navigation_options,
            context=self.current_context
        )
    
    def search(self, query: str, element_types: Optional[List[NodeType]] = None,
               languages: Optional[List[str]] = None) -> ViewData:
        """Search across the project."""
        results = self.graph.search(query, element_types, languages)
        elements = [self._element_to_view_dict(result) for result in results]
        
        title = f"Search Results: '{query}'"
        metrics = {
            'total_results': len(results),
            'by_type': self._group_by_type(results),
            'by_language': self._group_by_language(results)
        }
        
        navigation_options = [
            {'type': 'back', 'label': "← Back", 'action': 'go_back'},
            {'type': 'refine', 'label': "Refine Search", 'action': 'refine_search'},
        ]
        
        return ViewData(
            level=ViewLevel.DETAILS,
            title=title,
            elements=elements,
            metrics=metrics,
            relationships={},
            navigation_options=navigation_options,
            context=self.current_context
        )
    
    def navigate_to(self, element_id: str) -> ViewData:
        """Navigate to a specific element."""
        element = self.graph.get_element(element_id)
        if not element:
            raise ValueError(f"Element {element_id} not found")
        
        # Update navigation context
        self.current_context.current_element_id = element_id
        self.current_context.breadcrumb.append(element_id)
        
        # Return appropriate view based on element type
        if element.type == NodeType.PROJECT:
            return self.get_project_overview()
        elif element.type == NodeType.PACKAGE:
            return self.get_package_view(element_id)
        elif element.type == NodeType.FILE:
            return self.get_file_view(element_id)
        elif element.type == NodeType.CLASS:
            return self.get_class_view(element.name)
        elif element.type in [NodeType.FUNCTION, NodeType.METHOD]:
            return self.get_function_view(element.name)
        else:
            # Default detail view
            return self._get_element_detail_view(element)
    
    def go_back(self) -> ViewData:
        """Navigate back in the breadcrumb."""
        if len(self.current_context.breadcrumb) > 1:
            self.current_context.breadcrumb.pop()
            previous_element_id = self.current_context.breadcrumb[-1]
            return self.navigate_to(previous_element_id)
        else:
            return self.get_project_overview()
    
    # Helper methods
    
    def _element_to_view_dict(self, element: CodeElement) -> Dict[str, Any]:
        """Convert element to view-friendly dictionary."""
        return {
            'id': element.id,
            'name': element.name,
            'type': element.type.value,
            'language': element.language,
            'location': element.location.to_dict(),
            'lines_of_code': element.lines_of_code,
            'complexity': element.complexity,
            'scope': element.scope.value if element.scope else None,
            'children_count': len(element.children_ids),
            'dependencies_count': len(element.depends_on),
            'properties': element.properties
        }
    
    def _get_element_metrics(self, element: CodeElement, children: List[CodeElement]) -> Dict[str, Any]:
        """Get metrics for an element and its children."""
        return {
            'lines_of_code': element.lines_of_code,
            'complexity': element.complexity,
            'children_count': len(children),
            'children_by_type': self._group_by_type(children),
            'dependencies_count': len(element.depends_on),
            'usages_count': len(element.used_by)
        }
    
    def _group_by_language(self, elements: List[CodeElement]) -> Dict[str, int]:
        """Group elements by programming language."""
        groups = {}
        for element in elements:
            lang = element.language
            groups[lang] = groups.get(lang, 0) + 1
        return groups
    
    def _group_by_type(self, elements: List[CodeElement]) -> Dict[str, int]:
        """Group elements by type."""
        groups = {}
        for element in elements:
            type_name = element.type.value
            groups[type_name] = groups.get(type_name, 0) + 1
        return groups
    
    def _get_package_relationships(self, package_id: str) -> Dict[str, List[str]]:
        """Get relationships for a package."""
        # Implementation would analyze package dependencies
        return {}
    
    def _get_file_relationships(self, file_id: str) -> Dict[str, List[str]]:
        """Get relationships for a file."""
        deps = self.graph.get_dependencies(file_id)
        usages = self.graph.get_usages(file_id)
        
        return {
            'depends_on': [dep.id for dep in deps],
            'used_by': [usage.id for usage in usages]
        }
    
    def _get_class_relationships(self, class_id: str) -> Dict[str, List[str]]:
        """Get relationships for a class."""
        class_element = self.graph.get_element(class_id)
        if not class_element:
            return {}
        
        relationships = {}
        if class_element.extends:
            relationships['extends'] = [class_element.extends]
        if class_element.implements:
            relationships['implements'] = list(class_element.implements)
        
        return relationships
    
    def _get_function_relationships(self, function_id: str) -> Dict[str, List[str]]:
        """Get relationships for a function."""
        deps = self.graph.get_dependencies(function_id)
        usages = self.graph.get_usages(function_id)
        
        return {
            'calls': [dep.id for dep in deps if dep.type in [NodeType.FUNCTION, NodeType.METHOD]],
            'called_by': [usage.id for usage in usages if usage.type in [NodeType.FUNCTION, NodeType.METHOD]]
        }
    
    def _get_function_metrics(self, functions: List[CodeElement]) -> Dict[str, Any]:
        """Get metrics for functions."""
        complexities = [f.complexity for f in functions if f.complexity]
        
        return {
            'count': len(functions),
            'avg_complexity': sum(complexities) / len(complexities) if complexities else 0,
            'max_complexity': max(complexities) if complexities else 0,
            'total_lines': sum(f.lines_of_code for f in functions)
        }
    
    def _calculate_complexity_distribution(self, functions: List[CodeElement]) -> Dict[str, int]:
        """Calculate complexity distribution."""
        distribution = {'low': 0, 'medium': 0, 'high': 0, 'very_high': 0}
        
        for func in functions:
            complexity = func.complexity or 0
            if complexity <= 5:
                distribution['low'] += 1
            elif complexity <= 10:
                distribution['medium'] += 1
            elif complexity <= 20:
                distribution['high'] += 1
            else:
                distribution['very_high'] += 1
        
        return distribution
    
    def _calculate_inheritance_metrics(self, classes: List[CodeElement]) -> Dict[str, Any]:
        """Calculate inheritance metrics."""
        # Simplified implementation
        return {
            'max_depth': 5,  # Would need actual calculation
            'avg_depth': 2.3,
            'total_hierarchies': 3
        }
    
    def _detect_circular_dependencies(self) -> List[List[str]]:
        """Detect circular dependencies."""
        # Simplified implementation - would need proper cycle detection
        return []
    
    def _get_element_detail_view(self, element: CodeElement) -> ViewData:
        """Get detailed view for any element."""
        return ViewData(
            level=ViewLevel.DETAILS,
            title=f"{element.type.value.title()}: {element.name}",
            elements=[self._element_to_view_dict(element)],
            metrics=self._get_element_metrics(element, []),
            relationships=self._get_file_relationships(element.id),
            navigation_options=[
                {'type': 'back', 'label': "← Back", 'action': 'go_back'},
                {'type': 'source', 'label': "View Source", 'action': 'view_source'},
            ],
            context=self.current_context
        )
