//! Rust Tree-sitter parser
//!
//! Wraps the tree-sitter-rust parser for use in the analyzer.

use anyhow::{Context, Result};
use tree_sitter::{Parser, Tree};

/// Rust language parser
pub struct RustParser {
    parser: Parser,
}

impl RustParser {
    /// Create a new Rust parser
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .context("Failed to set Rust language for parser")?;

        Ok(Self { parser })
    }

    /// Parse Rust source code
    pub fn parse(&mut self, source: &str) -> Result<Tree> {
        self.parser
            .parse(source, None)
            .context("Failed to parse Rust source")
    }

    /// Parse with old tree for incremental parsing
    pub fn parse_with_old_tree(&mut self, source: &str, old_tree: &Tree) -> Result<Tree> {
        self.parser
            .parse(source, Some(old_tree))
            .context("Failed to incrementally parse Rust source")
    }
}

impl Default for RustParser {
    fn default() -> Self {
        Self::new().expect("Failed to create Rust parser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let mut parser = RustParser::new().unwrap();
        let source = "fn hello() {\n    println!(\"Hello\");\n}";
        let tree = parser.parse(source).unwrap();

        let root = tree.root_node();
        assert_eq!(root.kind(), "source_file");
        assert!(root.child_count() > 0);
    }

    #[test]
    fn test_parse_struct() {
        let mut parser = RustParser::new().unwrap();
        let source = "struct MyStruct {\n    value: i32,\n}";
        let tree = parser.parse(source).unwrap();

        let root = tree.root_node();
        assert!(root.to_sexp().contains("struct_item"));
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let mut parser = RustParser::new().unwrap();
        let source = "fn hello(\n    // Unclosed parenthesis";
        let tree = parser.parse(source).unwrap();

        // Tree-sitter should still produce a tree even with errors
        assert!(tree.root_node().has_error());
    }
}
