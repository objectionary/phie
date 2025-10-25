// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! Universe module provides a bridge between phie and SODG for Rust atoms.
//!
//! This module implements the Universe abstraction that allows dynamically compiled
//! Rust atoms to interact with the object graph through SODG API. The Universe
//! manages vertex creation, data storage, and graph traversal operations.
//!
//! The key abstraction is the Universe struct which wraps SODG and provides
//! a type-safe interface for Rust atoms to perform graph operations during
//! dataization.

mod cache;
mod operations;

pub use cache::Cache;
pub use operations::Operations;

use crate::data::Data;

/// Type alias for vertex identifiers in the graph.
///
/// Vertices are numbered sequentially starting from 0. Each vertex can store
/// data and have labeled edges to other vertices representing object attributes.
pub type VertexId = u32;

/// Universe provides the runtime interface for Rust atoms to interact with SODG.
///
/// Universe wraps graph operations and provides methods for:
/// - Creating new vertices with unique IDs
/// - Storing and retrieving i16 data values at vertices
/// - Creating labeled edges between vertices
/// - Resolving paths through the object graph
/// - Caching dataized values for performance
///
/// The Universe is passed to compiled Rust atoms as a mutable pointer, allowing
/// them to query and modify the object graph during execution.
///
/// # Thread Safety
///
/// Universe is not thread-safe and should be accessed from a single thread
/// or protected by synchronization primitives when shared.
///
/// # Examples
///
/// ```
/// use phie::universe::{Universe, VertexId};
/// use phie::data::Data;
///
/// let mut uni = Universe::new();
/// let v = uni.next_id();
/// uni.add(v).unwrap();
/// uni.put(v, 42).unwrap();
/// assert_eq!(uni.dataize_vertex(v).unwrap(), 42);
/// ```
pub struct Universe {
    ops: Operations,
}

impl Universe {
    /// Creates a new empty Universe.
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::Universe;
    ///
    /// let uni = Universe::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Universe {
            ops: Operations::new(),
        }
    }

    /// Returns the next available vertex ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::Universe;
    ///
    /// let mut uni = Universe::new();
    /// let id1 = uni.next_id();
    /// let id2 = uni.next_id();
    /// assert_eq!(id2, id1 + 1);
    /// ```
    #[inline]
    pub fn next_id(&mut self) -> VertexId {
        self.ops.next_id()
    }

    /// Adds a new vertex to the graph.
    ///
    /// # Arguments
    ///
    /// * `v` - The vertex ID to add
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::Universe;
    ///
    /// let mut uni = Universe::new();
    /// let v = uni.next_id();
    /// uni.add(v).unwrap();
    /// ```
    pub fn add(&mut self, v: VertexId) -> Result<(), String> {
        self.ops.add(v)
    }

    /// Stores data at a vertex.
    ///
    /// # Arguments
    ///
    /// * `v` - The vertex ID
    /// * `data` - The data value to store
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::Universe;
    ///
    /// let mut uni = Universe::new();
    /// let v = uni.next_id();
    /// uni.add(v).unwrap();
    /// uni.put(v, 42).unwrap();
    /// ```
    pub fn put(&mut self, v: VertexId, data: Data) -> Result<(), String> {
        self.ops.put(v, data)
    }

    /// Retrieves and dataizes a vertex by path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the vertex (e.g., "v1.𝛼0")
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::Universe;
    ///
    /// let mut uni = Universe::new();
    /// let v = uni.next_id();
    /// uni.add(v).unwrap();
    /// uni.put(v, 42).unwrap();
    /// assert_eq!(uni.dataize("v0").unwrap(), 42);
    /// ```
    pub fn dataize(&mut self, path: &str) -> Result<Data, String> {
        let v = self.ops.parse_path(path)?;
        self.dataize_vertex(v)
    }

    /// Retrieves data from a vertex by ID.
    ///
    /// # Arguments
    ///
    /// * `v` - The vertex ID
    pub fn dataize_vertex(&mut self, v: VertexId) -> Result<Data, String> {
        self.ops.dataize(v)
    }

    /// Creates an edge between two vertices.
    ///
    /// # Arguments
    ///
    /// * `from` - Source vertex ID
    /// * `to` - Target vertex ID
    /// * `label` - Edge label
    pub fn bind(&mut self, from: VertexId, to: VertexId, label: &str) -> Result<(), String> {
        self.ops.bind(from, to, label)
    }
}

impl Default for Universe {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universe_creation() {
        let uni = Universe::new();
        let _ = uni;
    }

    #[test]
    fn test_universe_default() {
        let uni = Universe::default();
        let _ = uni;
    }

    #[test]
    fn test_complete_workflow() {
        let mut uni = Universe::new();
        let v = uni.next_id();
        uni.add(v).unwrap();
        uni.put(v, 42).unwrap();
        assert_eq!(uni.dataize_vertex(v).unwrap(), 42);
    }

    #[test]
    fn test_dataize_by_path() {
        let mut uni = Universe::new();
        let v = uni.next_id();
        uni.add(v).unwrap();
        uni.put(v, 100).unwrap();
        assert_eq!(uni.dataize("v0").unwrap(), 100);
    }

    #[test]
    fn test_bind_workflow() {
        let mut uni = Universe::new();
        let v1 = uni.next_id();
        let v2 = uni.next_id();
        uni.add(v1).unwrap();
        uni.add(v2).unwrap();
        uni.bind(v1, v2, "𝛼0").unwrap();
        uni.put(v1, 10).unwrap();
        uni.put(v2, 20).unwrap();
        assert_eq!(uni.dataize_vertex(v1).unwrap(), 10);
        assert_eq!(uni.dataize_vertex(v2).unwrap(), 20);
    }

    #[test]
    fn test_multiple_vertices() {
        let mut uni = Universe::new();
        for i in 0..10 {
            let v = uni.next_id();
            uni.add(v).unwrap();
            uni.put(v, i as i16).unwrap();
        }
        for i in 0..10 {
            assert_eq!(uni.dataize_vertex(i).unwrap(), i as i16);
        }
    }

    #[test]
    fn test_invalid_path() {
        let mut uni = Universe::new();
        assert!(uni.dataize("invalid").is_err());
    }
}
