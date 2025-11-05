//! Python symbol extraction
//!
//! Extracts functions, classes, and imports from Python AST.

use analyzer_core::{Symbol, SymbolKind};
use anyhow::Result;
use tree_sitter::{Node, Tree, TreeCursor};

/// Extract symbols from a Python parse tree
pub fn extract_symbols(tree: &Tree, source: &str) -> Result<Vec<Symbol>> {
    let mut symbols = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    extract_from_node(&mut cursor, source, &mut symbols, None, 0)?;

    Ok(symbols)
}

/// Recursively extract symbols from a node
fn extract_from_node(
    cursor: &mut TreeCursor,
    source: &str,
    symbols: &mut Vec<Symbol>,
    parent_scope: Option<String>,
    _file_id: i64,
) -> Result<()> {
    let node = cursor.node();

    match node.kind() {
        "function_definition" => {
            if let Some(symbol) = extract_function(node, source, parent_scope.as_deref())? {
                // Extract nested symbols from function body
                let function_scope = Some(symbol.name.clone());
                symbols.push(symbol);

                if cursor.goto_first_child() {
                    loop {
                        extract_from_node(cursor, source, symbols, function_scope.clone(), _file_id)?;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
        }
        "class_definition" => {
            if let Some(symbol) = extract_class(node, source, parent_scope.as_deref())? {
                // Extract nested symbols from class body
                let class_scope = Some(symbol.name.clone());
                symbols.push(symbol);

                if cursor.goto_first_child() {
                    loop {
                        extract_from_node(cursor, source, symbols, class_scope.clone(), _file_id)?;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
        }
        "import_statement" | "import_from_statement" => {
            if let Some(symbol) = extract_import(node, source, parent_scope.as_deref())? {
                symbols.push(symbol);
            }
        }
        "assignment" => {
            // Extract variable assignments (module-level only for now)
            if parent_scope.is_none() {
                if let Some(symbol) = extract_variable(node, source)? {
                    symbols.push(symbol);
                }
            }
        }
        _ => {
            // Recurse into children
            if cursor.goto_first_child() {
                loop {
                    extract_from_node(cursor, source, symbols, parent_scope.clone(), _file_id)?;
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }
        }
    }

    Ok(())
}

/// Extract a function definition
fn extract_function(node: Node, source: &str, scope: Option<&str>) -> Result<Option<Symbol>> {
    let name_node = node
        .child_by_field_name("name")
        .ok_or_else(|| anyhow::anyhow!("Function has no name"))?;

    let name = node_text(name_node, source);
    let line_start = node.start_position().row;
    let line_end = node.end_position().row;

    Ok(Some(Symbol {
        id: None,
        file_id: 0, // Will be set by caller
        name,
        kind: SymbolKind::Function,
        line_start,
        line_end,
        scope: scope.map(|s| s.to_string()),
        metadata: None,
    }))
}

/// Extract a class definition
fn extract_class(node: Node, source: &str, scope: Option<&str>) -> Result<Option<Symbol>> {
    let name_node = node
        .child_by_field_name("name")
        .ok_or_else(|| anyhow::anyhow!("Class has no name"))?;

    let name = node_text(name_node, source);
    let line_start = node.start_position().row;
    let line_end = node.end_position().row;

    Ok(Some(Symbol {
        id: None,
        file_id: 0,
        name,
        kind: SymbolKind::Class,
        line_start,
        line_end,
        scope: scope.map(|s| s.to_string()),
        metadata: None,
    }))
}

/// Extract an import statement
fn extract_import(node: Node, source: &str, scope: Option<&str>) -> Result<Option<Symbol>> {
    // Try to get the module name
    let name = if node.kind() == "import_statement" {
        // import foo
        node.child_by_field_name("name")
            .map(|n| node_text(n, source))
    } else {
        // from foo import bar
        node.child_by_field_name("module_name")
            .map(|n| node_text(n, source))
    };

    if let Some(name) = name {
        let line_start = node.start_position().row;
        let line_end = node.end_position().row;

        Ok(Some(Symbol {
            id: None,
            file_id: 0,
            name,
            kind: SymbolKind::Import,
            line_start,
            line_end,
            scope: scope.map(|s| s.to_string()),
            metadata: None,
        }))
    } else {
        Ok(None)
    }
}

/// Extract a variable assignment
fn extract_variable(node: Node, source: &str) -> Result<Option<Symbol>> {
    // Get the left side of the assignment
    let left = node.child_by_field_name("left");

    if let Some(left_node) = left {
        if left_node.kind() == "identifier" {
            let name = node_text(left_node, source);
            let line_start = node.start_position().row;
            let line_end = node.end_position().row;

            return Ok(Some(Symbol {
                id: None,
                file_id: 0,
                name,
                kind: SymbolKind::Variable,
                line_start,
                line_end,
                scope: None,
                metadata: None,
            }));
        }
    }

    Ok(None)
}

/// Get text content of a node
fn node_text(node: Node, source: &str) -> String {
    source[node.byte_range()].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::PythonParser;

    #[test]
    fn test_extract_function() {
        let source = r#"
def my_function():
    pass
"#;
        let mut parser = PythonParser::new().unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = extract_symbols(&tree, source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "my_function");
        assert!(matches!(symbols[0].kind, SymbolKind::Function));
    }

    #[test]
    fn test_extract_class_with_methods() {
        let source = r#"
class MyClass:
    def __init__(self):
        pass

    def method(self):
        pass
"#;
        let mut parser = PythonParser::new().unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = extract_symbols(&tree, source).unwrap();

        // Should find class + 2 methods
        assert!(symbols.len() >= 3);
        assert!(symbols.iter().any(|s| s.name == "MyClass" && matches!(s.kind, SymbolKind::Class)));
        assert!(symbols.iter().any(|s| s.name == "__init__" && matches!(s.kind, SymbolKind::Function)));
        assert!(symbols.iter().any(|s| s.name == "method" && matches!(s.kind, SymbolKind::Function)));
    }

    #[test]
    fn test_extract_imports() {
        let source = r#"
import os
from pathlib import Path
"#;
        let mut parser = PythonParser::new().unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = extract_symbols(&tree, source).unwrap();

        assert!(symbols.iter().any(|s| s.name == "os" && matches!(s.kind, SymbolKind::Import)));
        assert!(symbols.iter().any(|s| s.name == "pathlib" && matches!(s.kind, SymbolKind::Import)));
    }
}
