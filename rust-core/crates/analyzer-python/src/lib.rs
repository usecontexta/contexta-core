//! Python code analyzer using Tree-sitter
//!
//! Provides parsing and symbol extraction for Python source files.

pub mod parser;
pub mod symbol_extract;

pub use parser::PythonParser;
pub use symbol_extract::extract_symbols;

use analyzer_core::{Symbol, SymbolKind};
use anyhow::Result;

/// Analyze a Python source file and extract symbols
pub fn analyze_python(source: &str) -> Result<Vec<Symbol>> {
    let mut parser = PythonParser::new()?;
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
def hello_world():
    print("Hello, world!")
"#;
        let symbols = analyze_python(source).unwrap();
        assert!(symbols.iter().any(|s| s.name == "hello_world" && matches!(s.kind, SymbolKind::Function)));
    }

    #[test]
    fn test_analyze_class() {
        let source = r#"
class MyClass:
    def __init__(self):
        self.value = 42

    def get_value(self):
        return self.value
"#;
        let symbols = analyze_python(source).unwrap();
        assert!(symbols.iter().any(|s| s.name == "MyClass" && matches!(s.kind, SymbolKind::Class)));
        assert!(symbols.iter().any(|s| s.name == "__init__" && matches!(s.kind, SymbolKind::Function)));
        assert!(symbols.iter().any(|s| s.name == "get_value" && matches!(s.kind, SymbolKind::Function)));
    }
}
