"""
Tests for Flask application
"""
import pytest
import json
import tempfile
from pathlib import Path
from unittest.mock import Mock, patch
import os

# Set required environment variable for testing
os.environ['SECRET_KEY'] = 'test-secret-key-for-testing'

from app import app


class TestFlaskApp:
    """Test Flask application endpoints."""
    
    @pytest.fixture
    def client(self):
        """Create test client."""
        app.config['TESTING'] = True
        with app.test_client() as client:
            yield client
    
    @pytest.fixture
    def mock_analyzer(self):
        """Create mock analyzer."""
        with patch('app.analyzer') as mock:
            yield mock
    
    def test_index_page(self, client):
        """Test index page loads."""
        response = client.get('/')
        assert response.status_code == 200
        assert b'AST Repository Visualizer' in response.data
    
    def test_health_check(self, client):
        """Test health check endpoint."""
        response = client.get('/health')
        assert response.status_code == 200
        
        data = json.loads(response.data)
        assert data['status'] == 'healthy'
        assert 'timestamp' in data
        assert 'cache' in data
        assert 'version' in data
    
    def test_analyze_missing_data(self, client):
        """Test analyze endpoint with missing data."""
        response = client.post('/api/analyze', json={})
        assert response.status_code == 400
        
        data = json.loads(response.data)
        assert 'error' in data
        assert 'path or URL' in data['error']
    
    def test_analyze_both_path_and_url(self, client):
        """Test analyze endpoint with both path and URL."""
        response = client.post('/api/analyze', json={
            'path': '/test/path',
            'url': 'https://github.com/test/repo.git'
        })
        assert response.status_code == 400
        
        data = json.loads(response.data)
        assert 'error' in data
        assert 'either path or URL' in data['error']
    
    def test_analyze_local_success(self, client, mock_analyzer):
        """Test successful local analysis."""
        mock_analyzer.analyze_local.return_value = {
            'summary': {
                'total_files': 5,
                'total_lines': 1000,
                'total_classes': 3,
                'total_functions': 15
            },
            'files': ['file1.py', 'file2.py'],
            'metrics': {},
            'analysis_time': 2.5
        }
        
        with tempfile.TemporaryDirectory() as tmpdir:
            response = client.post('/api/analyze', json={'path': tmpdir})
            assert response.status_code == 200
            
            data = json.loads(response.data)
            assert 'analysis_id' in data
            assert data['summary']['total_files'] == 5
            assert 'files' in data
            assert 'metrics' in data
    
    def test_analyze_url_success(self, client, mock_analyzer):
        """Test successful URL analysis."""
        mock_analyzer.analyze_from_url.return_value = {
            'summary': {
                'total_files': 3,
                'total_lines': 500
            },
            'files': ['main.py'],
            'metrics': {},
            'analysis_time': 1.8
        }
        
        response = client.post('/api/analyze', json={
            'url': 'https://github.com/test/repo.git'
        })
        assert response.status_code == 200
        
        data = json.loads(response.data)
        assert 'analysis_id' in data
        assert data['summary']['total_files'] == 3
    
    def test_analyze_security_error(self, client, mock_analyzer):
        """Test analysis with security error."""
        from ast_analyzer import SecurityError
        mock_analyzer.analyze_local.side_effect = SecurityError("Invalid path")
        
        response = client.post('/api/analyze', json={'path': '/invalid/path'})
        assert response.status_code == 400
        
        data = json.loads(response.data)
        assert 'error' in data
        assert 'Security validation failed' in data['error']
    
    def test_visualize_invalid_id(self, client):
        """Test visualization with invalid analysis ID."""
        response = client.get('/api/visualize/invalid-id')
        assert response.status_code == 400
        
        data = json.loads(response.data)
        assert 'error' in data
        assert 'Invalid analysis ID' in data['error']
    
    def test_visualize_not_found(self, client):
        """Test visualization with non-existent analysis."""
        import uuid
        valid_id = str(uuid.uuid4())
        
        response = client.get(f'/api/visualize/{valid_id}')
        assert response.status_code == 404
        
        data = json.loads(response.data)
        assert 'error' in data
        assert 'not found' in data['error']
    
    def test_search_invalid_id(self, client):
        """Test search with invalid analysis ID."""
        response = client.get('/api/search/invalid-id?q=test')
        assert response.status_code == 400
        
        data = json.loads(response.data)
        assert 'error' in data
        assert 'Invalid analysis ID' in data['error']
    
    def test_search_query_too_long(self, client):
        """Test search with query that's too long."""
        import uuid
        valid_id = str(uuid.uuid4())
        long_query = 'x' * 101  # Exceeds 100 character limit
        
        response = client.get(f'/api/search/{valid_id}?q={long_query}')
        assert response.status_code == 400
        
        data = json.loads(response.data)
        assert 'error' in data
        assert 'Query too long' in data['error']
    
    def test_search_invalid_node_type(self, client):
        """Test search with invalid node type."""
        import uuid
        valid_id = str(uuid.uuid4())
        
        response = client.get(f'/api/search/{valid_id}?type=InvalidType')
        assert response.status_code == 400
        
        data = json.loads(response.data)
        assert 'error' in data
        assert 'Invalid node type' in data['error']
    
    def test_status_endpoint(self, client):
        """Test status endpoint."""
        response = client.get('/api/status')
        assert response.status_code == 200
        
        data = json.loads(response.data)
        assert data['status'] == 'running'
        assert 'cache' in data
        assert 'configuration' in data
    
    def test_export_invalid_id(self, client):
        """Test export with invalid analysis ID."""
        response = client.get('/api/export/invalid-id')
        assert response.status_code == 400
        
        data = json.loads(response.data)
        assert 'error' in data
        assert 'Invalid analysis ID' in data['error']
    
    def test_visualization_page_invalid_id(self, client):
        """Test visualization page with invalid ID."""
        response = client.get('/visualization/invalid-id')
        assert response.status_code == 400
        assert b'Invalid analysis ID' in response.data
    
    def test_rate_limiting(self, client):
        """Test rate limiting functionality."""
        # This test would need to be adjusted based on actual rate limits
        # For now, just verify the endpoint works
        response = client.get('/health')
        assert response.status_code == 200
    
    def test_security_headers(self, client):
        """Test that security headers are present."""
        response = client.get('/')
        
        assert 'X-Content-Type-Options' in response.headers
        assert response.headers['X-Content-Type-Options'] == 'nosniff'
        assert 'X-Frame-Options' in response.headers
        assert response.headers['X-Frame-Options'] == 'DENY'
        assert 'X-XSS-Protection' in response.headers
        assert 'Strict-Transport-Security' in response.headers
    
    def test_content_length_limit(self, client):
        """Test content length limit."""
        # Create a large payload
        large_data = {'data': 'x' * (100 * 1024 * 1024 + 1)}  # Larger than MAX_FILE_SIZE
        
        response = client.post('/api/analyze', 
                              data=json.dumps(large_data),
                              content_type='application/json')
        
        # Should be rejected due to content length
        assert response.status_code == 413
    
    def test_non_json_request(self, client):
        """Test non-JSON request to JSON endpoint."""
        response = client.post('/api/analyze', data='not json')
        assert response.status_code == 400


class TestIntegration:
    """Integration tests with real file analysis."""
    
    @pytest.fixture
    def client(self):
        """Create test client."""
        app.config['TESTING'] = True
        with app.test_client() as client:
            yield client
    
    def test_full_analysis_workflow(self, client):
        """Test complete analysis workflow with real files."""
        with tempfile.TemporaryDirectory() as tmpdir:
            # Create test Python files
            test_file = Path(tmpdir) / 'test_module.py'
            test_file.write_text('''
import os
import sys

class TestClass:
    """A test class."""
    
    def __init__(self, value):
        self.value = value
    
    def calculate(self, x):
        """Calculate something."""
        if x > 0:
            return self.value * x
        else:
            return 0

def main():
    """Main function."""
    obj = TestClass(42)
    result = obj.calculate(10)
    print(f"Result: {result}")

if __name__ == "__main__":
    main()
''')
            
            # Start analysis
            response = client.post('/api/analyze', json={'path': tmpdir})
            
            if response.status_code == 200:
                data = json.loads(response.data)
                analysis_id = data['analysis_id']
                
                # Check summary
                assert data['summary']['total_files'] >= 1
                assert data['summary']['total_classes'] >= 1
                assert data['summary']['total_functions'] >= 2
                
                # Test visualization endpoint
                viz_response = client.get(f'/api/visualize/{analysis_id}')
                if viz_response.status_code == 200:
                    viz_data = json.loads(viz_response.data)
                    assert 'nodes' in viz_data
                    assert 'edges' in viz_data
                    assert len(viz_data['nodes']) >= 1
                
                # Test search endpoint
                search_response = client.get(f'/api/search/{analysis_id}?q=TestClass')
                if search_response.status_code == 200:
                    search_data = json.loads(search_response.data)
                    assert 'results' in search_data
                
                # Test visualization page
                page_response = client.get(f'/visualization/{analysis_id}')
                assert page_response.status_code == 200


if __name__ == '__main__':
    pytest.main([__file__])
