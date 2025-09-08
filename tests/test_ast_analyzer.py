"""
Tests for ast_analyzer.py
"""
import pytest
import tempfile
import os
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock

from ast_analyzer import RepositoryAnalyzer, SecurityError, ASTNode, FileAnalysis


class TestRepositoryAnalyzer:
    """Test repository analyzer functionality."""
    
    def setup_method(self):
        """Set up test environment."""
        self.cache_mock = Mock()
        self.analyzer = RepositoryAnalyzer(self.cache_mock)
    
    def test_initialization(self):
        """Test analyzer initialization."""
        analyzer = RepositoryAnalyzer()
        
        assert analyzer.cache is None
        assert analyzer.node_registry == {}
        assert analyzer.file_registry == {}
        assert analyzer.max_workers >= 1
    
    def test_path_validation_valid_path(self):
        """Test path validation with valid path."""
        with tempfile.TemporaryDirectory() as tmpdir:
            valid_path = Path(tmpdir)
            
            # Should not raise exception
            self.analyzer._validate_path(valid_path)
    
    def test_path_validation_nonexistent_path(self):
        """Test path validation with non-existent path."""
        invalid_path = Path('/non/existent/path')
        
        with pytest.raises(SecurityError):
            self.analyzer._validate_path(invalid_path)
    
    def test_url_validation_valid_urls(self):
        """Test URL validation with valid URLs."""
        valid_urls = [
            'https://github.com/user/repo.git',
            'http://gitlab.com/user/repo.git',
            'git://example.com/repo.git'
        ]
        
        for url in valid_urls:
            # Should not raise exception
            self.analyzer._validate_url(url)
    
    def test_url_validation_invalid_urls(self):
        """Test URL validation with invalid URLs."""
        invalid_urls = [
            'ftp://example.com/repo.git',
            'file:///local/path',
            'javascript:alert(1)',
            'http://localhost/repo.git',
            'https://127.0.0.1/repo.git'
        ]
        
        for url in invalid_urls:
            with pytest.raises(SecurityError):
                self.analyzer._validate_url(url)
    
    def test_find_python_files(self):
        """Test finding Python files in directory."""
        with tempfile.TemporaryDirectory() as tmpdir:
            tmpdir_path = Path(tmpdir)
            
            # Create test files
            (tmpdir_path / 'test.py').write_text('print("hello")')
            (tmpdir_path / 'test.txt').write_text('not python')
            (tmpdir_path / '__pycache__').mkdir()
            (tmpdir_path / '__pycache__' / 'test.pyc').write_text('bytecode')
            (tmpdir_path / 'subdir').mkdir()
            (tmpdir_path / 'subdir' / 'module.py').write_text('def func(): pass')
            
            files = self.analyzer._find_python_files(tmpdir_path)
            
            # Should find .py files but not .txt or .pyc
            py_files = [f.name for f in files]
            assert 'test.py' in py_files
            assert 'module.py' in py_files
            assert 'test.txt' not in py_files
            assert 'test.pyc' not in py_files
    
    def test_file_size_limits(self):
        """Test file size limits are enforced."""
        with tempfile.TemporaryDirectory() as tmpdir:
            tmpdir_path = Path(tmpdir)
            
            # Create large file
            large_file = tmpdir_path / 'large.py'
            large_content = 'print("x")' * 100000  # Large file
            large_file.write_text(large_content)
            
            # Temporarily reduce size limit
            original_limit = self.analyzer.MAX_FILE_SIZE
            self.analyzer.MAX_FILE_SIZE = 1024  # 1KB limit
            
            try:
                files = self.analyzer._find_python_files(tmpdir_path)
                # Large file should be skipped
                assert len(files) == 0
            finally:
                self.analyzer.MAX_FILE_SIZE = original_limit
    
    def test_read_file_safely_utf8(self):
        """Test safe file reading with UTF-8 encoding."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False, encoding='utf-8') as f:
            f.write('# -*- coding: utf-8 -*-\nprint("Hello, 世界")')
            f.flush()
            
            try:
                content, encoding = self.analyzer._read_file_safely(Path(f.name))
                
                assert content is not None
                assert '世界' in content
                assert encoding == 'utf-8'
            finally:
                os.unlink(f.name)
    
    def test_read_file_safely_latin1(self):
        """Test safe file reading with Latin-1 encoding."""
        with tempfile.NamedTemporaryFile(mode='wb', suffix='.py', delete=False) as f:
            # Write Latin-1 encoded content
            content = 'print("café")'.encode('latin-1')
            f.write(content)
            f.flush()
            
            try:
                result_content, encoding = self.analyzer._read_file_safely(Path(f.name))
                
                assert result_content is not None
                assert 'café' in result_content
                assert encoding in ['utf-8', 'latin-1']  # Could be detected as either
            finally:
                os.unlink(f.name)
    
    def test_analyze_file_valid_python(self):
        """Test analyzing a valid Python file."""
        python_code = '''
import os
import sys

class TestClass:
    def __init__(self):
        self.value = 42
    
    def method(self):
        if True:
            return self.value
        else:
            return 0

def test_function():
    obj = TestClass()
    return obj.method()
'''
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as f:
            f.write(python_code)
            f.flush()
            
            try:
                result = self.analyzer._analyze_file(Path(f.name), 'test_analysis')
                
                assert result is not None
                assert isinstance(result, FileAnalysis)
                assert result.path == f.name
                assert len(result.nodes) > 0
                assert 'os' in result.imports
                assert 'sys' in result.imports
                assert 'TestClass' in result.classes
                assert 'test_function' in result.functions
                assert result.complexity > 0
                assert result.lines > 0
                
            finally:
                os.unlink(f.name)
    
    def test_analyze_file_syntax_error(self):
        """Test analyzing file with syntax error."""
        invalid_code = '''
def broken_function(
    print("missing closing parenthesis"
'''
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as f:
            f.write(invalid_code)
            f.flush()
            
            try:
                result = self.analyzer._analyze_file(Path(f.name), 'test_analysis')
                
                # Should return None for files with syntax errors
                assert result is None
                
            finally:
                os.unlink(f.name)
    
    def test_complexity_calculation(self):
        """Test complexity calculation."""
        simple_code = 'print("hello")'
        complex_code = '''
def complex_function(x):
    if x > 0:
        for i in range(x):
            if i % 2 == 0:
                try:
                    result = i / 2
                except ZeroDivisionError:
                    result = 0
            else:
                result = i
        return result
    else:
        return 0
'''
        
        simple_complexity = self.analyzer._calculate_complexity(simple_code)
        complex_complexity = self.analyzer._calculate_complexity(complex_code)
        
        assert simple_complexity >= 0
        assert complex_complexity > simple_complexity
    
    def test_node_processing(self):
        """Test AST node processing."""
        import ast
        
        code = 'def test_func(x): return x + 1'
        tree = ast.parse(code)
        
        # Find function definition node
        func_node = None
        for node in ast.walk(tree):
            if isinstance(node, ast.FunctionDef):
                func_node = node
                break
        
        assert func_node is not None
        
        ast_node = self.analyzer._process_node(func_node, 'test_file.py')
        
        assert ast_node is not None
        assert isinstance(ast_node, ASTNode)
        assert ast_node.type == 'FunctionDef'
        assert ast_node.name == 'test_func'
        assert ast_node.file == 'test_file.py'
        assert ast_node.line == func_node.lineno
    
    def test_generate_summary(self):
        """Test summary generation."""
        # Create mock file analyses
        file1 = FileAnalysis(
            path='file1.py',
            nodes=[],
            imports=['os', 'sys'],
            classes=['Class1'],
            functions=['func1', 'func2'],
            complexity=2.5,
            lines=100,
            hash='hash1',
            size_bytes=1024
        )
        
        file2 = FileAnalysis(
            path='file2.py',
            nodes=[],
            imports=['json', 'os'],
            classes=['Class2', 'Class3'],
            functions=['func3'],
            complexity=1.5,
            lines=50,
            hash='hash2',
            size_bytes=512
        )
        
        results = [file1, file2]
        summary = self.analyzer._generate_summary(results)
        
        assert summary['total_files'] == 2
        assert summary['total_lines'] == 150
        assert summary['total_classes'] == 3
        assert summary['total_functions'] == 3
        assert summary['average_complexity'] == 2.0
        assert 'os' in summary['imports']
        assert 'sys' in summary['imports']
        assert 'json' in summary['imports']
    
    def test_prepare_visualization(self):
        """Test visualization data preparation."""
        analysis_data = {
            'files': [
                {
                    'path': 'test1.py',
                    'lines': 100,
                    'complexity': 2.5,
                    'classes': ['Class1'],
                    'functions': ['func1'],
                    'size_bytes': 1024
                },
                {
                    'path': 'test2.py',
                    'lines': 50,
                    'complexity': 1.5,
                    'classes': [],
                    'functions': ['func2'],
                    'size_bytes': 512
                }
            ],
            'summary': {'total_files': 2},
            'metrics': {
                'import_graph': {
                    'test1': ['test2'],
                    'test2': []
                }
            }
        }
        
        viz_data = self.analyzer.prepare_visualization(analysis_data)
        
        assert 'nodes' in viz_data
        assert 'edges' in viz_data
        assert 'summary' in viz_data
        assert len(viz_data['nodes']) == 2
        assert len(viz_data['edges']) == 1
        
        # Check node structure
        node = viz_data['nodes'][0]
        assert 'id' in node
        assert 'label' in node
        assert 'type' in node
        assert 'metrics' in node
    
    def test_search_nodes(self):
        """Test node searching functionality."""
        # Mock cache with analysis data
        analysis_data = {
            'nodes': {
                'node1': {
                    'name': 'test_function',
                    'type': 'FunctionDef',
                    'file': 'test.py',
                    'properties': {}
                },
                'node2': {
                    'name': 'TestClass',
                    'type': 'ClassDef',
                    'file': 'test.py',
                    'properties': {}
                },
                'node3': {
                    'name': 'other_function',
                    'type': 'FunctionDef',
                    'file': 'other.py',
                    'properties': {}
                }
            }
        }
        
        self.cache_mock.get.return_value = analysis_data
        
        # Test search by name
        results = self.analyzer.search_nodes('test_id', 'test', '')
        assert len(results) == 2  # test_function and TestClass
        
        # Test search by type
        results = self.analyzer.search_nodes('test_id', '', 'FunctionDef')
        assert len(results) == 2  # Both functions
        
        # Test combined search
        results = self.analyzer.search_nodes('test_id', 'test', 'FunctionDef')
        assert len(results) == 1  # Only test_function
    
    @patch('git.Repo.clone_from')
    def test_analyze_from_url(self, mock_clone):
        """Test analyzing repository from URL."""
        # Mock git clone
        mock_repo = Mock()
        mock_clone.return_value = mock_repo
        
        with tempfile.TemporaryDirectory() as tmpdir:
            # Create a mock cloned repository
            repo_path = Path(tmpdir) / 'repo'
            repo_path.mkdir()
            (repo_path / 'test.py').write_text('print("hello")')
            
            # Mock the clone to use our test directory
            def mock_clone_side_effect(url, path, **kwargs):
                # Copy our test file to the clone path
                (Path(path) / 'test.py').write_text('print("hello")')
                return mock_repo
            
            mock_clone.side_effect = mock_clone_side_effect
            
            with patch.object(self.analyzer, 'analyze_local') as mock_analyze_local:
                mock_analyze_local.return_value = {'summary': {}, 'files': [], 'metrics': {}}
                
                result = self.analyzer.analyze_from_url(
                    'https://github.com/test/repo.git', 
                    'test_analysis'
                )
                
                assert result is not None
                mock_clone.assert_called_once()
                mock_analyze_local.assert_called_once()


if __name__ == '__main__':
    pytest.main([__file__])
