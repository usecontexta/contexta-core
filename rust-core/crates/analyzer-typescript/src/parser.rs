//! TypeScript Tree-sitter parser
//!
//! Wraps the tree-sitter-typescript parser for use in the analyzer.

use anyhow::{Context, Result};
use tree_sitter::{Parser, Tree};

/// TypeScript language parser
pub struct TypeScriptParser {
    parser: Parser,
}

impl TypeScriptParser {
    /// Create a new TypeScript parser
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .context("Failed to set TypeScript language for parser")?;

        Ok(Self { parser })
    }

    /// Parse TypeScript source code
    pub fn parse(&mut self, source: &str) -> Result<Tree> {
        self.parser
            .parse(source, None)
            .context("Failed to parse TypeScript source")
    }

    /// Parse with old tree for incremental parsing
    pub fn parse_with_old_tree(&mut self, source: &str, old_tree: &Tree) -> Result<Tree> {
        self.parser
            .parse(source, Some(old_tree))
            .context("Failed to incrementally parse TypeScript source")
    }
}

impl Default for TypeScriptParser {
    fn default() -> Self {
        Self::new().expect("Failed to create TypeScript parser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let mut parser = TypeScriptParser::new().unwrap();
        let source = "function hello(): void {\n    console.log('Hello');\n}";
        let tree = parser.parse(source).unwrap();

        let root = tree.root_node();
        assert_eq!(root.kind(), "program");
        assert!(root.child_count() > 0);
    }

    #[test]
    fn test_parse_class() {
        let mut parser = TypeScriptParser::new().unwrap();
        let source = "class MyClass {\n    constructor() {}\n}";
        let tree = parser.parse(source).unwrap();

        let root = tree.root_node();
        assert!(root.to_sexp().contains("class_declaration"));
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let mut parser = TypeScriptParser::new().unwrap();
        let source = "function hello(\n    // Unclosed parenthesis";
        let tree = parser.parse(source).unwrap();

        // Tree-sitter should still produce a tree even with errors
        assert!(tree.root_node().has_error());
    }
}
