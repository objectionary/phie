// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! SODG graph operations for the Universe.
//!
//! This module provides all graph manipulation operations including
//! vertex management, edge creation, data storage, and dataization.
//! It bridges between phie's internal data representation and SODG's
//! graph structure.

use crate::data::Data;
use crate::universe::{Cache, VertexId};
use sodg::{Hex, Sodg};

/// Core graph operations manager.
///
/// Operations handles all interactions with the underlying SODG graph,
/// providing type-safe wrappers that convert between phie's i16 data
/// type and SODG's Hex representation.
///
/// # Thread Safety
///
/// Operations is not thread-safe. External synchronization is required
/// if shared across threads.
pub struct Operations {
    /// The underlying SODG directed graph
    sodg: Sodg,
    /// Cache for dataized values
    cache: Cache,
    /// Counter for generating unique vertex IDs
    next_id: VertexId,
}

impl Operations {
    /// Creates a new Operations instance with an empty graph.
    ///
    /// Initializes the SODG graph, cache, and vertex ID counter.
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::Operations;
    ///
    /// let ops = Operations::new();
    /// ```
    pub fn new() -> Self {
        Operations {
            sodg: Sodg::empty(),
            cache: Cache::new(),
            next_id: 0,
        }
    }

    /// Generates the next unique vertex ID.
    ///
    /// Vertex IDs are sequential integers starting from 0.
    /// Each call increments the internal counter.
    ///
    /// # Returns
    ///
    /// A unique vertex ID that hasn't been used before
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::Operations;
    ///
    /// let mut ops = Operations::new();
    /// assert_eq!(ops.next_id(), 0);
    /// assert_eq!(ops.next_id(), 1);
    /// ```
    #[inline]
    pub fn next_id(&mut self) -> VertexId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Adds a new vertex to the graph.
    ///
    /// Creates a new vertex with the given ID. If the vertex already
    /// exists, the operation succeeds without modification.
    ///
    /// # Arguments
    ///
    /// * `v` - The vertex ID to add
    ///
    /// # Errors
    ///
    /// Returns an error if SODG validation fails or alerts trigger.
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::Operations;
    ///
    /// let mut ops = Operations::new();
    /// let v = ops.next_id();
    /// ops.add(v).unwrap();
    /// ```
    pub fn add(&mut self, v: VertexId) -> Result<(), String> {
        self.sodg
            .add(v)
            .map_err(|e| format!("Failed to add vertex: {e}"))
    }

    /// Stores data at a vertex.
    ///
    /// Converts the i16 data value to SODG's Hex format and stores it
    /// at the specified vertex. Also updates the cache with the value.
    ///
    /// # Arguments
    ///
    /// * `v` - The vertex ID
    /// * `data` - The i16 data value to store
    ///
    /// # Errors
    ///
    /// Returns an error if the vertex doesn't exist or SODG validation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::Operations;
    ///
    /// let mut ops = Operations::new();
    /// let v = ops.next_id();
    /// ops.add(v).unwrap();
    /// ops.put(v, 42).unwrap();
    /// ```
    pub fn put(&mut self, v: VertexId, data: Data) -> Result<(), String> {
        let bytes = data.to_be_bytes().to_vec();
        let hex = Hex::from_vec(bytes);
        self.sodg
            .put(v, &hex)
            .map_err(|e| format!("Failed to put data: {e}"))?;
        self.cache.put(v, data);
        Ok(())
    }

    /// Retrieves and dataizes a vertex value.
    ///
    /// First checks the cache for the value. If not found, retrieves
    /// from SODG, converts from Hex to i16, and caches the result.
    ///
    /// # Arguments
    ///
    /// * `v` - The vertex ID to dataize
    ///
    /// # Returns
    ///
    /// The i16 data value stored at the vertex
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The vertex doesn't exist
    /// - The stored data is not exactly 2 bytes
    /// - SODG operations fail
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::Operations;
    ///
    /// let mut ops = Operations::new();
    /// let v = ops.next_id();
    /// ops.add(v).unwrap();
    /// ops.put(v, 100).unwrap();
    /// assert_eq!(ops.dataize(v).unwrap(), 100);
    /// ```
    pub fn dataize(&mut self, v: VertexId) -> Result<Data, String> {
        if let Some(data) = self.cache.get(v) {
            return Ok(data);
        }
        let hex = self
            .sodg
            .data(v)
            .map_err(|e| format!("Failed to get data: {e}"))?;
        let bytes = hex.to_vec();
        if bytes.len() != 2 {
            let len = bytes.len();
            return Err(format!("Invalid data length: expected 2 bytes, got {len}"));
        }
        let data = Data::from_be_bytes([bytes[0], bytes[1]]);
        self.cache.put(v, data);
        Ok(data)
    }

    /// Creates a labeled edge between two vertices.
    ///
    /// Establishes a directed edge from the source vertex to the
    /// target vertex with the given label. The label typically
    /// represents object attributes like "𝛼0", "𝛼1", "ρ", etc.
    ///
    /// # Arguments
    ///
    /// * `from` - Source vertex ID
    /// * `to` - Target vertex ID
    /// * `label` - Edge label (attribute name)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Either vertex doesn't exist
    /// - The label is empty
    /// - from equals to (self-loops not allowed)
    /// - SODG validation fails
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::Operations;
    ///
    /// let mut ops = Operations::new();
    /// let v1 = ops.next_id();
    /// let v2 = ops.next_id();
    /// ops.add(v1).unwrap();
    /// ops.add(v2).unwrap();
    /// ops.bind(v1, v2, "𝛼0").unwrap();
    /// ```
    pub fn bind(&mut self, from: VertexId, to: VertexId, label: &str) -> Result<(), String> {
        self.sodg
            .bind(from, to, label)
            .map_err(|e| format!("Failed to bind: {e}"))
    }

    /// Resolves a vertex path to the vertex ID it points at.
    ///
    /// The path starts with a vertex reference `vN`, where N is the vertex
    /// ID number, and may continue with edge labels separated by dots,
    /// such as `v1.𝛼0` or `v1.𝛼0.𝛼1`. Every label is followed through
    /// the graph edges with `Sodg::kid`, and a missing edge is an error.
    ///
    /// # Arguments
    ///
    /// * `path` - Path string (e.g., "v0", "v1.𝛼0")
    ///
    /// # Returns
    ///
    /// The ID of the vertex the path points at
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path doesn't start with `v` followed by a number
    /// - Any label along the path has no matching edge
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::Operations;
    ///
    /// let mut ops = Operations::new();
    /// let v1 = ops.next_id();
    /// let v2 = ops.next_id();
    /// ops.add(v1).unwrap();
    /// ops.add(v2).unwrap();
    /// ops.bind(v1, v2, "𝛼0").unwrap();
    /// assert_eq!(ops.resolve_path("v0").unwrap(), 0);
    /// assert_eq!(ops.resolve_path("v0.𝛼0").unwrap(), 1);
    /// assert!(ops.resolve_path("v0.𝛼1").is_err());
    /// ```
    pub fn resolve_path(&self, path: &str) -> Result<VertexId, String> {
        let mut parts = path.split('.');
        let head = parts.next().unwrap_or_default();
        if !head.starts_with('v') {
            return Err(format!("Invalid path format: {path}"));
        }
        let mut v = head[1..]
            .parse::<VertexId>()
            .map_err(|e| format!("Invalid vertex ID: {e}"))?;
        for label in parts {
            v = self
                .sodg
                .kid(v, label)
                .ok_or_else(|| format!("No edge '{label}' from vertex {v} in path: {path}"))?;
        }
        Ok(v)
    }

    /// Dataizes the vertex a path points at.
    ///
    /// # Arguments
    ///
    /// * `path` - Path string (e.g., "v0", "v1.𝛼0")
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be resolved or the vertex
    /// reached holds no data of the right length.
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::Operations;
    ///
    /// let mut ops = Operations::new();
    /// let v1 = ops.next_id();
    /// let v2 = ops.next_id();
    /// ops.add(v1).unwrap();
    /// ops.add(v2).unwrap();
    /// ops.bind(v1, v2, "𝛼0").unwrap();
    /// ops.put(v2, 42).unwrap();
    /// assert_eq!(ops.dataize_path("v0.𝛼0").unwrap(), 42);
    /// ```
    pub fn dataize_path(&mut self, path: &str) -> Result<Data, String> {
        let v = self.resolve_path(path)?;
        self.dataize(v)
    }
}

impl Default for Operations {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operations_creation() {
        let ops = Operations::new();
        assert_eq!(ops.next_id, 0);
    }

    #[test]
    fn test_operations_default() {
        let ops = Operations::default();
        assert_eq!(ops.next_id, 0);
    }

    #[test]
    fn test_next_id_sequence() {
        let mut ops = Operations::new();
        assert_eq!(ops.next_id(), 0);
        assert_eq!(ops.next_id(), 1);
        assert_eq!(ops.next_id(), 2);
    }

    #[test]
    fn test_add_vertex() {
        let mut ops = Operations::new();
        let v = ops.next_id();
        assert!(ops.add(v).is_ok());
    }

    #[test]
    fn test_put_and_dataize() {
        let mut ops = Operations::new();
        let v = ops.next_id();
        ops.add(v).unwrap();
        ops.put(v, 42).unwrap();
        assert_eq!(ops.dataize(v).unwrap(), 42);
    }

    #[test]
    fn test_dataize_caches_value() {
        let mut ops = Operations::new();
        let v = ops.next_id();
        ops.add(v).unwrap();
        ops.put(v, 100).unwrap();
        assert_eq!(ops.dataize(v).unwrap(), 100);
        assert_eq!(ops.dataize(v).unwrap(), 100);
    }

    #[test]
    fn test_dataize_reads_graph_when_cache_is_cold() {
        let mut ops = Operations::new();
        let v = ops.next_id();
        ops.add(v).unwrap();
        ops.put(v, 7).unwrap();
        ops.cache = Cache::new();
        assert_eq!(ops.dataize(v).unwrap(), 7);
        assert_eq!(ops.cache.get(v), Some(7));
    }

    #[test]
    fn test_dataize_fails_on_data_of_wrong_length() {
        let mut ops = Operations::new();
        let v = ops.next_id();
        ops.add(v).unwrap();
        ops.sodg.put(v, &Hex::from_vec(vec![1])).unwrap();
        ops.cache = Cache::new();
        let err = ops.dataize(v).unwrap_err();
        assert!(err.contains("expected 2 bytes, got 1"), "{err}");
    }

    #[test]
    fn test_dataize_fails_on_vertex_without_data() {
        let mut ops = Operations::new();
        let v = ops.next_id();
        ops.add(v).unwrap();
        let err = ops.dataize(v).unwrap_err();
        assert!(err.contains("expected 2 bytes, got 0"), "{err}");
    }

    #[test]
    fn test_dataize_fails_on_unknown_vertex() {
        let mut ops = Operations::new();
        let err = ops.dataize(99).unwrap_err();
        assert!(err.contains("Failed to get data"), "{err}");
    }

    #[test]
    fn test_bind_vertices() {
        let mut ops = Operations::new();
        let v1 = ops.next_id();
        let v2 = ops.next_id();
        ops.add(v1).unwrap();
        ops.add(v2).unwrap();
        assert!(ops.bind(v1, v2, "𝛼0").is_ok());
    }

    #[test]
    fn test_resolve_path_valid() {
        let ops = Operations::new();
        assert_eq!(ops.resolve_path("v0").unwrap(), 0);
        assert_eq!(ops.resolve_path("v1").unwrap(), 1);
        assert_eq!(ops.resolve_path("v999").unwrap(), 999);
    }

    #[test]
    fn test_resolve_path_invalid() {
        let ops = Operations::new();
        assert!(ops.resolve_path("invalid").is_err());
        assert!(ops.resolve_path("x123").is_err());
        assert!(ops.resolve_path("").is_err());
        assert!(ops.resolve_path("vabc").is_err());
    }

    #[test]
    fn test_resolve_path_walks_edges() {
        let mut ops = Operations::new();
        let v1 = ops.next_id();
        let v2 = ops.next_id();
        let v3 = ops.next_id();
        ops.add(v1).unwrap();
        ops.add(v2).unwrap();
        ops.add(v3).unwrap();
        ops.bind(v1, v2, "𝛼0").unwrap();
        ops.bind(v2, v3, "𝛼1").unwrap();
        assert_eq!(ops.resolve_path("v0.𝛼0").unwrap(), 1);
        assert_eq!(ops.resolve_path("v0.𝛼0.𝛼1").unwrap(), 2);
    }

    #[test]
    fn test_resolve_path_fails_on_missing_edge() {
        let mut ops = Operations::new();
        let v1 = ops.next_id();
        let v2 = ops.next_id();
        ops.add(v1).unwrap();
        ops.add(v2).unwrap();
        ops.bind(v1, v2, "𝛼0").unwrap();
        let err = ops.resolve_path("v0.𝛼1").unwrap_err();
        assert!(err.contains("No edge '𝛼1'"), "{err}");
    }

    #[test]
    fn test_dataize_path_returns_edge_target_data() {
        let mut ops = Operations::new();
        let v1 = ops.next_id();
        let v2 = ops.next_id();
        ops.add(v1).unwrap();
        ops.add(v2).unwrap();
        ops.bind(v1, v2, "𝛼0").unwrap();
        ops.put(v1, 10).unwrap();
        ops.put(v2, 42).unwrap();
        assert_eq!(ops.dataize_path("v0.𝛼0").unwrap(), 42);
        assert_eq!(ops.dataize_path("v0").unwrap(), 10);
    }

    #[test]
    fn test_dataize_path_fails_on_missing_edge() {
        let mut ops = Operations::new();
        let v1 = ops.next_id();
        ops.add(v1).unwrap();
        assert!(ops.dataize_path("v0.𝛼0").is_err());
    }

    #[test]
    fn test_put_fails_on_unknown_vertex_without_caching() {
        let mut ops = Operations::new();
        let v = ops.next_id();
        assert!(ops.put(v, 42).is_err());
        assert!(
            ops.dataize(v).is_err(),
            "the cache must not hold a rejected value"
        );
    }

    #[test]
    fn test_negative_data() {
        let mut ops = Operations::new();
        let v = ops.next_id();
        ops.add(v).unwrap();
        ops.put(v, -42).unwrap();
        assert_eq!(ops.dataize(v).unwrap(), -42);
    }

    #[test]
    fn test_zero_data() {
        let mut ops = Operations::new();
        let v = ops.next_id();
        ops.add(v).unwrap();
        ops.put(v, 0).unwrap();
        assert_eq!(ops.dataize(v).unwrap(), 0);
    }

    #[test]
    fn test_max_min_i16() {
        let mut ops = Operations::new();
        let v1 = ops.next_id();
        let v2 = ops.next_id();
        ops.add(v1).unwrap();
        ops.add(v2).unwrap();
        ops.put(v1, i16::MAX).unwrap();
        ops.put(v2, i16::MIN).unwrap();
        assert_eq!(ops.dataize(v1).unwrap(), i16::MAX);
        assert_eq!(ops.dataize(v2).unwrap(), i16::MIN);
    }
}
