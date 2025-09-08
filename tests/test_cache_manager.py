"""
Tests for cache_manager.py
"""
import pytest
import json
import tempfile
import os
from unittest.mock import Mock, patch

from cache_manager import CacheManager


class TestCacheManager:
    """Test cache manager functionality."""
    
    def test_memory_cache_initialization(self):
        """Test memory cache initialization when Redis is unavailable."""
        with patch('redis.from_url') as mock_redis:
            mock_redis.side_effect = Exception("Redis unavailable")
            
            cache = CacheManager()
            
            assert not cache.use_redis
            assert cache.memory_cache == {}
    
    def test_memory_cache_set_get(self):
        """Test setting and getting values from memory cache."""
        with patch('redis.from_url') as mock_redis:
            mock_redis.side_effect = Exception("Redis unavailable")
            
            cache = CacheManager()
            
            # Test setting and getting
            assert cache.set('test_key', 'test_value')
            assert cache.get('test_key') == 'test_value'
            
            # Test non-existent key
            assert cache.get('non_existent') is None
    
    def test_memory_cache_delete(self):
        """Test deleting values from memory cache."""
        with patch('redis.from_url') as mock_redis:
            mock_redis.side_effect = Exception("Redis unavailable")
            
            cache = CacheManager()
            
            # Set and delete
            cache.set('test_key', 'test_value')
            assert cache.delete('test_key')
            assert cache.get('test_key') is None
            
            # Delete non-existent key
            assert not cache.delete('non_existent')
    
    def test_memory_cache_clear_pattern(self):
        """Test clearing keys by pattern in memory cache."""
        with patch('redis.from_url') as mock_redis:
            mock_redis.side_effect = Exception("Redis unavailable")
            
            cache = CacheManager()
            
            # Set multiple keys
            cache.set('analysis:1', 'data1')
            cache.set('analysis:2', 'data2')
            cache.set('file:1', 'file_data')
            
            # Clear analysis keys
            count = cache.clear_pattern('analysis:*')
            assert count == 2
            
            # Check remaining keys
            assert cache.get('analysis:1') is None
            assert cache.get('analysis:2') is None
            assert cache.get('file:1') == 'file_data'
    
    def test_memory_cache_size_limit(self):
        """Test memory cache size limit enforcement."""
        with patch('redis.from_url') as mock_redis:
            mock_redis.side_effect = Exception("Redis unavailable")
            
            cache = CacheManager()
            cache._max_memory_items = 3  # Set small limit for testing
            
            # Add items up to limit
            for i in range(3):
                cache.set(f'key_{i}', f'value_{i}')
            
            # Add one more - should remove oldest
            cache.set('key_3', 'value_3')
            
            # First key should be removed
            assert cache.get('key_0') is None
            assert cache.get('key_3') == 'value_3'
    
    def test_health_check_memory(self):
        """Test health check for memory cache."""
        with patch('redis.from_url') as mock_redis:
            mock_redis.side_effect = Exception("Redis unavailable")
            
            cache = CacheManager()
            cache.set('test', 'data')
            
            status = cache.health_check()
            
            assert status['type'] == 'memory'
            assert status['healthy'] is True
            assert status['items_count'] == 1
            assert 'memory_usage_mb' in status
    
    def test_invalid_key_handling(self):
        """Test handling of invalid keys."""
        with patch('redis.from_url') as mock_redis:
            mock_redis.side_effect = Exception("Redis unavailable")
            
            cache = CacheManager()
            
            # Test invalid keys
            assert not cache.set('', 'value')
            assert not cache.set(None, 'value')
            assert cache.get('') is None
            assert cache.get(None) is None
    
    def test_json_serialization_preference(self):
        """Test that JSON is preferred over pickle for simple data."""
        mock_redis_client = Mock()
        
        with patch('redis.from_url') as mock_redis:
            mock_redis.return_value = mock_redis_client
            mock_redis_client.ping.return_value = True
            mock_redis_client.setex.return_value = True
            
            cache = CacheManager()
            
            # Test with JSON-serializable data
            test_data = {'key': 'value', 'number': 42}
            cache.set('test_json', test_data)
            
            # Should call setex with JSON string, not pickle
            mock_redis_client.setex.assert_called_once()
            args = mock_redis_client.setex.call_args[0]
            assert args[0] == 'test_json'  # key
            assert isinstance(args[2], str)  # JSON string
            
            # Verify it's valid JSON
            json.loads(args[2])
    
    @patch('redis.from_url')
    def test_redis_connection_success(self, mock_redis):
        """Test successful Redis connection."""
        mock_redis_client = Mock()
        mock_redis.return_value = mock_redis_client
        mock_redis_client.ping.return_value = True
        
        cache = CacheManager()
        
        assert cache.use_redis is True
        mock_redis_client.ping.assert_called_once()
    
    def test_close_method(self):
        """Test cache close method."""
        with patch('redis.from_url') as mock_redis:
            mock_redis.side_effect = Exception("Redis unavailable")
            
            cache = CacheManager()
            cache.set('test', 'data')
            
            # Should not raise exception
            cache.close()
            
            # Memory cache should be cleared
            assert len(cache.memory_cache) == 0


if __name__ == '__main__':
    pytest.main([__file__])
