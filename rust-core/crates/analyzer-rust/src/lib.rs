//! Rust code analyzer using Tree-sitter
//!
//! Provides parsing and symbol extraction for Rust source files.

pub mod parser;
pub mod symbol_extract;

pub use parser::RustParser;
pub use symbol_extract::extract_symbols;

use analyzer_core::{Symbol, SymbolKind};
use anyhow::Result;

/// Analyze a Rust source file and extract symbols
pub fn analyze_rust(source: &str) -> Result<Vec<Symbol>> {
    let mut parser = RustParser::new()?;
    let tree = parser.parse(source)?;
    let symbols = extract_symbols(&tree, source)?;
    Ok(symbols)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_simple_function() {
        let source = r#"
fn hello_world() {
    println!("Hello, world!");
}
"#;
        let symbols = analyze_rust(source).unwrap();
        assert!(symbols.iter().any(|s| s.name == "hello_world" && matches!(s.kind, SymbolKind::Function)));
    }

    #[test]
    fn test_analyze_struct() {
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
        let symbols = analyze_rust(source).unwrap();
        assert!(symbols.iter().any(|s| s.name == "MyStruct" && matches!(s.kind, SymbolKind::Class)));
        assert!(symbols.iter().any(|s| s.name == "new" && matches!(s.kind, SymbolKind::Function)));
        assert!(symbols.iter().any(|s| s.name == "get_value" && matches!(s.kind, SymbolKind::Function)));
    }

    #[test]
    fn test_analyze_trait() {
        let source = r#"
trait MyTrait {
    fn do_something(&self);
}
"#;
        let symbols = analyze_rust(source).unwrap();
        assert!(symbols.iter().any(|s| s.name == "MyTrait" && matches!(s.kind, SymbolKind::Type)));
    }
}
