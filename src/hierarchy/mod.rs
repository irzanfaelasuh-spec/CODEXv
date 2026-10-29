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

fn rust_symbols(source: &str) -> Vec<Symbol> {
lexical_symbols(source)
}

fn lexical_symbols(source: &str) -> Vec<Symbol> {
let mut out = Vec::new();

for (index, line) in source.lines().enumerate() {
    let trimmed = line.trim();

    let patterns = [
        ("fn ", "Function"),
        ("pub fn ", "Function"),
        ("async fn ", "Function"),
        ("pub async fn ", "Function"),
        ("def ", "Function"),
        ("function ", "Function"),
        ("class ", "Class"),
        ("struct ", "Struct"),
        ("pub struct ", "Struct"),
        ("enum ", "Enum"),
        ("pub enum ", "Enum"),
        ("trait ", "Trait"),
        ("pub trait ", "Trait"),
        ("mod ", "Module"),
        ("pub mod ", "Module"),
        ("const ", "Constant"),
        ("pub const ", "Constant"),
        ("static ", "Static"),
        ("pub static ", "Static"),
        ("func ", "Function"),
        ("interface ", "Interface"),
    ];

    for (prefix, kind) in patterns {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let name = rest
                .split(|c: char| {
                    c == '('
                        || c == '{'
                        || c == ':'
                        || c == ';'
                        || c.is_whitespace()
                })
                .next()
                .unwrap_or("symbol")
                .trim();

            if !name.is_empty() {
                out.push(Symbol {
                    name: name.to_owned(),
                    kind: kind.to_owned(),
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
