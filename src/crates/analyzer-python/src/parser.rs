//! Python Tree-sitter parser
//!
//! Wraps the tree-sitter-python parser for use in the analyzer.

use anyhow::{Context, Result};
use tree_sitter::{Parser, Tree};

/// Python language parser
pub struct PythonParser {
    parser: Parser,
}

impl PythonParser {
    /// Create a new Python parser
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .context("Failed to set Python language for parser")?;

        Ok(Self { parser })
    }

    /// Parse Python source code
    pub fn parse(&mut self, source: &str) -> Result<Tree> {
        self.parser
            .parse(source, None)
            .context("Failed to parse Python source")
    }

    /// Parse with old tree for incremental parsing
    pub fn parse_with_old_tree(&mut self, source: &str, old_tree: &Tree) -> Result<Tree> {
        self.parser
            .parse(source, Some(old_tree))
            .context("Failed to incrementally parse Python source")
    }
}

impl Default for PythonParser {
    fn default() -> Self {
        Self::new().expect("Failed to create Python parser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let mut parser = PythonParser::new().unwrap();
        let source = "def hello():\n    print('Hello')";
        let tree = parser.parse(source).unwrap();

        let root = tree.root_node();
        assert_eq!(root.kind(), "module");
        assert!(root.child_count() > 0);
    }

    #[test]
    fn test_parse_class() {
        let mut parser = PythonParser::new().unwrap();
        let source = "class MyClass:\n    def __init__(self):\n        pass";
        let tree = parser.parse(source).unwrap();

        let root = tree.root_node();
        assert!(root.to_sexp().contains("class_definition"));
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let mut parser = PythonParser::new().unwrap();
        let source = "def hello(\n    # Unclosed parenthesis";
        let tree = parser.parse(source).unwrap();

        // Tree-sitter should still produce a tree even with errors
        assert!(tree.root_node().has_error());
    }
}
