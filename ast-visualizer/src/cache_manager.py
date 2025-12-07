"""
Cache Manager for AST Visualizer
Provides Redis-based caching with in-memory fallback
"""
import json
import redis
from typing import Any, Optional, Dict
import pickle
import logging
from datetime import timedelta
import os

logger = logging.getLogger(__name__)


class CacheManager:
    """Manages caching for analysis results with Redis and memory fallback."""
    
    def __init__(self, redis_url: Optional[str] = None):
        """Initialize cache manager with Redis or memory fallback."""
        self.redis_url = redis_url or os.getenv('REDIS_URL', 'redis://localhost:6379')
        self.use_redis = False
        self.memory_cache: Dict[str, Any] = {}
        self._max_memory_items = 1000  # Prevent memory bloat
        # MEMORY OPTIMIZATION: Track memory usage and implement LRU-like eviction
        self._cache_access_order = []  # Track access order for LRU
        self._current_memory_mb = 0.0
        self._max_memory_mb = 100.0  # 100MB limit
        
        self._connect_redis()
    
    def _connect_redis(self) -> None:
        """Attempt to connect to Redis, fallback to memory cache."""
        try:
            self.redis_client = redis.from_url(
                self.redis_url,
                socket_connect_timeout=5,
                socket_timeout=5,
                decode_responses=False
            )
            # Test connection
            self.redis_client.ping()
            self.use_redis = True
            logger.info("Redis cache initialized successfully")
        except Exception as e:
            logger.warning(f"Redis unavailable, using memory cache: {e}")
            self.use_redis = False
            self.memory_cache = {}
    
    def set(self, key: str, value: Any, expire: int = 3600) -> bool:
        """
        Set value in cache with expiration.
        
        Args:
            key: Cache key
            value: Value to cache
            expire: Expiration time in seconds
            
        Returns:
            True if successful, False otherwise
        """
        if not key or not isinstance(key, str):
            logger.error(f"Invalid cache key: {key}")
            return False
            
        try:
            if self.use_redis:
                # Use JSON for security instead of pickle
                if isinstance(value, (dict, list, str, int, float, bool)):
                    serialized = json.dumps(value)
                    return bool(self.redis_client.setex(key, expire, serialized))
                else:
                    # For complex objects, use pickle with caution
                    serialized = pickle.dumps(value)
                    return bool(self.redis_client.setex(f"pickle:{key}", expire, serialized))
            else:
                # MEMORY OPTIMIZATION: Intelligent memory management
                import sys
                value_size_mb = sys.getsizeof(value) / (1024 * 1024)
                
                # Evict items if necessary
                self._evict_if_needed(value_size_mb)
                
                # Store new item
                self.memory_cache[key] = value
                self._current_memory_mb += value_size_mb
                
                # Update access order for LRU
                if key in self._cache_access_order:
                    self._cache_access_order.remove(key)
                self._cache_access_order.append(key)
                
                return True
                
        except Exception as e:
            logger.error(f"Cache set error for key '{key}': {e}")
            return False
    
    def get(self, key: str) -> Optional[Any]:
        """
        Get value from cache.
        
        Args:
            key: Cache key
            
        Returns:
            Cached value or None if not found/error
        """
        if not key or not isinstance(key, str):
            return None
            
        try:
            if self.use_redis:
                data = self.redis_client.get(key)
                if data:
                    try:
                        # Try JSON first
                        return json.loads(data)
                    except json.JSONDecodeError:
                        # Fallback to pickle if it's a pickle key
                        if key.startswith("pickle:"):
                            return pickle.loads(data)
                        
                # Try pickle key variant
                pickle_data = self.redis_client.get(f"pickle:{key}")
                if pickle_data:
                    return pickle.loads(pickle_data)
                    
            else:
                # MEMORY OPTIMIZATION: Update LRU order on access
                value = self.memory_cache.get(key)
                if value is not None and key in self._cache_access_order:
                    self._cache_access_order.remove(key)
                    self._cache_access_order.append(key)
                return value
                
        except Exception as e:
            logger.error(f"Cache get error for key '{key}': {e}")
            
        return None
    
    def delete(self, key: str) -> bool:
        """
        Delete value from cache.
        
        Args:
            key: Cache key
            
        Returns:
            True if deleted, False otherwise
        """
        if not key:
            return False
            
        try:
            if self.use_redis:
                deleted = self.redis_client.delete(key)
                # Also try to delete pickle variant
                pickle_deleted = self.redis_client.delete(f"pickle:{key}")
                return bool(deleted or pickle_deleted)
            else:
                if key in self.memory_cache:
                    # MEMORY OPTIMIZATION: Update memory tracking on deletion
                    import sys
                    value_size_mb = sys.getsizeof(self.memory_cache[key]) / (1024 * 1024)
                    del self.memory_cache[key]
                    self._current_memory_mb = max(0, self._current_memory_mb - value_size_mb)
                    
                    if key in self._cache_access_order:
                        self._cache_access_order.remove(key)
                    return True
                    
        except Exception as e:
            logger.error(f"Cache delete error for key '{key}': {e}")
            
        return False
    
    def clear_pattern(self, pattern: str) -> int:
        """
        Clear all keys matching pattern.
        
        Args:
            pattern: Pattern to match (Redis glob pattern)
            
        Returns:
            Number of keys deleted
        """
        count = 0
        try:
            if self.use_redis:
                for key in self.redis_client.scan_iter(match=pattern):
                    if self.redis_client.delete(key):
                        count += 1
                # Also clear pickle variants
                for key in self.redis_client.scan_iter(match=f"pickle:{pattern}"):
                    if self.redis_client.delete(key):
                        count += 1
            else:
                # Simple pattern matching for memory cache
                pattern_str = pattern.replace('*', '')
                keys_to_delete = [k for k in self.memory_cache if pattern_str in k]
                for key in keys_to_delete:
                    del self.memory_cache[key]
                    count += 1
                    
        except Exception as e:
            logger.error(f"Cache clear pattern error for '{pattern}': {e}")
            
        return count
    
    def health_check(self) -> Dict[str, Any]:
        """
        Check cache health and return status.
        
        Returns:
            Dictionary with cache status information
        """
        status = {
            'type': 'redis' if self.use_redis else 'memory',
            'healthy': False,
            'items_count': 0,
            'memory_usage_mb': 0
        }
        
        try:
            if self.use_redis:
                info = self.redis_client.info()
                status.update({
                    'healthy': True,
                    'items_count': info.get('db0', {}).get('keys', 0),
                    'memory_usage_mb': info.get('used_memory', 0) / (1024 * 1024)
                })
            else:
                import sys
                status.update({
                    'healthy': True,
                    'items_count': len(self.memory_cache),
                    'memory_usage_mb': self._current_memory_mb,
                    'memory_limit_mb': self._max_memory_mb
                })
                
        except Exception as e:
            logger.error(f"Cache health check error: {e}")
            
        return status
    
    def _evict_if_needed(self, new_item_size_mb: float) -> None:
        """MEMORY OPTIMIZATION: Evict items if memory limits would be exceeded."""
        target_size = self._current_memory_mb + new_item_size_mb
        
        # Evict if we would exceed memory limit or item count limit
        while (target_size > self._max_memory_mb or 
               len(self.memory_cache) >= self._max_memory_items) and self._cache_access_order:
            
            # Remove least recently used item
            oldest_key = self._cache_access_order.pop(0)
            if oldest_key in self.memory_cache:
                import sys
                old_size_mb = sys.getsizeof(self.memory_cache[oldest_key]) / (1024 * 1024)
                del self.memory_cache[oldest_key]
                self._current_memory_mb = max(0, self._current_memory_mb - old_size_mb)
                target_size = self._current_memory_mb + new_item_size_mb
    
    def close(self) -> None:
        """Close cache connections."""
        try:
            if self.use_redis and hasattr(self, 'redis_client'):
                self.redis_client.close()
            self.memory_cache.clear()
            self._cache_access_order.clear()
            self._current_memory_mb = 0.0
        except Exception as e:
            logger.error(f"Cache close error: {e}")
