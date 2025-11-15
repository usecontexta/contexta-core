//! Rust symbol extraction
//!
//! Extracts functions, structs, enums, traits, and imports from Rust AST.

use analyzer_core::{Symbol, SymbolKind};
use anyhow::Result;
use tree_sitter::{Node, Tree, TreeCursor};

/// Extract symbols from a Rust parse tree
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
        "function_item" => {
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
        "struct_item" => {
            if let Some(symbol) = extract_struct(node, source, parent_scope.as_deref())? {
                symbols.push(symbol);
            }
        }
        "enum_item" => {
            if let Some(symbol) = extract_enum(node, source, parent_scope.as_deref())? {
                symbols.push(symbol);
            }
        }
        "trait_item" => {
            if let Some(symbol) = extract_trait(node, source, parent_scope.as_deref())? {
                symbols.push(symbol);
            }
        }
        "type_item" => {
            if let Some(symbol) = extract_type_alias(node, source, parent_scope.as_deref())? {
                symbols.push(symbol);
            }
        }
        "impl_item" => {
            // Extract methods from impl blocks
            if let Some(impl_scope) = extract_impl_scope(node, source)? {
                if cursor.goto_first_child() {
                    loop {
                        extract_from_node(cursor, source, symbols, Some(impl_scope.clone()), _file_id)?;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
        }
        "use_declaration" => {
            if let Some(symbol) = extract_use(node, source, parent_scope.as_deref())? {
                symbols.push(symbol);
            }
        }
        "const_item" | "static_item" => {
            // Extract constants and static variables (module-level only for now)
            if parent_scope.is_none() {
                if let Some(symbol) = extract_constant(node, source)? {
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
        file_id: 0,
        name,
        kind: SymbolKind::Function,
        line_start,
        line_end,
        scope: scope.map(|s| s.to_string()),
        metadata: None,
    }))
}

/// Extract a struct definition
fn extract_struct(node: Node, source: &str, scope: Option<&str>) -> Result<Option<Symbol>> {
    let name_node = node
        .child_by_field_name("name")
        .ok_or_else(|| anyhow::anyhow!("Struct has no name"))?;

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

/// Extract an enum definition
fn extract_enum(node: Node, source: &str, scope: Option<&str>) -> Result<Option<Symbol>> {
    let name_node = node
        .child_by_field_name("name")
        .ok_or_else(|| anyhow::anyhow!("Enum has no name"))?;

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

/// Extract a trait definition
fn extract_trait(node: Node, source: &str, scope: Option<&str>) -> Result<Option<Symbol>> {
    let name_node = node
        .child_by_field_name("name")
        .ok_or_else(|| anyhow::anyhow!("Trait has no name"))?;

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

/// Extract a type alias
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

/// Extract the scope name from an impl block
fn extract_impl_scope(node: Node, source: &str) -> Result<Option<String>> {
    // Get the type being implemented
    let type_node = node.child_by_field_name("type");

    if let Some(type_node) = type_node {
        let type_name = node_text(type_node, source);
        Ok(Some(type_name))
    } else {
        Ok(None)
    }
}

/// Extract a use declaration
fn extract_use(node: Node, source: &str, scope: Option<&str>) -> Result<Option<Symbol>> {
    // Get the argument (what's being imported)
    let arg = node.child_by_field_name("argument");

    if let Some(arg_node) = arg {
        let name = node_text(arg_node, source);
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

/// Extract a constant or static variable
fn extract_constant(node: Node, source: &str) -> Result<Option<Symbol>> {
    let name_node = node.child_by_field_name("name");

    if let Some(name_node) = name_node {
        let name = node_text(name_node, source);
        let line_start = node.start_position().row;
        let line_end = node.end_position().row;

        Ok(Some(Symbol {
            id: None,
            file_id: 0,
            name,
            kind: SymbolKind::Variable,
            line_start,
            line_end,
            scope: None,
            metadata: None,
        }))
    } else {
        Ok(None)
    }
}

/// Get text content of a node
fn node_text(node: Node, source: &str) -> String {
    source[node.byte_range()].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::RustParser;

    #[test]
    fn test_extract_function() {
        let source = r#"
fn my_function() {
    println!("test");
}
"#;
        let mut parser = RustParser::new().unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = extract_symbols(&tree, source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "my_function");
        assert!(matches!(symbols[0].kind, SymbolKind::Function));
    }

    #[test]
    fn test_extract_struct_with_impl() {
        let source = r#"
struct MyStruct {
    value: i32,
}

impl MyStruct {
    fn new() -> Self {
        Self { value: 42 }
    }

    fn get_value(&self) -> i32 {
        self.value
    }
}
"#;
        let mut parser = RustParser::new().unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = extract_symbols(&tree, source).unwrap();

        assert!(symbols.iter().any(|s| s.name == "MyStruct" && matches!(s.kind, SymbolKind::Class)));
        assert!(symbols.iter().any(|s| s.name == "new" && matches!(s.kind, SymbolKind::Function)));
        assert!(symbols.iter().any(|s| s.name == "get_value" && matches!(s.kind, SymbolKind::Function)));
    }

    #[test]
    fn test_extract_trait() {
        let source = r#"
trait MyTrait {
    fn do_something(&self);
}
"#;
        let mut parser = RustParser::new().unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = extract_symbols(&tree, source).unwrap();

        assert!(symbols.iter().any(|s| s.name == "MyTrait" && matches!(s.kind, SymbolKind::Type)));
    }

    #[test]
    fn test_extract_enum() {
        let source = r#"
enum MyEnum {
    Variant1,
    Variant2(i32),
}
"#;
        let mut parser = RustParser::new().unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = extract_symbols(&tree, source).unwrap();

        assert!(symbols.iter().any(|s| s.name == "MyEnum" && matches!(s.kind, SymbolKind::Type)));
    }

    #[test]
    fn test_extract_use() {
        let source = r#"
use std::collections::HashMap;
use anyhow::Result;
"#;
        let mut parser = RustParser::new().unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = extract_symbols(&tree, source).unwrap();

        assert!(symbols.iter().any(|s| s.name == "std::collections::HashMap" && matches!(s.kind, SymbolKind::Import)));
        assert!(symbols.iter().any(|s| s.name == "anyhow::Result" && matches!(s.kind, SymbolKind::Import)));
    }
}
