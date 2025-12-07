"""
Visualization Adapters for Different Chart Types and Data Presentations
"""
from typing import Dict, List, Any, Optional, Union
from dataclasses import dataclass
from enum import Enum

from models.project_model import ProjectGraph, CodeElement, NodeType
from .navigation import ViewData, ViewLevel


class ChartType(Enum):
    """Types of visualizations available."""
    TREE = "tree"                    # Hierarchical tree
    NETWORK = "network"              # Network/graph visualization  
    SUNBURST = "sunburst"           # Hierarchical sunburst
    TREEMAP = "treemap"             # Space-filling treemap
    SANKEY = "sankey"               # Flow diagram
    HEATMAP = "heatmap"             # Grid-based heatmap
    BAR_CHART = "bar_chart"         # Bar/column chart
    PIE_CHART = "pie_chart"         # Pie/donut chart
    SCATTER = "scatter"             # Scatter plot
    TIMELINE = "timeline"           # Time-based visualization


@dataclass
class ChartData:
    """Data structure for chart visualization."""
    chart_type: ChartType
    title: str
    data: Dict[str, Any]           # Chart-specific data format
    config: Dict[str, Any]         # Chart configuration
    interactions: Dict[str, Any]   # Interaction callbacks
    metadata: Dict[str, Any]       # Additional metadata


class VisualizationAdapter:
    """Converts project data into various visualization formats."""
    
    def __init__(self, project_graph: ProjectGraph):
        self.graph = project_graph
    
    def get_project_overview_charts(self) -> List[ChartData]:
        """Get charts for project overview."""
        charts = []
        
        # 1. Language distribution pie chart
        charts.append(self._create_language_distribution_chart())
        
        # 2. File tree visualization
        charts.append(self._create_file_tree_chart())
        
        # 3. Complexity heatmap
        charts.append(self._create_complexity_heatmap())
        
        # 4. Dependency network
        charts.append(self._create_dependency_network())
        
        return charts
    
    def get_file_level_charts(self, file_id: Optional[str] = None) -> List[ChartData]:
        """Get charts for file-level analysis."""
        charts = []
        
        if file_id:
            # Single file analysis
            charts.append(self._create_file_structure_tree(file_id))
        else:
            # All files analysis
            charts.append(self._create_files_treemap())
            charts.append(self._create_files_metrics_chart())
            charts.append(self._create_files_language_breakdown())
        
        return charts
    
    def get_class_level_charts(self, class_filter: Optional[str] = None) -> List[ChartData]:
        """Get charts for class-level analysis."""
        charts = []
        
        if class_filter:
            # Single class analysis
            charts.append(self._create_class_hierarchy_tree(class_filter))
            charts.append(self._create_class_methods_chart(class_filter))
        else:
            # All classes analysis
            charts.append(self._create_class_inheritance_network())
            charts.append(self._create_class_size_distribution())
        
        return charts
    
    def get_dependency_charts(self, element_id: Optional[str] = None) -> List[ChartData]:
        """Get dependency visualization charts."""
        charts = []
        
        if element_id:
            charts.append(self._create_element_dependency_graph(element_id))
        else:
            charts.append(self._create_global_dependency_network())
        
        return charts
    
    # Chart creation methods
    
    def _create_language_distribution_chart(self) -> ChartData:
        """Create language distribution pie chart."""
        language_counts = {}
        for element in self.graph.elements.values():
            if element.type == NodeType.FILE:
                lang = element.language
                language_counts[lang] = language_counts.get(lang, 0) + 1
        
        data = {
            'labels': list(language_counts.keys()),
            'values': list(language_counts.values()),
            'colors': self._get_language_colors()
        }
        
        config = {
            'responsive': True,
            'plugins': {
                'legend': {'position': 'right'},
                'tooltip': {'enabled': True}
            }
        }
        
        return ChartData(
            chart_type=ChartType.PIE_CHART,
            title="Language Distribution",
            data=data,
            config=config,
            interactions={'click': 'filter_by_language'},
            metadata={'level': 'project', 'category': 'overview'}
        )
    
    def _create_file_tree_chart(self) -> ChartData:
        """Create hierarchical file tree."""
        tree_data = self.graph.get_file_tree()
        
        # Convert to hierarchical format for visualization
        vis_data = self._convert_tree_to_vis_format(tree_data)
        
        config = {
            'layout': 'tree',
            'orientation': 'vertical',
            'node_size': 'lines_of_code',
            'node_color': 'language',
            'expandable': True
        }
        
        return ChartData(
            chart_type=ChartType.TREE,
            title="Project File Tree",
            data={'nodes': vis_data},
            config=config,
            interactions={'click': 'navigate_to_file', 'expand': 'load_children'},
            metadata={'level': 'project', 'category': 'structure'}
        )
    
    def _create_complexity_heatmap(self) -> ChartData:
        """Create complexity heatmap by file."""
        files_data = []
        for file_path, file_id in self.graph.files.items():
            file_element = self.graph.get_element(file_id)
            if file_element:
                files_data.append({
                    'file': file_element.name,
                    'path': file_path,
                    'complexity': file_element.complexity or 0,
                    'lines': file_element.lines_of_code,
                    'language': file_element.language
                })
        
        # Group by directory for heatmap grid
        heatmap_data = self._create_heatmap_matrix(files_data)
        
        config = {
            'colorScale': 'RdYlBu_r',
            'showScale': True,
            'hovertemplate': '<b>%{text}</b><br>Complexity: %{z}<extra></extra>'
        }
        
        return ChartData(
            chart_type=ChartType.HEATMAP,
            title="Complexity Heatmap",
            data=heatmap_data,
            config=config,
            interactions={'click': 'navigate_to_file'},
            metadata={'level': 'project', 'category': 'metrics'}
        )
    
    def _create_dependency_network(self) -> ChartData:
        """Create dependency network graph."""
        nodes = []
        edges = []
        
        # Create nodes for files
        for element in self.graph.elements.values():
            if element.type == NodeType.FILE:
                nodes.append({
                    'id': element.id,
                    'label': element.name,
                    'size': element.lines_of_code,
                    'color': self._get_language_color(element.language),
                    'type': 'file'
                })
        
        # Create edges for dependencies
        for element_id, dependencies in self.graph.dependency_graph.items():
            for dep_id in dependencies:
                if element_id in [n['id'] for n in nodes] and dep_id in [n['id'] for n in nodes]:
                    edges.append({
                        'from': element_id,
                        'to': dep_id,
                        'arrows': 'to'
                    })
        
        config = {
            'physics': {'enabled': True, 'stabilization': True},
            'layout': {'improvedLayout': True},
            'interaction': {'hover': True, 'selectConnectedEdges': True}
        }
        
        return ChartData(
            chart_type=ChartType.NETWORK,
            title="File Dependencies",
            data={'nodes': nodes, 'edges': edges},
            config=config,
            interactions={'click': 'navigate_to_element', 'hover': 'show_details'},
            metadata={'level': 'project', 'category': 'dependencies'}
        )
    
    def _create_file_structure_tree(self, file_id: str) -> ChartData:
        """Create tree structure for a specific file."""
        file_element = self.graph.get_element(file_id)
        if not file_element:
            return self._create_empty_chart("File not found")
        
        children = self.graph.get_children(file_id)
        
        # Build hierarchical structure
        tree_nodes = [{
            'id': file_element.id,
            'name': file_element.name,
            'type': 'file',
            'children': []
        }]
        
        # Group children by type
        classes = [c for c in children if c.type == NodeType.CLASS]
        functions = [c for c in children if c.type in [NodeType.FUNCTION, NodeType.METHOD]]
        variables = [c for c in children if c.type == NodeType.VARIABLE]
        
        # Add class nodes with their methods
        for class_elem in classes:
            class_methods = self.graph.get_children(class_elem.id)
            class_node = {
                'id': class_elem.id,
                'name': class_elem.name,
                'type': 'class',
                'complexity': class_elem.complexity,
                'children': [
                    {
                        'id': method.id,
                        'name': method.name,
                        'type': 'method',
                        'complexity': method.complexity
                    }
                    for method in class_methods
                ]
            }
            tree_nodes[0]['children'].append(class_node)
        
        # Add standalone functions
        for func in functions:
            if not any(func.parent_id == cls.id for cls in classes):
                tree_nodes[0]['children'].append({
                    'id': func.id,
                    'name': func.name,
                    'type': 'function',
                    'complexity': func.complexity
                })
        
        config = {
            'layout': 'tree',
            'node_size_attr': 'complexity',
            'color_by': 'type',
            'expandable': True
        }
        
        return ChartData(
            chart_type=ChartType.TREE,
            title=f"Structure: {file_element.name}",
            data={'nodes': tree_nodes},
            config=config,
            interactions={'click': 'navigate_to_element'},
            metadata={'level': 'file', 'file_id': file_id}
        )
    
    def _create_files_treemap(self) -> ChartData:
        """Create treemap of all files sized by lines of code."""
        treemap_data = []
        
        for element in self.graph.elements.values():
            if element.type == NodeType.FILE:
                treemap_data.append({
                    'id': element.id,
                    'name': element.name,
                    'parent': element.parent_id or 'root',
                    'value': element.lines_of_code,
                    'complexity': element.complexity,
                    'language': element.language
                })
        
        config = {
            'colorscale': 'Viridis',
            'hovertemplate': '<b>%{label}</b><br>Lines: %{value}<br>Complexity: %{customdata}<extra></extra>',
            'textinfo': 'label+value'
        }
        
        return ChartData(
            chart_type=ChartType.TREEMAP,
            title="Files by Size",
            data={'data': treemap_data},
            config=config,
            interactions={'click': 'navigate_to_file'},
            metadata={'level': 'files', 'metric': 'lines_of_code'}
        )
    
    def _create_files_metrics_chart(self) -> ChartData:
        """Create scatter plot of files by complexity vs size."""
        scatter_data = []
        
        for element in self.graph.elements.values():
            if element.type == NodeType.FILE:
                scatter_data.append({
                    'x': element.lines_of_code,
                    'y': element.complexity or 0,
                    'text': element.name,
                    'marker': {
                        'size': max(5, element.lines_of_code // 50),
                        'color': self._get_language_color(element.language)
                    }
                })
        
        config = {
            'mode': 'markers',
            'hovertemplate': '<b>%{text}</b><br>Lines: %{x}<br>Complexity: %{y}<extra></extra>',
            'xaxis': {'title': 'Lines of Code'},
            'yaxis': {'title': 'Complexity'}
        }
        
        return ChartData(
            chart_type=ChartType.SCATTER,
            title="Files: Complexity vs Size",
            data={'data': scatter_data},
            config=config,
            interactions={'click': 'navigate_to_file'},
            metadata={'level': 'files', 'metric': 'complexity_vs_size'}
        )
    
    def _create_files_language_breakdown(self) -> ChartData:
        """Create language breakdown bar chart."""
        language_data = {}
        
        for element in self.graph.elements.values():
            if element.type == NodeType.FILE:
                lang = element.language
                if lang not in language_data:
                    language_data[lang] = {
                        'files': 0,
                        'lines': 0,
                        'complexity': 0
                    }
                language_data[lang]['files'] += 1
                language_data[lang]['lines'] += element.lines_of_code
                language_data[lang]['complexity'] += element.complexity or 0
        
        # Prepare bar chart data
        languages = list(language_data.keys())
        files_count = [language_data[lang]['files'] for lang in languages]
        lines_count = [language_data[lang]['lines'] for lang in languages]
        
        config = {
            'barmode': 'group',
            'xaxis': {'title': 'Programming Languages'},
            'yaxis': {'title': 'Count'}
        }
        
        data = {
            'traces': [
                {
                    'x': languages,
                    'y': files_count,
                    'name': 'Files',
                    'type': 'bar'
                },
                {
                    'x': languages,
                    'y': lines_count,
                    'name': 'Lines of Code',
                    'type': 'bar',
                    'yaxis': 'y2'
                }
            ]
        }
        
        return ChartData(
            chart_type=ChartType.BAR_CHART,
            title="Language Breakdown",
            data=data,
            config=config,
            interactions={'click': 'filter_by_language'},
            metadata={'level': 'files', 'metric': 'language_breakdown'}
        )
    
    def _create_global_dependency_network(self) -> ChartData:
        """Create comprehensive dependency network."""
        nodes = []
        edges = []
        
        # Include files, classes, and major functions
        for element in self.graph.elements.values():
            if element.type in [NodeType.FILE, NodeType.CLASS]:
                nodes.append({
                    'id': element.id,
                    'label': element.name,
                    'size': max(10, element.lines_of_code // 10),
                    'color': self._get_type_color(element.type),
                    'type': element.type.value,
                    'group': element.language
                })
        
        # Add dependency edges
        for element_id, dependencies in self.graph.dependency_graph.items():
            for dep_id in dependencies:
                if (element_id in [n['id'] for n in nodes] and 
                    dep_id in [n['id'] for n in nodes]):
                    edges.append({
                        'from': element_id,
                        'to': dep_id,
                        'arrows': 'to',
                        'width': 2
                    })
        
        config = {
            'physics': {'enabled': True, 'barnesHut': {'gravitationalConstant': -8000}},
            'groups': self._get_language_groups(),
            'interaction': {'hover': True, 'selectConnectedEdges': True}
        }
        
        return ChartData(
            chart_type=ChartType.NETWORK,
            title="Global Dependency Network",
            data={'nodes': nodes, 'edges': edges},
            config=config,
            interactions={'click': 'navigate_to_element', 'doubleClick': 'focus_on_element'},
            metadata={'level': 'project', 'category': 'dependencies'}
        )
    
    # Helper methods
    
    def _convert_tree_to_vis_format(self, tree_data: Dict[str, Any], parent_id: str = None) -> List[Dict[str, Any]]:
        """Convert tree data to visualization format."""
        nodes = []
        
        for name, item in tree_data.items():
            node_id = f"{parent_id}/{name}" if parent_id else name
            
            if item.get('type') == 'directory':
                nodes.append({
                    'id': node_id,
                    'name': name,
                    'type': 'directory',
                    'parent': parent_id,
                    'children': len(item.get('children', {}))
                })
                # Recursively add children
                nodes.extend(self._convert_tree_to_vis_format(
                    item.get('children', {}), 
                    node_id
                ))
            else:
                nodes.append({
                    'id': node_id,
                    'name': name,
                    'type': 'file',
                    'parent': parent_id,
                    'language': item.get('language'),
                    'lines_of_code': item.get('lines_of_code', 0),
                    'complexity': item.get('complexity', 0),
                    'element_id': item.get('element_id')
                })
        
        return nodes
    
    def _create_heatmap_matrix(self, files_data: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Create heatmap matrix from files data."""
        # Group files by directory
        dirs = {}
        for file_data in files_data:
            path_parts = file_data['path'].split('/')
            dir_name = '/'.join(path_parts[:-1]) if len(path_parts) > 1 else 'root'
            
            if dir_name not in dirs:
                dirs[dir_name] = []
            dirs[dir_name].append(file_data)
        
        # Create matrix
        z_data = []
        x_labels = []
        y_labels = list(dirs.keys())
        text_data = []
        
        max_files = max(len(files) for files in dirs.values())
        
        for dir_name in y_labels:
            row_z = []
            row_text = []
            files = dirs[dir_name]
            
            for i in range(max_files):
                if i < len(files):
                    file_data = files[i]
                    row_z.append(file_data['complexity'])
                    row_text.append(file_data['file'])
                    
                    if len(x_labels) <= i:
                        x_labels.append(f"File {i+1}")
                else:
                    row_z.append(0)
                    row_text.append("")
            
            z_data.append(row_z)
            text_data.append(row_text)
        
        return {
            'z': z_data,
            'x': x_labels,
            'y': y_labels,
            'text': text_data,
            'type': 'heatmap'
        }
    
    def _get_language_colors(self) -> Dict[str, str]:
        """Get color mapping for languages."""
        return {
            'python': '#3776ab',
            'javascript': '#f7df1e',
            'typescript': '#3178c6',
            'go': '#00add8',
            'rust': '#dea584',
            'java': '#ed8b00',
            'c': '#a8b9cc',
            'cpp': '#00599c',
            'css': '#1572b6',
            'html': '#e34f26'
        }
    
    def _get_language_color(self, language: str) -> str:
        """Get color for a specific language."""
        colors = self._get_language_colors()
        return colors.get(language, '#6c757d')  # Default gray
    
    def _get_type_color(self, node_type: NodeType) -> str:
        """Get color for node type."""
        type_colors = {
            NodeType.FILE: '#17a2b8',
            NodeType.CLASS: '#28a745',
            NodeType.FUNCTION: '#ffc107',
            NodeType.METHOD: '#fd7e14',
            NodeType.PACKAGE: '#6f42c1'
        }
        return type_colors.get(node_type, '#6c757d')
    
    def _get_language_groups(self) -> Dict[str, Dict[str, Any]]:
        """Get group configuration for languages."""
        colors = self._get_language_colors()
        groups = {}
        
        for lang, color in colors.items():
            groups[lang] = {
                'color': {'background': color, 'border': color},
                'font': {'color': 'white'}
            }
        
        return groups
    
    def _create_empty_chart(self, message: str) -> ChartData:
        """Create empty chart with message."""
        return ChartData(
            chart_type=ChartType.BAR_CHART,
            title="No Data",
            data={'message': message},
            config={},
            interactions={},
            metadata={'empty': True}
        )
