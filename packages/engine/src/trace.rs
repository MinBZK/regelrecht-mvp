//! Execution tracing for audit trails and debugging
//!
//! This module provides structures and utilities for recording the execution
//! path through law evaluation. This is useful for:
//!
//! - **Audit trails**: Documenting exactly how a decision was reached
//! - **Debugging**: Understanding why a particular result was produced
//! - **Explainability**: Providing transparency in automated legal decisions
//!
//! # Example
//!
//! ```ignore
//! use regelrecht_engine::trace::{PathNode, PathNodeType, TraceBuilder};
//!
//! let mut builder = TraceBuilder::new();
//!
//! // Start resolving a variable
//! builder.push("inkomen", PathNodeType::Resolve);
//!
//! // Nested operation
//! builder.push("vergelijk_drempel", PathNodeType::Operation);
//! builder.set_result(Value::Bool(true));
//! builder.pop();
//!
//! // Complete the resolution
//! builder.set_result(Value::Int(50000));
//! builder.pop();
//!
//! let trace = builder.build();
//! ```

use crate::types::{PathNodeType, ResolveType, Value};
use serde::Serialize;
use std::time::Instant;

/// A node in the execution trace tree.
///
/// Each node represents a single step in the execution process, such as:
/// - Resolving a variable
/// - Executing an operation
/// - Evaluating an action
///
/// Nodes can have children, forming a tree structure that mirrors the
/// nested nature of law evaluation.
#[derive(Debug, Clone, Serialize)]
pub struct PathNode {
    /// Type of this execution step
    pub node_type: PathNodeType,

    /// Name or identifier for this step (e.g., variable name, operation type)
    pub name: String,

    /// The result value produced by this step, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// For resolve nodes, indicates how the value was resolved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_type: Option<ResolveType>,

    /// Child nodes representing nested execution steps
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<PathNode>,

    /// Execution duration in microseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_us: Option<u64>,
}

impl PathNode {
    /// Create a new PathNode with the given type and name.
    pub fn new(node_type: PathNodeType, name: impl Into<String>) -> Self {
        Self {
            node_type,
            name: name.into(),
            result: None,
            resolve_type: None,
            children: Vec::new(),
            duration_us: None,
        }
    }

    /// Set the result value for this node.
    pub fn with_result(mut self, result: Value) -> Self {
        self.result = Some(result);
        self
    }

    /// Set the resolve type for this node.
    pub fn with_resolve_type(mut self, resolve_type: ResolveType) -> Self {
        self.resolve_type = Some(resolve_type);
        self
    }

    /// Add a child node.
    pub fn with_child(mut self, child: PathNode) -> Self {
        self.children.push(child);
        self
    }

    /// Set the duration in microseconds.
    pub fn with_duration(mut self, duration_us: u64) -> Self {
        self.duration_us = Some(duration_us);
        self
    }
}

/// A node being built, with timing information.
#[derive(Debug)]
struct BuildingNode {
    node: PathNode,
    start_time: Instant,
}

/// Builder for constructing execution traces using a stack-based approach.
///
/// The builder maintains a stack of nodes being constructed. As execution
/// proceeds, nodes are pushed when entering a new scope and popped when
/// leaving. This naturally produces the tree structure of nested execution.
#[derive(Debug)]
pub struct TraceBuilder {
    /// Stack of nodes being built (last is current)
    stack: Vec<BuildingNode>,

    /// Whether tracing is enabled
    enabled: bool,
}

impl Default for TraceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceBuilder {
    /// Create a new TraceBuilder with tracing enabled.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            enabled: true,
        }
    }

    /// Create a new TraceBuilder with tracing disabled (no-op).
    pub fn disabled() -> Self {
        Self {
            stack: Vec::new(),
            enabled: false,
        }
    }

    /// Check if tracing is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Push a new node onto the stack.
    ///
    /// Call this when entering a new execution scope (resolving a variable,
    /// starting an operation, etc.).
    pub fn push(&mut self, name: impl Into<String>, node_type: PathNodeType) {
        if !self.enabled {
            return;
        }

        let node = PathNode::new(node_type, name);
        self.stack.push(BuildingNode {
            node,
            start_time: Instant::now(),
        });
    }

    /// Set the result value for the current node.
    pub fn set_result(&mut self, result: Value) {
        if !self.enabled {
            return;
        }

        if let Some(current) = self.stack.last_mut() {
            current.node.result = Some(result);
        }
    }

    /// Set the resolve type for the current node.
    pub fn set_resolve_type(&mut self, resolve_type: ResolveType) {
        if !self.enabled {
            return;
        }

        if let Some(current) = self.stack.last_mut() {
            current.node.resolve_type = Some(resolve_type);
        }
    }

    /// Pop the current node from the stack, making it a child of the parent.
    ///
    /// Returns the popped node. If this was the last node on the stack,
    /// returns the completed root node.
    pub fn pop(&mut self) -> Option<PathNode> {
        if !self.enabled {
            return None;
        }

        let building = self.stack.pop()?;
        let duration = building.start_time.elapsed().as_micros() as u64;

        let mut completed = building.node;
        completed.duration_us = Some(duration);

        // If there's a parent, add this as a child
        if let Some(parent) = self.stack.last_mut() {
            parent.node.children.push(completed.clone());
        }

        Some(completed)
    }

    /// Build the final trace, consuming the builder.
    ///
    /// Pops all remaining nodes from the stack, returning the root node.
    /// Returns None if the stack is empty or tracing was disabled.
    pub fn build(mut self) -> Option<PathNode> {
        if !self.enabled {
            return None;
        }

        // Pop all nodes to build complete tree
        let mut result = None;
        while !self.stack.is_empty() {
            result = self.pop();
        }
        result
    }

    /// Get the current depth of the trace stack.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Check if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_node_creation() {
        let node = PathNode::new(PathNodeType::Resolve, "test_var");
        assert_eq!(node.name, "test_var");
        assert!(matches!(node.node_type, PathNodeType::Resolve));
        assert!(node.result.is_none());
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_path_node_builder_pattern() {
        let node = PathNode::new(PathNodeType::Operation, "ADD")
            .with_result(Value::Int(42))
            .with_child(PathNode::new(PathNodeType::Resolve, "a"))
            .with_child(PathNode::new(PathNodeType::Resolve, "b"));

        assert_eq!(node.result, Some(Value::Int(42)));
        assert_eq!(node.children.len(), 2);
    }

    #[test]
    fn test_trace_builder_simple() {
        let mut builder = TraceBuilder::new();
        assert!(builder.is_enabled());
        assert!(builder.is_empty());

        builder.push("root", PathNodeType::Resolve);
        assert_eq!(builder.depth(), 1);

        builder.set_result(Value::Int(100));
        let node = builder.pop().unwrap();

        assert_eq!(node.name, "root");
        assert_eq!(node.result, Some(Value::Int(100)));
        assert!(node.duration_us.is_some());
    }

    #[test]
    fn test_trace_builder_nested() {
        let mut builder = TraceBuilder::new();

        // Root node
        builder.push("calculate", PathNodeType::Action);

        // Nested operation
        builder.push("add_values", PathNodeType::Operation);
        builder.set_result(Value::Int(30));
        builder.pop();

        // Complete root
        builder.set_result(Value::Int(30));
        let root = builder.build().unwrap();

        assert_eq!(root.name, "calculate");
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].name, "add_values");
        assert_eq!(root.children[0].result, Some(Value::Int(30)));
    }

    #[test]
    fn test_trace_builder_disabled() {
        let mut builder = TraceBuilder::disabled();
        assert!(!builder.is_enabled());

        builder.push("should_be_ignored", PathNodeType::Resolve);
        builder.set_result(Value::Int(42));

        assert!(builder.is_empty()); // Nothing was actually pushed
        assert!(builder.pop().is_none());
        assert!(builder.build().is_none());
    }

    #[test]
    fn test_trace_builder_resolve_type() {
        let mut builder = TraceBuilder::new();

        builder.push("param", PathNodeType::Resolve);
        builder.set_resolve_type(ResolveType::Parameter);
        builder.set_result(Value::String("test".to_string()));

        let node = builder.pop().unwrap();
        assert_eq!(node.resolve_type, Some(ResolveType::Parameter));
    }

    #[test]
    fn test_path_node_serialization() {
        let node = PathNode::new(PathNodeType::Resolve, "test")
            .with_result(Value::Int(42))
            .with_resolve_type(ResolveType::Definition);

        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"result\":42"));
    }

    #[test]
    fn test_deeply_nested_trace() {
        let mut builder = TraceBuilder::new();

        // Build a 3-level deep trace
        builder.push("level1", PathNodeType::Action);
        builder.push("level2", PathNodeType::Operation);
        builder.push("level3", PathNodeType::Resolve);
        builder.set_result(Value::Int(1));
        builder.pop(); // level3

        builder.set_result(Value::Int(2));
        builder.pop(); // level2

        builder.set_result(Value::Int(3));
        let root = builder.build().unwrap();

        assert_eq!(root.name, "level1");
        assert_eq!(root.result, Some(Value::Int(3)));
        assert_eq!(root.children.len(), 1);

        let level2 = &root.children[0];
        assert_eq!(level2.name, "level2");
        assert_eq!(level2.result, Some(Value::Int(2)));
        assert_eq!(level2.children.len(), 1);

        let level3 = &level2.children[0];
        assert_eq!(level3.name, "level3");
        assert_eq!(level3.result, Some(Value::Int(1)));
        assert!(level3.children.is_empty());
    }
}
