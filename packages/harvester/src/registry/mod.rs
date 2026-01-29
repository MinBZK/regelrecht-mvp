//! Element registry system for extensible XML parsing.
//!
//! This module provides a registry-based approach to parsing Dutch law XML.
//! Element handlers can be registered for specific tag names, allowing for
//! extensible and testable parsing.

mod config;
mod engine;
mod handler;
pub mod handlers;
mod registry;
mod types;

pub use config::create_content_registry;
pub use engine::ParseEngine;
pub use handler::{extract_text_with_tail, ElementHandler, RecurseFn};
pub use registry::ElementRegistry;
pub use types::{ElementType, ParseContext, ParseResult, ReferenceCollector};
