//! TypeScript code analyzer using Tree-sitter
//!
//! Provides parsing and symbol extraction for TypeScript source files.

pub mod parser;
pub mod symbol_extract;

pub use parser::TypeScriptParser;
pub use symbol_extract::extract_symbols;

use analyzer_core::{Symbol, SymbolKind};
use anyhow::Result;

/// Analyze a TypeScript source file and extract symbols
pub fn analyze_typescript(source: &str) -> Result<Vec<Symbol>> {
    let mut parser = TypeScriptParser::new()?;
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
function helloWorld() {
    console.log("Hello, world!");
}
"#;
        let symbols = analyze_typescript(source).unwrap();
        assert!(symbols.iter().any(|s| s.name == "helloWorld" && matches!(s.kind, SymbolKind::Function)));
    }

    #[test]
    fn test_analyze_class() {
        let source = r#"
class MyClass {
    private value: number;

    constructor() {
        this.value = 42;
    }

    getValue(): number {
        return this.value;
    }
}
"#;
        let symbols = analyze_typescript(source).unwrap();
        assert!(symbols.iter().any(|s| s.name == "MyClass" && matches!(s.kind, SymbolKind::Class)));
        assert!(symbols.iter().any(|s| s.name == "getValue" && matches!(s.kind, SymbolKind::Function)));
    }

    #[test]
    fn test_analyze_interface() {
        let source = r#"
interface User {
    name: string;
    age: number;
}
"#;
        let symbols = analyze_typescript(source).unwrap();
        assert!(symbols.iter().any(|s| s.name == "User" && matches!(s.kind, SymbolKind::Type)));
    }
}
