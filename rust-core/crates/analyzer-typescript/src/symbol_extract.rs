//! TypeScript symbol extraction
//!
//! Extracts functions, classes, interfaces, types, and imports from TypeScript AST.

use analyzer_core::{Symbol, SymbolKind};
use anyhow::Result;
use tree_sitter::{Node, Tree, TreeCursor};

/// Extract symbols from a TypeScript parse tree
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
        "function_declaration" | "function" | "arrow_function" | "method_definition" => {
            if let Some(symbol) = extract_function(node, source, parent_scope.as_deref())? {
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
        "class_declaration" | "class" => {
            if let Some(symbol) = extract_class(node, source, parent_scope.as_deref())? {
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
        "interface_declaration" => {
            if let Some(symbol) = extract_interface(node, source, parent_scope.as_deref())? {
                symbols.push(symbol);
            }
        }
        "type_alias_declaration" => {
            if let Some(symbol) = extract_type_alias(node, source, parent_scope.as_deref())? {
                symbols.push(symbol);
            }
        }
        "import_statement" => {
            if let Some(symbol) = extract_import(node, source, parent_scope.as_deref())? {
                symbols.push(symbol);
            }
        }
        "export_statement" => {
            if let Some(symbol) = extract_export(node, source, parent_scope.as_deref())? {
                symbols.push(symbol);
            }
        }
        "lexical_declaration" | "variable_declaration" => {
            // Extract const/let/var declarations (module-level only for now)
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

/// Extract a function declaration
fn extract_function(node: Node, source: &str, scope: Option<&str>) -> Result<Option<Symbol>> {
    let name_node = node.child_by_field_name("name");

    let name = if let Some(name_node) = name_node {
        node_text(name_node, source)
    } else {
        // Anonymous function or arrow function
        return Ok(None);
    };

    let line_start = node.start_position().row;
    let line_end = node.end_position().row;

    Ok(Some(Symbol {
        id: None,
        file_id: 0,
        name,
        kind: SymbolKind::Function,
        line_start,
        line_end,
        scope: scope.map(|s| s.to_string()),
        metadata: None,
    }))
}

/// Extract a class declaration
fn extract_class(node: Node, source: &str, scope: Option<&str>) -> Result<Option<Symbol>> {
    let name_node = node.child_by_field_name("name");

    // Skip anonymous classes
    if name_node.is_none() {
        return Ok(None);
    }

    let name = node_text(name_node.unwrap(), source);
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

/// Extract an interface declaration
fn extract_interface(node: Node, source: &str, scope: Option<&str>) -> Result<Option<Symbol>> {
    let name_node = node
        .child_by_field_name("name")
        .ok_or_else(|| anyhow::anyhow!("Interface has no name"))?;

    let name = node_text(name_node, source);
    let line_start = node.start_position().row;
    let line_end = node.end_position().row;

    Ok(Some(Symbol {
        id: None,
        file_id: 0,
        name,
        kind: SymbolKind::Type,
        line_start,
        line_end,
        scope: scope.map(|s| s.to_string()),
        metadata: None,
    }))
}

/// Extract a type alias declaration
fn extract_type_alias(node: Node, source: &str, scope: Option<&str>) -> Result<Option<Symbol>> {
    let name_node = node
        .child_by_field_name("name")
        .ok_or_else(|| anyhow::anyhow!("Type alias has no name"))?;

    let name = node_text(name_node, source);
    let line_start = node.start_position().row;
    let line_end = node.end_position().row;

    Ok(Some(Symbol {
        id: None,
        file_id: 0,
        name,
        kind: SymbolKind::Type,
        line_start,
        line_end,
        scope: scope.map(|s| s.to_string()),
        metadata: None,
    }))
}

/// Extract an import statement
fn extract_import(node: Node, source: &str, scope: Option<&str>) -> Result<Option<Symbol>> {
    // Try to get the module specifier
    let source_node = node.child_by_field_name("source");

    if let Some(source_node) = source_node {
        let mut name = node_text(source_node, source);
        // Remove quotes
        name = name.trim_matches(|c| c == '"' || c == '\'').to_string();

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

/// Extract an export statement
fn extract_export(node: Node, source: &str, scope: Option<&str>) -> Result<Option<Symbol>> {
    // Check if it's a named export
    let declaration = node.child_by_field_name("declaration");

    if let Some(decl_node) = declaration {
        // Extract the exported declaration recursively
        // For now, just skip - we'll handle the declaration itself when we encounter it
        Ok(None)
    } else {
        Ok(None)
    }
}

/// Extract a variable declaration
fn extract_variable(node: Node, source: &str) -> Result<Option<Symbol>> {
    // Get the variable declarator
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.kind() == "variable_declarator" {
                let name_node = child.child_by_field_name("name");
                if let Some(name_node) = name_node {
                    if name_node.kind() == "identifier" {
                        let name = node_text(name_node, source);
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
            }
            if !cursor.goto_next_sibling() {
                break;
            }
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
    use crate::parser::TypeScriptParser;

    #[test]
    fn test_extract_function() {
        let source = r#"
function myFunction() {
    console.log("test");
}
"#;
        let mut parser = TypeScriptParser::new().unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = extract_symbols(&tree, source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "myFunction");
        assert!(matches!(symbols[0].kind, SymbolKind::Function));
    }

    #[test]
    fn test_extract_class_with_methods() {
        let source = r#"
class MyClass {
    constructor() {
        this.value = 42;
    }

    getValue() {
        return this.value;
    }
}
"#;
        let mut parser = TypeScriptParser::new().unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = extract_symbols(&tree, source).unwrap();

        assert!(symbols.iter().any(|s| s.name == "MyClass" && matches!(s.kind, SymbolKind::Class)));
        assert!(symbols.iter().any(|s| s.name == "getValue" && matches!(s.kind, SymbolKind::Function)));
    }

    #[test]
    fn test_extract_interface() {
        let source = r#"
interface User {
    name: string;
    age: number;
}
"#;
        let mut parser = TypeScriptParser::new().unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = extract_symbols(&tree, source).unwrap();

        assert!(symbols.iter().any(|s| s.name == "User" && matches!(s.kind, SymbolKind::Type)));
    }

    #[test]
    fn test_extract_type_alias() {
        let source = r#"
type UserId = string | number;
"#;
        let mut parser = TypeScriptParser::new().unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = extract_symbols(&tree, source).unwrap();

        assert!(symbols.iter().any(|s| s.name == "UserId" && matches!(s.kind, SymbolKind::Type)));
    }

    #[test]
    fn test_extract_imports() {
        let source = r#"
import { useState } from 'react';
import axios from 'axios';
"#;
        let mut parser = TypeScriptParser::new().unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = extract_symbols(&tree, source).unwrap();

        assert!(symbols.iter().any(|s| s.name == "react" && matches!(s.kind, SymbolKind::Import)));
        assert!(symbols.iter().any(|s| s.name == "axios" && matches!(s.kind, SymbolKind::Import)));
    }
}
