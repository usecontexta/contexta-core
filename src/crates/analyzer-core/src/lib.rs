// Analyzer Core - Core analyzer logic for Contexta MCP
// Handles file indexing, storage, queries, and incremental updates

pub mod indexer;
pub mod storage;
pub mod query;
pub mod incremental;

// Analysis modules
pub mod analysis {
    pub mod deep;
}

use serde::{Deserialize, Serialize};

/// Represents a code symbol (function, class, variable, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Symbol {
    /// Unique identifier
    pub id: Option<i64>,

    /// File ID reference
    pub file_id: i64,

    /// Symbol name
    pub name: String,

    /// Symbol kind (function, class, variable, import, etc.)
    pub kind: SymbolKind,

    /// Start line number (0-indexed)
    pub line_start: usize,

    /// End line number (0-indexed)
    pub line_end: usize,

    /// Parent scope (JSON array of parent symbol IDs)
    pub scope: Option<String>,

    /// Additional metadata (language-specific)
    pub metadata: Option<String>,
}

/// Symbol kind enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SymbolKind {
    Function,
    Class,
    Variable,
    Import,
    Export,
    Module,
    Struct,
    Enum,
    Trait,
    Interface,
    Type,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SymbolKind::Function => "function",
            SymbolKind::Class => "class",
            SymbolKind::Variable => "variable",
            SymbolKind::Import => "import",
            SymbolKind::Export => "export",
            SymbolKind::Module => "module",
            SymbolKind::Struct => "struct",
            SymbolKind::Enum => "enum",
            SymbolKind::Trait => "trait",
            SymbolKind::Interface => "interface",
            SymbolKind::Type => "type",
        };
        write!(f, "{}", s)
    }
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: Option<i64>,
    pub path: String,
    pub language: String,
    pub size: u64,
    pub last_indexed: Option<String>,
    pub parse_errors: i32,
}

/// Language detection based on file extension
pub fn detect_language(path: &str) -> Option<&'static str> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())?;

    match ext {
        "py" | "pyi" => Some("python"),
        "ts" | "tsx" => Some("typescript"),
        "js" | "jsx" => Some("javascript"),
        "rs" => Some("rust"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("test.py"), Some("python"));
        assert_eq!(detect_language("test.ts"), Some("typescript"));
        assert_eq!(detect_language("test.js"), Some("javascript"));
        assert_eq!(detect_language("test.rs"), Some("rust"));
        assert_eq!(detect_language("test.txt"), None);
    }

    #[test]
    fn test_symbol_kind_display() {
        assert_eq!(SymbolKind::Function.to_string(), "function");
        assert_eq!(SymbolKind::Class.to_string(), "class");
    }
}
