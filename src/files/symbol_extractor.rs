use crate::files::symbol::Symbol;
use std::path::Path;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Parser, Query, QueryCursor};

fn get_language_key(file_path: &Path) -> Option<&'static str> {
    let ext = file_path.extension()?.to_str()?;
    match ext {
        "rs" => Some("rust"),
        "py" => Some("python"),
        "js" | "mjs" | "cjs" => Some("javascript"),
        "jsx" => Some("javascript_jsx"),
        "ts" | "mts" | "cts" => Some("typescript"),
        "tsx" => Some("typescript_tsx"),
        "go" => Some("go"),
        "html" | "htm" => Some("html"),
        "css" => Some("css"),
        "yaml" | "yml" => Some("yaml"),
        "toml" => Some("toml"),
        "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" | "c" | "h" => Some("cpp"),
        "cu" | "cuh" => Some("cuda"),
        "md" | "markdown" => Some("markdown"),
        "tex" | "sty" | "cls" => Some("latex"),
        _ => None,
    }
}

pub fn get_language(file_path: &Path) -> Option<Language> {
    match get_language_key(file_path)? {
        "rust" => Some(tree_sitter_rust::LANGUAGE.into()),
        "python" => Some(tree_sitter_python::LANGUAGE.into()),
        "javascript" | "javascript_jsx" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "typescript" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "typescript_tsx" => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "html" => Some(tree_sitter_html::LANGUAGE.into()),
        "css" => Some(tree_sitter_css::LANGUAGE.into()),
        "yaml" => Some(tree_sitter_yaml::LANGUAGE.into()),
        "toml" => Some(tree_sitter_toml_ng::LANGUAGE.into()),
        "cpp" => Some(tree_sitter_cpp::LANGUAGE.into()),
        "cuda" => Some(tree_sitter_cuda::LANGUAGE.into()),
        "markdown" => Some(tree_sitter_md::LANGUAGE.into()),
        "latex" => Some(codebook_tree_sitter_latex::LANGUAGE.into()),
        _ => None,
    }
}

/// Comprehensive Rust queries following Zed's outline.scm pattern
const RUST_QUERY: &str = r"
; Functions
(function_item name: (identifier) @name) @item

; Structs
(struct_item name: (type_identifier) @name) @item

; Enums and variants
(enum_item name: (type_identifier) @name) @item
(enum_variant name: (identifier) @name) @item

; Traits
(trait_item name: (type_identifier) @name) @item

; Impl blocks - capture the type being implemented
(impl_item
  trait: (type_identifier)? @trait_name
  type: (type_identifier) @name) @item

; Type aliases
(type_item name: (type_identifier) @name) @item

; Associated types in traits
(associated_type name: (type_identifier) @name) @item

; Constants and statics
(const_item name: (identifier) @name) @item
(static_item name: (identifier) @name) @item

; Modules
(mod_item name: (identifier) @name) @item

; Macros
(macro_definition name: (identifier) @name) @item

; Struct/enum fields (optional - can be noisy)
; (field_declaration name: (field_identifier) @name) @item
";

/// Comprehensive Python queries
const PYTHON_QUERY: &str = r"
; Functions and methods
(function_definition name: (identifier) @name) @item

; Classes
(class_definition name: (identifier) @name) @item

; Decorated definitions
(decorated_definition
  definition: (function_definition name: (identifier) @name)) @item
(decorated_definition
  definition: (class_definition name: (identifier) @name)) @item

; Async functions
(function_definition
  name: (identifier) @name) @item

; Global assignments (module-level constants)
(assignment
  left: (identifier) @name) @item
";

/// Comprehensive JavaScript queries
const JAVASCRIPT_QUERY: &str = r"
; Function declarations
(function_declaration name: (identifier) @name) @item

; Class declarations
(class_declaration name: (identifier) @name) @item

; Method definitions in classes
(method_definition name: (property_identifier) @name) @item

; Arrow functions assigned to variables
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    value: (arrow_function))) @item

(variable_declaration
  (variable_declarator
    name: (identifier) @name
    value: (arrow_function))) @item

; Function expressions assigned to variables
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    value: (function_expression))) @item

(variable_declaration
  (variable_declarator
    name: (identifier) @name
    value: (function_expression))) @item

; Object methods
(pair
  key: (property_identifier) @name
  value: (function_expression)) @item

(pair
  key: (property_identifier) @name
  value: (arrow_function)) @item

; Export declarations
(export_statement
  declaration: (function_declaration name: (identifier) @name)) @item
(export_statement
  declaration: (class_declaration name: (identifier) @name)) @item
";

/// Comprehensive TypeScript queries
const TYPESCRIPT_QUERY: &str = r"
; Function declarations
(function_declaration name: (identifier) @name) @item

; Class declarations
(class_declaration name: (identifier) @name) @item

; Interface declarations
(interface_declaration name: (type_identifier) @name) @item

; Type aliases
(type_alias_declaration name: (type_identifier) @name) @item

; Enum declarations
(enum_declaration name: (identifier) @name) @item

; Method definitions
(method_definition name: (property_identifier) @name) @item

; Method signatures in interfaces
(method_signature name: (property_identifier) @name) @item

; Property signatures in interfaces
(property_signature name: (property_identifier) @name) @item

; Arrow functions assigned to variables
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    value: (arrow_function))) @item

; Function expressions assigned to variables
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    value: (function_expression))) @item

; Namespace declarations
(module name: (identifier) @name) @item

; Abstract class declarations
(abstract_class_declaration name: (identifier) @name) @item

; Export declarations
(export_statement
  declaration: (function_declaration name: (identifier) @name)) @item
(export_statement
  declaration: (class_declaration name: (identifier) @name)) @item
(export_statement
  declaration: (interface_declaration name: (type_identifier) @name)) @item
(export_statement
  declaration: (type_alias_declaration name: (type_identifier) @name)) @item
";

/// Comprehensive Go queries
const GO_QUERY: &str = r"
; Function declarations
(function_declaration name: (identifier) @name) @item

; Method declarations
(method_declaration
  name: (field_identifier) @name) @item

; Type declarations (structs, interfaces, type aliases)
(type_declaration
  (type_spec name: (type_identifier) @name)) @item

; Struct types
(type_declaration
  (type_spec
    name: (type_identifier) @name
    type: (struct_type))) @item

; Interface types
(type_declaration
  (type_spec
    name: (type_identifier) @name
    type: (interface_type))) @item

; Constants
(const_declaration
  (const_spec name: (identifier) @name)) @item

; Variables (package-level)
(var_declaration
  (var_spec name: (identifier) @name)) @item
";

/// HTML queries - elements with id/class attributes
const HTML_QUERY: &str = r#"
; Elements with id attribute
(element
  (start_tag
    (tag_name) @name
    (attribute
      (attribute_name) @attr_name
      (quoted_attribute_value) @attr_value)
    (#eq? @attr_name "id"))) @item

; Script tags
(script_element
  (start_tag (tag_name) @name)) @item

; Style tags
(style_element
  (start_tag (tag_name) @name)) @item
"#;

/// CSS queries
const CSS_QUERY: &str = r#"
; Class selectors
(class_selector (class_name) @name) @item

; ID selectors
(id_selector (id_name) @name) @item

; Tag selectors in rule sets
(rule_set
  (selectors
    (tag_name) @name)) @item

; Keyframes
(keyframes_statement
  name: (keyframes_name) @name) @item

; Media queries
(media_statement) @item

; CSS variables (custom properties)
(declaration
  (property_name) @name
  (#match? @name "^--")) @item
"#;

// Note: HTML_QUERY and CSS_QUERY keep r#""# syntax because they contain special
// characters that would interfere with tree-sitter query parsing

/// YAML queries - top-level keys and anchors
const YAML_QUERY: &str = r"
; Mapping keys
(block_mapping_pair key: (flow_node) @name) @item

; Anchors
(anchor (anchor_name) @name) @item
";

/// TOML queries - tables and keys
const TOML_QUERY: &str = r"
; Tables and array-of-table headers
(table (bare_key) @name) @item
(table (dotted_key) @name) @item
(table_array_element (bare_key) @name) @item
(table_array_element (dotted_key) @name) @item

; Key/value pairs
(pair (bare_key) @name) @item
(pair (dotted_key) @name) @item
";

/// C/C++ queries, shared with CUDA (which extends the C++ grammar)
const CPP_QUERY: &str = r"
; Functions
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @name)) @item
(function_definition
  declarator: (function_declarator
    declarator: (field_identifier) @name)) @item

; Qualified methods, e.g. Foo::bar()
(function_definition
  declarator: (function_declarator
    declarator: (qualified_identifier) @name)) @item

; Classes, structs, unions and enums
(class_specifier name: (type_identifier) @name) @item
(struct_specifier name: (type_identifier) @name) @item
(union_specifier name: (type_identifier) @name) @item
(enum_specifier name: (type_identifier) @name) @item

; Namespaces
(namespace_definition name: (namespace_identifier) @name) @item

; Type aliases and typedefs
(alias_declaration name: (type_identifier) @name) @item
(type_definition declarator: (type_identifier) @name) @item

; Templates carry their declaration inside
(template_declaration
  (class_specifier name: (type_identifier) @name)) @item
(template_declaration
  (function_definition
    declarator: (function_declarator
      declarator: (identifier) @name))) @item
";

/// Markdown queries - headings become the document outline
const MARKDOWN_QUERY: &str = r"
(atx_heading (inline) @name) @item
(setext_heading (paragraph (inline) @name)) @item
";

/// LaTeX queries - sectioning commands and definitions
const LATEX_QUERY: &str = r"
; Sectioning
(part text: (curly_group) @name) @item
(chapter text: (curly_group) @name) @item
(section text: (curly_group) @name) @item
(subsection text: (curly_group) @name) @item
(paragraph text: (curly_group) @name) @item

; Labels and macros
(label_definition name: (curly_group_label) @name) @item
(new_command_definition
  declaration: (curly_group_command_name) @name) @item

; Environments
(generic_environment
  begin: (begin name: (curly_group_text) @name)) @item
";

pub fn get_query(file_path: &Path, language: &Language) -> Option<Query> {
    let lang_key = get_language_key(file_path)?;
    let query_str = match lang_key {
        "rust" => RUST_QUERY,
        "python" => PYTHON_QUERY,
        "javascript" | "javascript_jsx" => JAVASCRIPT_QUERY,
        "typescript" | "typescript_tsx" => TYPESCRIPT_QUERY,
        "go" => GO_QUERY,
        "html" => HTML_QUERY,
        "css" => CSS_QUERY,
        "yaml" => YAML_QUERY,
        "toml" => TOML_QUERY,
        "cpp" | "cuda" => CPP_QUERY,
        "markdown" => MARKDOWN_QUERY,
        "latex" => LATEX_QUERY,
        _ => return None,
    };

    Query::new(language, query_str).ok()
}

pub fn extract_symbols(file_path: &Path, content: &str) -> Vec<Symbol> {
    let Some(language) = get_language(file_path) else {
        return Vec::new();
    };

    let Some(query) = get_query(file_path, &language) else {
        return Vec::new();
    };

    let mut parser = Parser::new();
    if parser.set_language(&language).is_err() {
        return Vec::new();
    }

    let Some(tree) = parser.parse(content, None) else {
        return Vec::new();
    };

    let mut symbols = Vec::new();
    let mut cursor = QueryCursor::new();

    // Get capture indices
    let name_index = query.capture_index_for_name("name");
    let item_index = query.capture_index_for_name("item");

    let Some(name_idx) = name_index else {
        return symbols;
    };

    // Use captures iterator (tree-sitter 0.24+ API with StreamingIterator)
    let mut captures = cursor.captures(&query, tree.root_node(), content.as_bytes());
    while let Some((mat, capture_idx)) = captures.next() {
        let capture = &mat.captures[*capture_idx];

        if capture.index == name_idx {
            let node = capture.node;
            let name = node.utf8_text(content.as_bytes()).unwrap_or("").to_string();

            // Skip empty names
            if name.is_empty() {
                continue;
            }

            // Find the @item node for context (parent/container)
            let item_node = item_index
                .and_then(|idx| mat.captures.iter().find(|c| c.index == idx).map(|c| c.node));

            // Use @item node for position and kind if available, otherwise use parent
            let context_node = item_node.or_else(|| node.parent());
            let kind = context_node.map_or_else(
                || crate::files::SymbolKind::Variable,
                determine_symbol_kind_from_node,
            );

            // Get line from the item node (whole definition) for better positioning
            let position_node = item_node.unwrap_or(node);
            let start_point = position_node.start_position();
            let line = start_point.row;
            symbols.push(Symbol {
                name,
                kind,
                file: file_path.to_path_buf(),
                line,
            });
        }
    }

    symbols
}

fn determine_symbol_kind_from_node(node: tree_sitter::Node) -> crate::files::SymbolKind {
    use crate::files::SymbolKind;

    // Allow same arms - keeping them separate for clarity and easier language-specific changes
    #[allow(clippy::match_same_arms)]
    match node.kind() {
        // Rust
        "function_item" => SymbolKind::Function,
        "struct_item" => SymbolKind::Struct,
        "enum_item" => SymbolKind::Enum,
        "enum_variant" => SymbolKind::Enum,
        "trait_item" => SymbolKind::Trait,
        "impl_item" => SymbolKind::Impl,
        "const_item" => SymbolKind::Const,
        "static_item" => SymbolKind::Static,
        "mod_item" => SymbolKind::Module,
        "type_item" => SymbolKind::Struct, // Type alias
        "associated_type" => SymbolKind::Struct,
        "macro_definition" => SymbolKind::Function, // Macros are function-like

        // Python
        "function_definition" => SymbolKind::Function,
        "class_definition" => SymbolKind::Class,
        "decorated_definition" => SymbolKind::Function,
        "assignment" => SymbolKind::Variable,

        // JavaScript/TypeScript/Go - Functions
        "function_declaration" | "method_declaration" => SymbolKind::Function,
        "class_declaration" => SymbolKind::Class,
        "method_definition" => SymbolKind::Method,
        "interface_declaration" => SymbolKind::Trait,
        "type_alias_declaration" => SymbolKind::Struct,
        "enum_declaration" => SymbolKind::Enum,
        "lexical_declaration" | "variable_declaration" => SymbolKind::Variable,
        "abstract_class_declaration" => SymbolKind::Class,
        "method_signature" | "property_signature" => SymbolKind::Method,
        "export_statement" => SymbolKind::Module,
        "module" => SymbolKind::Module, // TS namespace

        // Go types and declarations
        "type_declaration" | "type_spec" => SymbolKind::Struct,
        "const_declaration" | "const_spec" => SymbolKind::Const,
        "var_declaration" | "var_spec" => SymbolKind::Variable,

        // YAML / TOML
        "block_mapping_pair" => SymbolKind::Variable,
        "anchor" => SymbolKind::Const,
        "table" | "table_array_element" => SymbolKind::Module,
        // Also a JS object entry, which reads as a variable just as well
        "pair" => SymbolKind::Variable,

        // C / C++ / CUDA
        "class_specifier" => SymbolKind::Class,
        "struct_specifier" => SymbolKind::Struct,
        "union_specifier" => SymbolKind::Struct,
        "enum_specifier" => SymbolKind::Enum,
        "namespace_definition" => SymbolKind::Module,
        "alias_declaration" | "type_definition" => SymbolKind::Struct,
        "template_declaration" => SymbolKind::Struct,

        // Markdown
        "atx_heading" | "setext_heading" => SymbolKind::Module,

        // LaTeX
        "part" | "chapter" | "section" | "subsection" => SymbolKind::Module,
        "label_definition" => SymbolKind::Const,
        "new_command_definition" => SymbolKind::Function,
        "generic_environment" => SymbolKind::Struct,

        // CSS
        "class_selector" => SymbolKind::Class,
        "id_selector" => SymbolKind::Variable,
        "rule_set" => SymbolKind::Variable,
        "keyframes_statement" => SymbolKind::Function,
        "media_statement" => SymbolKind::Module,

        // HTML
        "element" | "script_element" | "style_element" => SymbolKind::Module,

        // Generic fallback
        _ => SymbolKind::Variable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_language_rust() {
        let lang = get_language(Path::new("test.rs"));
        assert!(lang.is_some());
    }

    #[test]
    fn test_get_language_python() {
        let lang = get_language(Path::new("test.py"));
        assert!(lang.is_some());
    }

    #[test]
    fn test_get_language_unknown() {
        let lang = get_language(Path::new("test.unknown"));
        assert!(lang.is_none());
    }
}

#[cfg(test)]
mod new_language_tests {
    use super::*;

    fn names(path: &str, source: &str) -> Vec<String> {
        extract_symbols(Path::new(path), source)
            .into_iter()
            .map(|s| s.name)
            .collect()
    }

    #[test]
    fn extracts_yaml_keys() {
        let found = names(
            "c.yaml",
            "name: build\njobs:\n  test:\n    runs-on: linux\n",
        );
        assert!(found.contains(&"name".to_string()), "got {found:?}");
        assert!(found.contains(&"jobs".to_string()), "got {found:?}");
    }

    #[test]
    fn extracts_toml_tables_and_keys() {
        let found = names("c.toml", "[package]\nname = \"viewer\"\n\n[dependencies]\n");
        assert!(found.contains(&"package".to_string()), "got {found:?}");
        assert!(found.contains(&"name".to_string()), "got {found:?}");
    }

    #[test]
    fn extracts_cpp_definitions() {
        let source = "class Widget {\npublic:\n  void draw();\n};\nnamespace ui {}\nint main() { return 0; }\n";
        let found = names("c.cpp", source);
        assert!(found.contains(&"Widget".to_string()), "got {found:?}");
        assert!(found.contains(&"main".to_string()), "got {found:?}");
        assert!(found.contains(&"ui".to_string()), "got {found:?}");
    }

    #[test]
    fn extracts_cuda_kernels() {
        let source = "__global__ void addKernel(int *c) {}\nstruct Params {};\n";
        let found = names("c.cu", source);
        assert!(found.contains(&"addKernel".to_string()), "got {found:?}");
        assert!(found.contains(&"Params".to_string()), "got {found:?}");
    }

    #[test]
    fn extracts_markdown_headings() {
        let found = names("c.md", "# Title\n\nsome text\n\n## Section\n");
        assert!(found.iter().any(|n| n.contains("Title")), "got {found:?}");
        assert!(found.iter().any(|n| n.contains("Section")), "got {found:?}");
    }

    #[test]
    fn extracts_latex_sections() {
        let source = "\\section{Intro}\n\\subsection{Details}\n";
        let found = names("c.tex", source);
        assert!(found.iter().any(|n| n.contains("Intro")), "got {found:?}");
        assert!(found.iter().any(|n| n.contains("Details")), "got {found:?}");
    }
}
