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
    let mut parser = tree_sitter::Parser::new();
    let language: tree_sitter::Language = tree_sitter_rust::LANGUAGE.into();
    if parser.set_language(&language).is_err() {
        return lexical_symbols(source);
    }
    let Some(tree) = parser.parse(source, None) else { return lexical_symbols(source); };
    let mut cursor = tree.walk();
    let mut out = Vec::new();
    visit_rust(&mut cursor, &mut out, 0, source.as_bytes());
    out
}

fn visit_rust(cursor: &mut tree_sitter::TreeCursor, out: &mut Vec<Symbol>, depth: usize, source: &[u8]) {
    loop {
        let node = cursor.node();
        let kind = match node.kind() {
            "function_item" => Some("Function"),
            "struct_item" => Some("Struct"),
            "enum_item" => Some("Enum"),
            "trait_item" => Some("Trait"),
            "impl_item" => Some("Impl"),
            "mod_item" => Some("Module"),
            "const_item" => Some("Constant"),
            "static_item" => Some("Static"),
            _ => None,
        };
        if let Some(kind) = kind {
            let name = node.child_by_field_name("name")
                .map(|n| n.utf8_text(source).unwrap_or("symbol").to_owned())
                .unwrap_or_else(|| kind.to_lowercase());
            out.push(Symbol { name, kind: kind.into(), line: node.start_position().row + 1, depth });
        }
        if cursor.goto_first_child() {
            visit_rust(cursor, out, depth + 1, source);
            cursor.goto_parent();
        }
        if !cursor.goto_next_sibling() { break; }
    }
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
                let name = rest.split(|c: char| c == '(' || c == '{' || c == ':' || c.is_whitespace()).next().unwrap_or("symbol");
                if !name.is_empty() { out.push(Symbol { name: name.to_owned(), kind: kind.into(), line: index + 1, depth: 0 }); }
                break;
            }
        }
    }
    out
}
