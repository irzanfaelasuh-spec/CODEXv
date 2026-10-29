#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub line: usize,
    pub depth: usize,
}

pub fn symbols(path: &str, source: &str) -> Vec<Symbol> {
    if path.ends_with(".rs") {
        rust_symbols(source)
    } else {
        lexical_symbols(source)
    }
}

// Browser/WASM-friendly Rust symbol extraction.
// This intentionally avoids tree-sitter's native C build dependency so
// VYRON can compile cleanly for wasm32-unknown-unknown on GitHub Pages.
fn rust_symbols(source: &str) -> Vec<Symbol> {
    let mut out = Vec::new();

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        let candidates = [
            ("pub async fn ", "Function"),
            ("pub fn ", "Function"),
            ("async fn ", "Function"),
            ("fn ", "Function"),
            ("pub struct ", "Struct"),
            ("struct ", "Struct"),
            ("pub enum ", "Enum"),
            ("enum ", "Enum"),
            ("pub trait ", "Trait"),
            ("trait ", "Trait"),
            ("pub mod ", "Module"),
            ("mod ", "Module"),
            ("pub const ", "Constant"),
            ("const ", "Constant"),
            ("pub static ", "Static"),
            ("static ", "Static"),
        ];

        for (prefix, kind) in candidates {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                let name = rest
                    .split(|c: char| c == '(' || c == '{' || c == ':' || c == '<' || c.is_whitespace())
                    .next()
                    .unwrap_or("symbol")
                    .trim();

                if !name.is_empty() {
                    out.push(Symbol {
                        name: name.to_owned(),
                        kind: kind.into(),
                        line: index + 1,
                        depth: 0,
                    });
                }
                break;
            }
        }
    }

    out
}

fn lexical_symbols(source: &str) -> Vec<Symbol> {
    let mut out = Vec::new();
    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        let patterns = [
            ("fn ", "Function"), ("def ", "Function"), ("function ", "Function"),
            ("class ", "Class"), ("struct ", "Struct"), ("interface ", "Interface"),
            ("enum ", "Enum"), ("trait ", "Trait"), ("func ", "Function"),
        ];
        for (prefix, kind) in patterns {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                let name = rest
                    .split(|c: char| c == '(' || c == '{' || c == ':' || c.is_whitespace())
                    .next()
                    .unwrap_or("symbol");
                if !name.is_empty() {
                    out.push(Symbol {
                        name: name.to_owned(),
                        kind: kind.into(),
                        line: index + 1,
                        depth: 0,
                    });
                }
                break;
            }
        }
    }
    out
}
