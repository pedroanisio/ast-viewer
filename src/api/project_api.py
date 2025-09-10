"""
Project API - RESTful endpoints for hierarchical project navigation
"""
from flask import Blueprint, request, jsonify
from typing import Dict, List, Optional, Any
import logging

from models.project_model import ProjectGraph, NodeType
from views.navigation import ProjectNavigator, ViewLevel
from views.visualizations import VisualizationAdapter, ChartType
from analyzers.project_builder import ProjectBuilder
from pathlib import Path

logger = logging.getLogger(__name__)

# Create Blueprint
project_bp = Blueprint('project', __name__, url_prefix='/api/v2')

# Import shared storage from main app (will be injected)
_project_graphs: Dict[str, ProjectGraph] = {}
_project_navigators: Dict[str, ProjectNavigator] = {}

def set_shared_storage(graphs: Dict[str, ProjectGraph], navigators: Dict[str, ProjectNavigator]):
    """Set shared storage references from main app."""
    global _project_graphs, _project_navigators
    _project_graphs = graphs
    _project_navigators = navigators


@project_bp.route('/project/<analysis_id>', methods=['GET'])
def get_project_overview(analysis_id: str):
    """Get project overview."""
    navigator = _get_navigator(analysis_id)
    if not navigator:
        return jsonify({'error': 'Project not found', 'available_analyses': list(_project_graphs.keys())}), 404
    
    try:
        view_data = navigator.get_project_overview()
        charts = _get_visualization_data(analysis_id, 'project')
        
        # Add breadcrumb for overview
        breadcrumb = ['Project']
        
        return jsonify({
            'status': 'success',
            'data': {
                'view': _view_data_to_dict(view_data),
                'charts': charts,
                'breadcrumb': breadcrumb
            }
        })
    except Exception as e:
        logger.error(f"Error getting project overview: {e}")
        return jsonify({'error': str(e)}), 500


@project_bp.route('/project/<analysis_id>/packages', methods=['GET'])
def get_packages_view(analysis_id: str):
    """Get packages view."""
    navigator = _get_navigator(analysis_id)
    if not navigator:
        return jsonify({'error': 'Project not found'}), 404
    
    package_id = request.args.get('package_id')
    
    try:
        view_data = navigator.get_package_view(package_id)
        return jsonify({
            'status': 'success',
            'data': {
                'view': _view_data_to_dict(view_data),
                'charts': _get_visualization_data(analysis_id, 'package', package_id)
            }
        })
    except Exception as e:
        logger.error(f"Error getting packages view: {e}")
        return jsonify({'error': str(e)}), 500


@project_bp.route('/project/<analysis_id>/files', methods=['GET'])
def get_files_view(analysis_id: str):
    """Get files view."""
    navigator = _get_navigator(analysis_id)
    if not navigator:
        return jsonify({'error': 'Project not found'}), 404
    
    file_id = request.args.get('file_id')
    
    try:
        view_data = navigator.get_files_view()  # Use get_files_view instead
        charts = _get_visualization_data(analysis_id, 'file', file_id)
        
        # Add breadcrumb
        breadcrumb = ['Project', 'Files']
        if file_id:
            breadcrumb.append(file_id)
        
        return jsonify({
            'status': 'success',
            'data': {
                'view': _view_data_to_dict(view_data),
                'charts': charts,
                'breadcrumb': breadcrumb
            }
        })
    except Exception as e:
        logger.error(f"Error getting files view: {e}")
        return jsonify({'error': str(e)}), 500


@project_bp.route('/project/<analysis_id>/classes', methods=['GET'])
def get_classes_view(analysis_id: str):
    """Get classes view."""
    navigator = _get_navigator(analysis_id)
    if not navigator:
        return jsonify({'error': 'Project not found'}), 404
    
    class_filter = request.args.get('class_name')
    
    try:
        view_data = navigator.get_classes_view()  # Use get_classes_view instead
        breadcrumb = ['Project', 'Classes']
        if class_filter:
            breadcrumb.append(class_filter)
            
        return jsonify({
            'status': 'success',
            'data': {
                'view': _view_data_to_dict(view_data),
                'charts': _get_visualization_data(analysis_id, 'class', class_filter),
                'breadcrumb': breadcrumb
            }
        })
    except Exception as e:
        logger.error(f"Error getting classes view: {e}")
        return jsonify({'error': str(e)}), 500


@project_bp.route('/project/<analysis_id>/functions', methods=['GET'])
def get_functions_view(analysis_id: str):
    """Get functions view."""
    navigator = _get_navigator(analysis_id)
    if not navigator:
        return jsonify({'error': 'Project not found'}), 404
    
    function_filter = request.args.get('function_name')
    
    try:
        view_data = navigator.get_functions_view()  # Use get_functions_view instead
        breadcrumb = ['Project', 'Functions']
        if function_filter:
            breadcrumb.append(function_filter)
            
        return jsonify({
            'status': 'success',
            'data': {
                'view': _view_data_to_dict(view_data),
                'charts': _get_visualization_data(analysis_id, 'function', function_filter),
                'breadcrumb': breadcrumb
            }
        })
    except Exception as e:
        logger.error(f"Error getting functions view: {e}")
        return jsonify({'error': str(e)}), 500


@project_bp.route('/project/<analysis_id>/dependencies', methods=['GET'])
def get_dependencies_view(analysis_id: str):
    """Get dependencies view."""
    navigator = _get_navigator(analysis_id)
    if not navigator:
        return jsonify({'error': 'Project not found'}), 404
    
    element_id = request.args.get('element_id')
    dependency_type = request.args.get('type', 'imports')
    
    try:
        view_data = navigator.get_dependency_view(element_id, dependency_type)
        return jsonify({
            'status': 'success',
            'data': {
                'view': _view_data_to_dict(view_data),
                'charts': _get_visualization_data(analysis_id, 'dependencies', element_id)
            }
        })
    except Exception as e:
        logger.error(f"Error getting dependencies view: {e}")
        return jsonify({'error': str(e)}), 500


@project_bp.route('/project/<analysis_id>/search', methods=['GET'])
def search_project(analysis_id: str):
    """Search across the project."""
    navigator = _get_navigator(analysis_id)
    if not navigator:
        return jsonify({'error': 'Project not found'}), 404
    
    query = request.args.get('q', '')
    if not query:
        return jsonify({'error': 'Query parameter required'}), 400
    
    # Parse optional filters
    element_types = request.args.getlist('types')
    languages = request.args.getlist('languages')
    
    # Convert string types to NodeType enums
    node_types = None
    if element_types:
        try:
            node_types = [NodeType(t) for t in element_types]
        except ValueError as e:
            return jsonify({'error': f'Invalid element type: {e}'}), 400
    
    try:
        view_data = navigator.search(query, node_types, languages)
        return jsonify({
            'status': 'success',
            'data': {
                'view': _view_data_to_dict(view_data),
                'query': query,
                'filters': {
                    'types': element_types,
                    'languages': languages
                }
            }
        })
    except Exception as e:
        logger.error(f"Error searching project: {e}")
        return jsonify({'error': str(e)}), 500


@project_bp.route('/project/<analysis_id>/navigate/<element_id>', methods=['GET'])
def navigate_to_element(analysis_id: str, element_id: str):
    """Navigate to a specific element."""
    navigator = _get_navigator(analysis_id)
    if not navigator:
        return jsonify({'error': 'Project not found'}), 404
    
    try:
        view_data = navigator.navigate_to(element_id)
        return jsonify({
            'status': 'success',
            'data': {
                'view': _view_data_to_dict(view_data),
                'breadcrumb': navigator.current_context.breadcrumb
            }
        })
    except Exception as e:
        logger.error(f"Error navigating to element: {e}")
        return jsonify({'error': str(e)}), 500


@project_bp.route('/project/<analysis_id>/charts/<chart_type>', methods=['GET'])
def get_chart_data(analysis_id: str, chart_type: str):
    """Get specific chart data."""
    graph = _get_project_graph(analysis_id)
    if not graph:
        return jsonify({'error': 'Project not found'}), 404
    
    try:
        chart_type_enum = ChartType(chart_type)
        adapter = VisualizationAdapter(graph)
        
        # Get appropriate charts based on type
        if chart_type == 'overview':
            charts = adapter.get_project_overview_charts()
        elif chart_type == 'files':
            charts = adapter.get_file_level_charts()
        elif chart_type == 'dependencies':
            charts = adapter.get_dependency_charts()
        else:
            return jsonify({'error': f'Unknown chart type: {chart_type}'}), 400
        
        return jsonify({
            'status': 'success',
            'data': [_chart_data_to_dict(chart) for chart in charts]
        })
    except Exception as e:
        logger.error(f"Error getting chart data: {e}")
        return jsonify({'error': str(e)}), 500


@project_bp.route('/project/<analysis_id>/metrics', methods=['GET'])
def get_project_metrics(analysis_id: str):
    """Get comprehensive project metrics."""
    graph = _get_project_graph(analysis_id)
    if not graph:
        return jsonify({'error': 'Project not found'}), 404
    
    try:
        metrics = graph.get_metrics()
        
        # Add additional computed metrics
        additional_metrics = {
            'file_tree': graph.get_file_tree(),
            'complexity_stats': _calculate_complexity_stats(graph),
            'dependency_stats': _calculate_dependency_stats(graph),
            'language_breakdown': _calculate_language_breakdown(graph)
        }
        
        return jsonify({
            'status': 'success',
            'data': {
                'basic_metrics': metrics,
                'detailed_metrics': additional_metrics
            }
        })
    except Exception as e:
        logger.error(f"Error getting project metrics: {e}")
        return jsonify({'error': str(e)}), 500


# Integration with existing analyzer
@project_bp.route('/project/build/<analysis_id>', methods=['POST'])
def build_project_graph(analysis_id: str):
    """Build hierarchical project graph from existing analysis."""
    try:
        # Get analysis details from request
        data = request.get_json()
        project_name = data.get('project_name', 'Unknown Project')
        root_path = data.get('root_path', '.')
        
        # Build project graph
        builder = ProjectBuilder()
        project_graph = builder.build_project(
            project_name=project_name,
            root_path=Path(root_path),
            analysis_id=analysis_id
        )
        
        # Store for future access
        _project_graphs[analysis_id] = project_graph
        _project_navigators[analysis_id] = ProjectNavigator(project_graph)
        
        return jsonify({
            'status': 'success',
            'data': {
                'analysis_id': analysis_id,
                'project_name': project_name,
                'metrics': project_graph.get_metrics(),
                'build_info': {
                    'total_elements': len(project_graph.elements),
                    'languages': list(project_graph.languages),
                    'file_count': len(project_graph.files)
                }
            }
        })
    except Exception as e:
        logger.error(f"Error building project graph: {e}")
        return jsonify({'error': str(e)}), 500


# Helper functions

def _get_project_graph(analysis_id: str) -> Optional[ProjectGraph]:
    """Get project graph by analysis ID."""
    return _project_graphs.get(analysis_id)


def _get_navigator(analysis_id: str) -> Optional[ProjectNavigator]:
    """Get project navigator by analysis ID."""
    return _project_navigators.get(analysis_id)


def _view_data_to_dict(view_data) -> Dict[str, Any]:
    """Convert ViewData to dictionary."""
    return {
        'level': view_data.level.value,
        'title': view_data.title,
        'elements': view_data.elements,
        'metrics': view_data.metrics,
        'relationships': view_data.relationships,
        'navigation_options': view_data.navigation_options,
        'context': {
            'current_level': view_data.context.current_level.value,
            'current_element_id': view_data.context.current_element_id,
            'breadcrumb': view_data.context.breadcrumb,
            'filters': view_data.context.filters
        }
    }


def _chart_data_to_dict(chart_data) -> Dict[str, Any]:
    """Convert ChartData to dictionary."""
    return {
        'chart_type': chart_data.chart_type.value,
        'title': chart_data.title,
        'data': chart_data.data,
        'config': chart_data.config,
        'interactions': chart_data.interactions,
        'metadata': chart_data.metadata
    }


def _get_visualization_data(analysis_id: str, view_type: str, 
                          element_filter: Optional[str] = None) -> List[Dict[str, Any]]:
    """Get visualization data for a specific view."""
    graph = _get_project_graph(analysis_id)
    if not graph:
        return []
    
    adapter = VisualizationAdapter(graph)
    
    try:
        if view_type == 'project':
            charts = adapter.get_project_overview_charts()
        elif view_type == 'file':
            charts = adapter.get_file_level_charts(element_filter)
        elif view_type == 'class':
            charts = adapter.get_class_level_charts(element_filter)
        elif view_type == 'function':
            # For functions, use file-level charts for now
            charts = adapter.get_file_level_charts()
        elif view_type == 'dependencies':
            charts = adapter.get_dependency_charts(element_filter)
        else:
            charts = []
        
        return [_chart_data_to_dict(chart) for chart in charts]
    except Exception as e:
        logger.warning(f"Error getting visualization data: {e}")
        return []


def _calculate_complexity_stats(graph: ProjectGraph) -> Dict[str, Any]:
    """Calculate complexity statistics."""
    complexities = []
    for element in graph.elements.values():
        if element.complexity and element.complexity > 0:
            complexities.append(element.complexity)
    
    if not complexities:
        return {'average': 0, 'max': 0, 'min': 0, 'distribution': {}}
    
    return {
        'average': sum(complexities) / len(complexities),
        'max': max(complexities),
        'min': min(complexities),
        'total_elements': len(complexities),
        'distribution': {
            'low': len([c for c in complexities if c <= 5]),
            'medium': len([c for c in complexities if 5 < c <= 10]),
            'high': len([c for c in complexities if 10 < c <= 20]),
            'very_high': len([c for c in complexities if c > 20])
        }
    }


def _calculate_dependency_stats(graph: ProjectGraph) -> Dict[str, Any]:
    """Calculate dependency statistics."""
    total_deps = sum(len(deps) for deps in graph.dependency_graph.values())
    elements_with_deps = len([deps for deps in graph.dependency_graph.values() if deps])
    
    return {
        'total_dependencies': total_deps,
        'elements_with_dependencies': elements_with_deps,
        'average_dependencies_per_element': total_deps / len(graph.elements) if graph.elements else 0,
        'dependency_density': elements_with_deps / len(graph.elements) if graph.elements else 0
    }


def _calculate_language_breakdown(graph: ProjectGraph) -> Dict[str, Any]:
    """Calculate detailed language breakdown."""
    breakdown = {}
    
    for language in graph.languages:
        elements = [e for e in graph.elements.values() if e.language == language]
        files = [e for e in elements if e.type == NodeType.FILE]
        
        breakdown[language] = {
            'total_elements': len(elements),
            'files': len(files),
            'lines_of_code': sum(e.lines_of_code for e in files),
            'average_complexity': sum(e.complexity or 0 for e in elements) / len(elements) if elements else 0,
            'types': {
                node_type.value: len([e for e in elements if e.type == node_type])
                for node_type in NodeType
            }
        }
    
    return breakdown
