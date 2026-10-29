use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct OpenFile {
    pub path: String,
    pub text: String,
    pub dirty: bool,
}

#[derive(Default)]
pub struct EditorState {
    pub tabs: Vec<OpenFile>,
    pub active: usize,
    pub find_query: String,
    pub replace_query: String,
    pub show_replace: bool,
    pub goto_line: String,
}

impl EditorState {
    pub fn open(&mut self, path: &str, text: &str) {
        if let Some(index) = self.tabs.iter().position(|t| t.path == path) {
            self.active = index;
            return;
        }
        self.tabs.push(OpenFile { path: path.to_owned(), text: text.to_owned(), dirty: false });
        self.active = self.tabs.len() - 1;
    }

    pub fn close(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.tabs.remove(index);
            if self.active >= self.tabs.len() && !self.tabs.is_empty() { self.active = self.tabs.len() - 1; }
        }
    }

    pub fn active_mut(&mut self) -> Option<&mut OpenFile> { self.tabs.get_mut(self.active) }
    pub fn active(&self) -> Option<&OpenFile> { self.tabs.get(self.active) }

    pub fn save_into(&mut self, workspace: &mut crate::workspace::Workspace) -> Result<(), String> {
        let Some(file) = self.active_mut() else { return Ok(()); };
        workspace.files.insert(file.path.clone(), file.text.clone());
        file.dirty = false;
        Ok(())
    }

    pub fn replace_all(&mut self) -> usize {
        let query = self.find_query.clone();
        if query.is_empty() { return 0; }
        let replacement = self.replace_query.clone();
        let Some(file) = self.active_mut() else { return 0; };
        let count = file.text.matches(&query).count();
        if count > 0 { file.text = file.text.replace(&query, &replacement); file.dirty = true; }
        count
    }

    pub fn line_count(&self) -> usize { self.active().map(|f| f.text.lines().count().max(1)).unwrap_or(1) }
}

pub fn language_for(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "rs" => "Rust", "py" => "Python", "js" => "JavaScript", "ts" => "TypeScript",
        "html" => "HTML", "css" => "CSS", "json" => "JSON", "md" => "Markdown",
        "c" => "C", "cc" | "cpp" | "cxx" => "C++", "java" => "Java", "go" => "Go",
        "sql" => "SQL", "sh" | "bash" => "Shell", _ => "Plain Text",
    }
}

pub fn simple_highlight(text: &str, language: &str) -> egui::text::LayoutJob {
    use egui::text::{LayoutJob, TextFormat};
    let mut job = LayoutJob::default();
    let normal = TextFormat { font_id: egui::FontId::monospace(14.0), color: egui::Color32::from_rgb(220, 224, 230), ..Default::default() };
    let comment = TextFormat { font_id: egui::FontId::monospace(14.0), color: egui::Color32::from_rgb(105, 135, 100), ..Default::default() };
    let string = TextFormat { font_id: egui::FontId::monospace(14.0), color: egui::Color32::from_rgb(210, 173, 112), ..Default::default() };
    let keyword = TextFormat { font_id: egui::FontId::monospace(14.0), color: egui::Color32::from_rgb(110, 183, 239), ..Default::default() };
    let keywords: BTreeMap<&str, bool> = ["fn","let","mut","pub","struct","enum","impl","trait","use","mod","return","if","else","for","while","match","class","def","import","from","const","function","interface","extends","new","async","await","true","false","null","package","func","var","SELECT","FROM","WHERE","INSERT","UPDATE"].into_iter().map(|k|(k,true)).collect();
    let mut token = String::new();
    let flush = |job: &mut LayoutJob, token: &mut String| {
        if token.is_empty() { return; }
        let fmt = if keywords.contains_key(token.as_str()) { keyword.clone() } else { normal.clone() };
        job.append(token, 0.0, fmt);
        token.clear();
    };
    let mut chars = text.chars().peekable();
    let mut in_string = false;
    let mut string_buf = String::new();
    while let Some(ch) = chars.next() {
        if in_string {
            string_buf.push(ch);
            if ch == '"' { in_string = false; job.append(&string_buf, 0.0, string.clone()); string_buf.clear(); }
            continue;
        }
        if ch == '"' {
            flush(&mut job, &mut token); in_string = true; string_buf.push(ch); continue;
        }
        if ch == '/' && chars.peek() == Some(&'/') {
            flush(&mut job, &mut token); let mut c = String::from("//"); chars.next(); while let Some(x)=chars.next(){ c.push(x); if x=='\n'{break;} } job.append(&c,0.0,comment.clone()); continue;
        }
        if ch.is_alphanumeric() || ch == '_' { token.push(ch); } else { flush(&mut job, &mut token); job.append(&ch.to_string(), 0.0, normal.clone()); }
    }
    if !string_buf.is_empty() { job.append(&string_buf, 0.0, string); }
    flush(&mut job, &mut token);
    let _ = language;
    job
}
