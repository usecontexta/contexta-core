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
    fn test_detect_language_with_pyi() {
        assert_eq!(detect_language("stubs.pyi"), Some("python"));
    }

    #[test]
    fn test_detect_language_with_tsx() {
        assert_eq!(detect_language("component.tsx"), Some("typescript"));
    }

    #[test]
    fn test_detect_language_with_jsx() {
        assert_eq!(detect_language("component.jsx"), Some("javascript"));
    }

    #[test]
    fn test_detect_language_no_extension() {
        assert_eq!(detect_language("Makefile"), None);
    }

    #[test]
    fn test_detect_language_with_path() {
        assert_eq!(detect_language("/path/to/file.py"), Some("python"));
        assert_eq!(detect_language("../relative/path.rs"), Some("rust"));
    }

    #[test]
    fn test_symbol_kind_display() {
        assert_eq!(SymbolKind::Function.to_string(), "function");
        assert_eq!(SymbolKind::Class.to_string(), "class");
        assert_eq!(SymbolKind::Variable.to_string(), "variable");
        assert_eq!(SymbolKind::Import.to_string(), "import");
        assert_eq!(SymbolKind::Export.to_string(), "export");
        assert_eq!(SymbolKind::Module.to_string(), "module");
        assert_eq!(SymbolKind::Struct.to_string(), "struct");
        assert_eq!(SymbolKind::Enum.to_string(), "enum");
        assert_eq!(SymbolKind::Trait.to_string(), "trait");
        assert_eq!(SymbolKind::Interface.to_string(), "interface");
        assert_eq!(SymbolKind::Type.to_string(), "type");
    }

    #[test]
    fn test_symbol_creation() {
        let symbol = Symbol {
            id: Some(1),
            file_id: 42,
            name: "test_function".to_string(),
            kind: SymbolKind::Function,
            line_start: 10,
            line_end: 20,
            scope: Some("module".to_string()),
            metadata: Some(r#"{"returns": "str"}"#.to_string()),
        };

        assert_eq!(symbol.name, "test_function");
        assert_eq!(symbol.kind, SymbolKind::Function);
        assert_eq!(symbol.line_start, 10);
        assert_eq!(symbol.line_end, 20);
    }

    #[test]
    fn test_symbol_clone() {
        let symbol = Symbol {
            id: Some(1),
            file_id: 1,
            name: "original".to_string(),
            kind: SymbolKind::Class,
            line_start: 0,
            line_end: 10,
            scope: None,
            metadata: None,
        };

        let cloned = symbol.clone();
        assert_eq!(symbol, cloned);
    }

    #[test]
    fn test_symbol_kind_serialization() {
        // Test that SymbolKind can be serialized/deserialized
        let kind = SymbolKind::Function;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, r#""function""#);

        let deserialized: SymbolKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, deserialized);
    }

    #[test]
    fn test_symbol_serialization() {
        let symbol = Symbol {
            id: Some(1),
            file_id: 42,
            name: "test".to_string(),
            kind: SymbolKind::Function,
            line_start: 1,
            line_end: 5,
            scope: None,
            metadata: None,
        };

        // Should be able to serialize and deserialize
        let json = serde_json::to_string(&symbol).unwrap();
        let deserialized: Symbol = serde_json::from_str(&json).unwrap();
        assert_eq!(symbol, deserialized);
    }

    #[test]
    fn test_file_metadata_creation() {
        let metadata = FileMetadata {
            id: Some(1),
            path: "/path/to/file.py".to_string(),
            language: "python".to_string(),
            size: 1024,
            last_indexed: Some("2024-01-01T00:00:00Z".to_string()),
            parse_errors: 0,
        };

        assert_eq!(metadata.path, "/path/to/file.py");
        assert_eq!(metadata.language, "python");
        assert_eq!(metadata.size, 1024);
        assert_eq!(metadata.parse_errors, 0);
    }

    #[test]
    fn test_all_symbol_kinds() {
        // Ensure all symbol kinds are constructible and display correctly
        let kinds = vec![
            SymbolKind::Function,
            SymbolKind::Class,
            SymbolKind::Variable,
            SymbolKind::Import,
            SymbolKind::Export,
            SymbolKind::Module,
            SymbolKind::Struct,
            SymbolKind::Enum,
            SymbolKind::Trait,
            SymbolKind::Interface,
            SymbolKind::Type,
        ];

        for kind in kinds {
            let s = kind.to_string();
            assert!(!s.is_empty());
        }
    }

    #[cfg(feature = "deep-mode")]
    #[test]
    fn test_deep_mode_available() {
        use crate::analysis::deep::is_deep_mode_available;
        assert!(is_deep_mode_available());
    }

    #[cfg(not(feature = "deep-mode"))]
    #[test]
    fn test_deep_mode_not_available() {
        use crate::analysis::deep::is_deep_mode_available;
        assert!(!is_deep_mode_available());
    }
}
