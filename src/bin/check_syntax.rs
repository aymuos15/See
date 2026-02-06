use syntect::parsing::SyntaxSet;

fn main() {
    let syntax_set = SyntaxSet::load_defaults_newlines();

    // Check if TOML syntax is available
    if let Some(syntax) = syntax_set.find_syntax_by_extension("toml") {
        println!("✓ TOML syntax found: {}", syntax.name);
    } else {
        println!("✗ TOML syntax NOT found");
    }

    // List some available syntaxes
    println!("\nFirst 20 available syntaxes:");
    for (i, syntax) in syntax_set.syntaxes().iter().enumerate() {
        if i < 20 {
            println!(
                "  - {} (extensions: {:?})",
                syntax.name, syntax.file_extensions
            );
        }
    }
    println!("  ... (total: {})", syntax_set.syntaxes().len());
}
