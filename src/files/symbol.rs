use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Const,
    Static,
    Module,
    Class,
    Method,
    Variable,
}

impl SymbolKind {
    #[allow(clippy::missing_const_for_fn)]
    pub fn icon(&self) -> &'static str {
        #[allow(clippy::match_same_arms)]
        match self {
            Self::Function | Self::Method => "fn",
            Self::Struct => "st",
            Self::Enum => "en",
            Self::Trait => "tr",
            Self::Impl => "im",
            Self::Const => "co",
            Self::Static => "st", // Same as Struct but semantically different
            Self::Module => "mo",
            Self::Class => "cl",
            Self::Variable => "va",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file: PathBuf,
    pub line: usize,
    #[allow(dead_code)]
    pub column: usize,
    #[allow(dead_code)]
    pub parent: Option<String>,
}

impl Symbol {
    #[allow(dead_code)]
    pub fn display_name(&self) -> String {
        self.parent.as_ref().map_or_else(
            || self.name.clone(),
            |parent| format!("{} in {}", self.name, parent),
        )
    }

    #[allow(dead_code)]
    pub fn display_location(&self) -> String {
        format!(
            "{}:{}",
            self.file.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            self.line
        )
    }
}
