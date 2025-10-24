// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! Caching layer for dataized vertex values.
//!
//! This module implements an efficient caching mechanism for storing
//! dataized vertex values. The cache uses a Vec-based structure optimized
//! for sequential vertex IDs, providing O(1) access time and minimal memory
//! overhead compared to HashMap-based approaches.
//!
//! The cache automatically expands as needed when higher vertex IDs are
//! accessed, using None values to represent uncached vertices.

use crate::data::Data;
use crate::universe::VertexId;

/// Efficient cache for storing dataized vertex values.
///
/// The cache is optimized for sequential vertex IDs starting from 0.
/// It uses a Vec of Option<Data> where the index corresponds to the
/// vertex ID, providing O(1) lookup and insertion.
///
/// # Memory Layout
///
/// The cache pre-allocates space for 128 vertices to avoid frequent
/// reallocations during initial graph construction. It automatically
/// expands when vertices with higher IDs are accessed.
///
/// # Examples
///
/// ```
/// use phie::universe::cache::Cache;
///
/// let mut cache = Cache::new();
/// cache.put(0, 42);
/// assert_eq!(cache.get(0), Some(42));
/// assert_eq!(cache.get(1), None);
/// ```
pub struct Cache {
    /// Storage for cached values indexed by vertex ID
    data: Vec<Option<Data>>,
}

impl Cache {
    /// Creates a new empty cache with initial capacity.
    ///
    /// Pre-allocates space for 128 vertices to minimize reallocations
    /// during typical graph construction.
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::cache::Cache;
    ///
    /// let cache = Cache::new();
    /// assert_eq!(cache.len(), 0);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Cache { data: Vec::with_capacity(128) }
    }

    /// Stores a value in the cache for the given vertex.
    ///
    /// If the vertex ID is beyond the current cache size, the cache
    /// automatically expands to accommodate it, filling intermediate
    /// positions with None.
    ///
    /// # Arguments
    ///
    /// * `v` - Vertex ID to cache the value for
    /// * `value` - Data value to store
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::cache::Cache;
    ///
    /// let mut cache = Cache::new();
    /// cache.put(5, 100);
    /// assert_eq!(cache.get(5), Some(100));
    /// ```
    pub fn put(&mut self, v: VertexId, value: Data) {
        let v_usize = v as usize;
        if v_usize >= self.data.len() {
            self.data.resize(v_usize + 1, None);
        }
        self.data[v_usize] = Some(value);
    }

    /// Retrieves a cached value for the given vertex.
    ///
    /// Returns None if the vertex has not been cached or if the
    /// vertex ID is beyond the current cache size.
    ///
    /// # Arguments
    ///
    /// * `v` - Vertex ID to retrieve
    ///
    /// # Returns
    ///
    /// The cached data value if present, None otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::cache::Cache;
    ///
    /// let mut cache = Cache::new();
    /// cache.put(3, 42);
    /// assert_eq!(cache.get(3), Some(42));
    /// assert_eq!(cache.get(4), None);
    /// ```
    #[inline]
    pub fn get(&self, v: VertexId) -> Option<Data> {
        let v_usize = v as usize;
        if v_usize < self.data.len() {
            self.data[v_usize]
        } else {
            None
        }
    }

    /// Returns the current size of the cache storage.
    ///
    /// This represents the highest vertex ID + 1 that can be stored
    /// without reallocation, not the number of cached values.
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::cache::Cache;
    ///
    /// let mut cache = Cache::new();
    /// cache.put(10, 42);
    /// assert!(cache.len() >= 11);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Checks if the cache is empty.
    ///
    /// Returns true if no storage has been allocated yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::cache::Cache;
    ///
    /// let cache = Cache::new();
    /// assert!(cache.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clears all cached values while retaining allocated capacity.
    ///
    /// This is useful for reusing the cache with a new graph while
    /// avoiding reallocation overhead.
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::cache::Cache;
    ///
    /// let mut cache = Cache::new();
    /// cache.put(0, 42);
    /// cache.clear();
    /// assert_eq!(cache.get(0), None);
    /// ```
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = Cache::new();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_default() {
        let cache = Cache::default();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_put_and_get() {
        let mut cache = Cache::new();
        cache.put(0, 42);
        assert_eq!(cache.get(0), Some(42));
    }

    #[test]
    fn test_get_nonexistent() {
        let cache = Cache::new();
        assert_eq!(cache.get(0), None);
        assert_eq!(cache.get(100), None);
    }

    #[test]
    fn test_put_extends_cache() {
        let mut cache = Cache::new();
        cache.put(200, 99);
        assert!(cache.len() >= 201);
        assert_eq!(cache.get(200), Some(99));
    }

    #[test]
    fn test_multiple_values() {
        let mut cache = Cache::new();
        cache.put(0, 10);
        cache.put(1, 20);
        cache.put(2, 30);
        assert_eq!(cache.get(0), Some(10));
        assert_eq!(cache.get(1), Some(20));
        assert_eq!(cache.get(2), Some(30));
    }

    #[test]
    fn test_overwrite_value() {
        let mut cache = Cache::new();
        cache.put(5, 100);
        cache.put(5, 200);
        assert_eq!(cache.get(5), Some(200));
    }

    #[test]
    fn test_sparse_cache() {
        let mut cache = Cache::new();
        cache.put(0, 10);
        cache.put(10, 20);
        cache.put(20, 30);
        assert_eq!(cache.get(0), Some(10));
        assert_eq!(cache.get(5), None);
        assert_eq!(cache.get(10), Some(20));
        assert_eq!(cache.get(15), None);
        assert_eq!(cache.get(20), Some(30));
    }

    #[test]
    fn test_clear() {
        let mut cache = Cache::new();
        cache.put(0, 42);
        cache.put(1, 43);
        cache.clear();
        assert_eq!(cache.get(0), None);
        assert_eq!(cache.get(1), None);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_negative_data() {
        let mut cache = Cache::new();
        cache.put(0, -42);
        assert_eq!(cache.get(0), Some(-42));
    }

    #[test]
    fn test_zero_data() {
        let mut cache = Cache::new();
        cache.put(0, 0);
        assert_eq!(cache.get(0), Some(0));
    }

    #[test]
    fn test_max_i16() {
        let mut cache = Cache::new();
        cache.put(0, i16::MAX);
        assert_eq!(cache.get(0), Some(i16::MAX));
    }

    #[test]
    fn test_min_i16() {
        let mut cache = Cache::new();
        cache.put(0, i16::MIN);
        assert_eq!(cache.get(0), Some(i16::MIN));
    }
}
