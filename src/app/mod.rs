use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use eframe::{App, CreationContext, Storage};
use egui::{Align, Color32, Context, FontId, Key, Layout, RichText, TextEdit, TextStyle};

use crate::{auth::AuthStore, editor::{language_for, simple_highlight, EditorState}, execution::{ExecutionState, RunState}, hierarchy::{symbols, Symbol}, settings::{Settings, Theme}, ui::{self, BORDER, MINT, MUTED, TEXT}, workspace::Workspace};

const AUTH_KEY: &str = "vyron.auth";
const SETTINGS_KEY: &str = "vyron.settings";
const WORKSPACE_KEY: &str = "vyron.workspace";
const BOOT_MS: u64 = 4000;

#[derive(Clone, Copy, PartialEq, Eq)]
enum View { Explorer, Search, SourceControl, Run, Hierarchy, Settings }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen { Boot, Auth, Welcome, Ide }

pub struct VyronApp {
    screen: Screen,
    boot_started: Instant,
    auth: AuthStore,
    settings: Settings,
    workspace: Workspace,
    editor: EditorState,
    execution: ExecutionState,
    current_symbols: Vec<Symbol>,
    view: View,
    show_output: bool,
    show_terminal: bool,
    show_command_palette: bool,
    show_search: bool,
    search_query: String,
    login_name: String,
    login_password: String,
    auth_error: Option<String>,
    auth_shake: Option<Instant>,
    auth_mode_signup: bool,
    welcome_started: Option<Instant>,
    toast: Option<(String, Instant)>,
    new_file_name: String,
    rename_name: String,
    show_new_file: bool,
    show_rename: bool,
    terminal_input: String,
    terminal_output: Vec<String>,
    command_query: String,
    command_index: usize,
    last_saved_snapshot: Workspace,
    upload_requested: bool,
    uploaded_notice: Option<String>,
    pending_cursor: Option<usize>,
}

impl VyronApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let auth: AuthStore = cc.storage.and_then(|s| eframe::get_value(s, AUTH_KEY)).unwrap_or_default();
        let settings: Settings = cc.storage.and_then(|s| eframe::get_value(s, SETTINGS_KEY)).unwrap_or_default();
        let workspace: Workspace = cc.storage.and_then(|s| eframe::get_value(s, WORKSPACE_KEY)).unwrap_or_else(Workspace::demo);
        settings.apply_visuals(&cc.egui_ctx);
        let mut app = Self {
            screen: Screen::Boot,
            boot_started: Instant::now(),
            auth,
            settings,
            last_saved_snapshot: workspace.clone(),
            workspace,
            editor: EditorState::default(),
            execution: ExecutionState::default(),
            current_symbols: Vec::new(),
            view: View::Explorer,
            show_output: true,
            show_terminal: false,
            show_command_palette: false,
            show_search: false,
            search_query: String::new(),
            login_name: String::new(),
            login_password: String::new(),
            auth_error: None,
            auth_shake: None,
            auth_mode_signup: false,
            welcome_started: None,
            toast: None,
            new_file_name: String::new(),
            rename_name: String::new(),
            show_new_file: false,
            show_rename: false,
            terminal_input: String::new(),
            terminal_output: vec!["VYRON Web Console — type `help` for commands.".into()],
            command_query: String::new(),
            command_index: 0,
            upload_requested: false,
            uploaded_notice: None,
            pending_cursor: None,
        };
        if let Some(path) = app.workspace.file_names().first().cloned() {
            if let Some(text) = app.workspace.files.get(&path).cloned() { app.editor.open(&path, &text); app.refresh_symbols(); }
        }
        app
    }

    fn persist(&self, storage: &mut dyn Storage) {
        eframe::set_value(storage, AUTH_KEY, &self.auth);
        eframe::set_value(storage, SETTINGS_KEY, &self.settings);
        eframe::set_value(storage, WORKSPACE_KEY, &self.workspace);
    }

    fn refresh_symbols(&mut self) {
        if let Some(file) = self.editor.active() { self.current_symbols = symbols(&file.path, &file.text); }
    }

    fn save_current(&mut self) {
        if self.editor.save_into(&mut self.workspace).is_ok() {
            self.last_saved_snapshot = self.workspace.clone();
            self.toast = Some(("Saved".into(), Instant::now()));
        }
        self.refresh_symbols();
    }

    fn open_file(&mut self, path: &str) {
        if let Some(text) = self.workspace.files.get(path).cloned() {
            self.editor.open(path, &text);
            self.refresh_symbols();
        }
    }

    fn open_upload(&mut self) {
        self.upload_requested = true;
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(file) = rfd::FileDialog::new().pick_file() {
                if let Ok(content) = std::fs::read_to_string(&file) {
                    let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("uploaded.txt").to_owned();
                    self.workspace.files.insert(name.clone(), content);
                    self.open_file(&name);
                    self.toast = Some((format!("Uploaded {name}"), Instant::now()));
                }
            }
            self.upload_requested = false;
        }
        #[cfg(target_arch = "wasm32")]
        crate::web::pick_file();
    }

    fn poll_upload(&mut self) {
        #[cfg(target_arch = "wasm32")]
        if let Some((name, content)) = crate::web::take_uploaded_file() {
            self.workspace.files.insert(name.clone(), content);
            self.open_file(&name);
            self.uploaded_notice = Some(name);
            self.upload_requested = false;
        }
    }

    fn auth_ui(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(15, 17, 20));
            ui.with_layout(Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                ui.set_max_width(420.0);
                ui.vertical_centered(|ui| {
                    ui.add_space(60.0);
                    ui.label(RichText::new("VYRON").size(38.0).strong().color(MINT));
                    ui.label(RichText::new("developer workspace").size(13.0).color(MUTED));
                    ui.add_space(30.0);
                    ui.label(RichText::new(if self.auth_mode_signup { "Create your VYRON account" } else { "Sign in to continue" }).size(20.0).strong().color(TEXT));
                    ui.add_space(18.0);
                    ui.label(RichText::new("Name").size(12.0).color(MUTED));
                    ui.add(TextEdit::singleline(&mut self.login_name).desired_width(360.0).hint_text("Your unique name"));
                    ui.add_space(8.0);
                    ui.label(RichText::new("Password").size(12.0).color(MUTED));
                    ui.add(TextEdit::singleline(&mut self.login_password).password(true).desired_width(360.0).hint_text("At least 6 characters"));
                    if let Some(error) = &self.auth_error {
                        ui.add_space(10.0);
                        let elapsed = self.auth_shake.map(|t| t.elapsed().as_secs_f32()).unwrap_or(1.0);
                        let shift = if elapsed < 0.55 { (elapsed * 48.0).sin() * (1.0 - elapsed / 0.55) * 9.0 } else { 0.0 };
                        ui.horizontal(|ui| {
                            ui.add_space(shift.abs());
                            let frame = egui::Frame::new().fill(Color32::from_rgb(60, 26, 31)).stroke(egui::Stroke::new(1.0, Color32::from_rgb(180, 65, 75))).corner_radius(8.0).inner_margin(10.0);
                            frame.show(ui, |ui| { ui.label(RichText::new(error).color(Color32::from_rgb(255, 145, 150))); });
                        });
                        if elapsed < 0.55 { ctx.request_repaint_after(Duration::from_millis(16)); }
                    }
                    ui.add_space(16.0);
                    let label = if self.auth_mode_signup { "Create account" } else { "Enter" };
                    if ui.add_sized([360.0, 42.0], egui::Button::new(RichText::new(label).strong().color(Color32::from_rgb(10,20,16))).fill(MINT)).clicked() || ui.input(|i| i.key_pressed(Key::Enter)) {
                        if self.auth_mode_signup { match self.auth.sign_up(&self.login_name, &self.login_password) { Ok(()) => self.enter_welcome(), Err(e) => { self.auth_error = Some(e); self.auth_shake = Some(Instant::now()); } } }
                        else { match self.auth.sign_in(&self.login_name, &self.login_password) { Ok(()) => self.enter_welcome(), Err(e) => { self.auth_error = Some(e); self.auth_shake = Some(Instant::now()); } } }
                    }
                    ui.add_space(12.0);
                    let switch = if self.auth_mode_signup { "Already have an account? Sign in" } else { "New here? Create an account" };
                    if ui.link(RichText::new(switch).color(MINT)).clicked() { self.auth_mode_signup = !self.auth_mode_signup; self.auth_error = None; self.auth_shake = None; }
                    ui.add_space(20.0);
                    ui.label(RichText::new("Local datastore mode • data stays in this browser").size(11.0).color(MUTED));
                });
            });
        });
    }

    fn enter_welcome(&mut self) { self.login_password.clear(); self.auth_error = None; self.screen = Screen::Welcome; self.welcome_started = Some(Instant::now()); }

    fn splash_ui(&self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.painter().rect_filled(ui.max_rect(), 0.0, Color32::from_rgb(15, 17, 20));
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.34);
                let pulse = if self.settings.animations { ((self.boot_started.elapsed().as_secs_f32() * 3.0).sin() * 0.5 + 0.5) } else { 0.5 };
                let size = 72.0 + pulse * 6.0;
                ui.label(RichText::new("V").size(size).strong().color(MINT));
                ui.label(RichText::new("VYRON IDE").size(28.0).strong().color(TEXT));
                ui.label(RichText::new("build • edit • ship").size(12.0).color(MUTED));
                ui.add_space(20.0);
                ui.add(egui::ProgressBar::new((self.boot_started.elapsed().as_millis() as f32 / BOOT_MS as f32).min(1.0)).desired_width(220.0).animate(self.settings.animations));
            });
        });
    }

    fn welcome_ui(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.painter().rect_filled(ui.max_rect(), 0.0, Color32::from_rgb(15,17,20));
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height()*0.36);
                let name = self.auth.active_user.clone().unwrap_or_else(|| "Developer".into());
                ui.label(RichText::new(format!("Welcome {name}")).size(34.0).strong().color(TEXT));
                ui.label(RichText::new("Your workspace is ready.").size(14.0).color(MUTED));
                ui.add_space(22.0);
                ui.add(egui::Spinner::new().size(24.0));
            });
        });
    }

    fn title_bar(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("titlebar").exact_height(48.0).show_inside(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(10.0);
                ui.label(RichText::new("V").strong().size(19.0).color(MINT));
                ui.label(RichText::new("VYRON").strong().size(13.0).color(TEXT));
                ui.separator();
                ui.label(RichText::new(self.workspace.name.clone()).size(12.0).color(MUTED));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Logout").clicked() { self.auth.logout(); self.screen = Screen::Auth; }
                    if ui.button("⌘").on_hover_text("Command palette • Ctrl/Cmd+Shift+P").clicked() { self.show_command_palette = true; }
                    if ui.button("Upload").clicked() { self.open_upload(); }
                    if ui.button("Save").clicked() { self.save_current(); }
                });
            });
        });
    }

    fn activity_bar(&mut self, ui: &mut egui::Ui) {
        egui::SidePanel::left("activity").exact_width(52.0).resizable(false).show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                for (view, glyph, tip) in [(View::Explorer,"▤","Explorer"),(View::Search,"⌕","Search"),(View::SourceControl,"⑂","Source control"),(View::Run,"▷","Run & output"),(View::Hierarchy,"⌘","Code hierarchy"),(View::Settings,"⚙","Settings")] {
                    if ui::icon_button(ui, glyph, tip, self.view == view).clicked() { self.view = view; if view == View::Search { self.show_search = true; } }
                    ui.add_space(2.0);
                }
                ui.with_layout(Layout::bottom_up(Align::Center), |ui| { ui.label(RichText::new("0.1").size(9.0).color(MUTED)); });
            });
        });
    }

    fn explorer_panel(&mut self, ui: &mut egui::Ui) {
        ui::section_title(ui, "Explorer", Some(self.workspace.files.len()));
        ui.horizontal(|ui| {
            if ui.small_button("＋ File").clicked() { self.show_new_file = true; }
            if ui.small_button("Upload").clicked() { self.open_upload(); }
        });
        ui.add_space(5.0);
        let files = self.workspace.file_names();
        for path in files {
            let active = self.editor.active().map(|f| f.path == path).unwrap_or(false);
            let label = format!("{}  {}", file_glyph(&path), path);
            let response = ui.selectable_label(active, RichText::new(label).size(12.0));
            if response.clicked() { self.open_file(&path); }
            response.context_menu(|ui| {
                if ui.button("Rename").clicked() { self.rename_name = path.clone(); self.show_rename = true; ui.close(); }
                if ui.button("Delete").clicked() { self.workspace.remove_file(&path); ui.close(); }
            });
        }
        if let Some(name) = &self.uploaded_notice { ui.label(RichText::new(format!("Uploaded: {name}")).size(10.0).color(MINT)); }
    }

    fn search_panel(&mut self, ui: &mut egui::Ui) {
        ui::section_title(ui, "Search", None);
        ui.add(TextEdit::singleline(&mut self.search_query).hint_text("Search workspace…"));
        ui.add_space(8.0);
        if self.search_query.is_empty() { ui.label(RichText::new("Type a query to search all open workspace files.").size(11.0).color(MUTED)); return; }
        let query = self.search_query.to_lowercase();
        let results: Vec<(String, Vec<(usize, String)>)> = self.workspace.files.iter().filter_map(|(path, text)| {
            let matches: Vec<_> = text.lines().enumerate().filter(|(_, line)| line.to_lowercase().contains(&query)).take(30).map(|(line, content)| (line, content.to_owned())).collect();
            if matches.is_empty() { None } else { Some((path.clone(), matches)) }
        }).collect();
        for (path, matches) in results {
            ui.collapsing(&path, |ui| {
                for (line, content) in matches { if ui.selectable_label(false, format!("{}  {}", line+1, content.trim())).clicked() { self.open_file(&path); } }
            });
        }
    }

    fn hierarchy_panel(&mut self, ui: &mut egui::Ui) {
        ui::section_title(ui, "Outline", Some(self.current_symbols.len()));
        if self.current_symbols.is_empty() { ui.label(RichText::new("No parsed symbols in this file.").size(11.0).color(MUTED)); }
        for sym in self.current_symbols.clone() {
            ui.horizontal(|ui| {
                ui.add_space(8.0 + sym.depth as f32 * 12.0);
                if ui.selectable_label(false, format!("{}  {}", symbol_glyph(&sym.kind), sym.name)).clicked() {
                    if let Some(file) = self.editor.active() {
                        let target = file.text.lines().take(sym.line.saturating_sub(1)).map(|line| line.chars().count() + 1).sum::<usize>();
                        self.pending_cursor = Some(target);
                    }
                    self.toast = Some((format!("Go to {}:{}", self.editor.active().map(|f| f.path.clone()).unwrap_or_default(), sym.line), Instant::now()));
                }
                ui.label(RichText::new(sym.kind).size(9.0).color(MUTED));
            });
        }
    }

    fn editor_area(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("tabs").exact_height(38.0).show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                for i in 0..self.editor.tabs.len() {
                    let (path, dirty) = { let t=&self.editor.tabs[i]; (t.path.clone(), t.dirty) };
                    let title = format!("{} {}{}", file_glyph(&path), path.rsplit('/').next().unwrap_or(&path), if dirty { " •" } else { "" });
                    let response = ui.selectable_label(self.editor.active == i, RichText::new(title).size(11.0));
                    if response.clicked() { self.editor.active = i; self.refresh_symbols(); }
                    if response.double_clicked() { self.editor.close(i); break; }
                }
            });
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if self.editor.tabs.is_empty() { ui.centered_and_justified(|ui| ui.label(RichText::new("Open a file from Explorer").color(MUTED))); return; }
            let active_index = self.editor.active;
            let path = self.editor.tabs[active_index].path.clone();
            let language = language_for(&path).to_owned();
            let line_count = self.editor.line_count();
            let mut changed = false;
            let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
                let mut job = simple_highlight(buf.as_str(), &language);
                job.wrap.max_width = if self.settings.word_wrap { wrap_width } else { f32::INFINITY };
                ui.fonts_mut(|f| f.layout_job(job))
            };
            ui.horizontal(|ui| {
                let available = ui.available_size();
                if self.settings.line_numbers && available.x > 600.0 {
                    egui::ScrollArea::vertical().id_salt("line_numbers").show(ui, |ui| {
                        for n in 1..=line_count { ui.label(RichText::new(format!("{:>4}", n)).font(FontId::monospace(self.settings.font_size)).color(Color32::from_rgb(90,98,109))); }
                    });
                }
                let edit = TextEdit::multiline(&mut self.editor.tabs[active_index].text)
                    .font(FontId::monospace(self.settings.font_size))
                    .desired_width(if self.settings.word_wrap { available.x } else { f32::INFINITY })
                    .desired_rows(24)
                    .lock_focus(true)
                    .layouter(&mut layouter);
                let editor_width = if self.settings.minimap { (available.x - 86.0).max(120.0) } else { available.x };
                let response = ui.add_sized([editor_width, available.y], edit);
                if let Some(char_index) = self.pending_cursor.take() {
                    let mut state = egui::TextEdit::load_state(ui.ctx(), response.id).unwrap_or_default();
                    state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(char_index))));
                    state.store(ui.ctx(), response.id);
                    ui.scroll_to_cursor(Some(Align::Center));
                }
                changed = response.changed();
                if self.settings.minimap {
                    ui.separator();
                    egui::Frame::new().fill(Color32::from_rgb(20,22,26)).inner_margin(4.0).show(ui, |ui| {
                        let preview = self.editor.tabs[active_index].text.lines().take(120).collect::<Vec<_>>();
                        for line in preview {
                            let compact: String = line.chars().take(18).collect();
                            ui.label(RichText::new(compact.replace('\t', "  ")).font(FontId::monospace(5.0)).color(Color32::from_rgb(72,80,90)));
                        }
                    });
                }
            });
            if changed {
                let tab_spaces = " ".repeat(self.settings.tab_size);
                if tab_spaces.len() > 1 { self.editor.tabs[active_index].text = self.editor.tabs[active_index].text.replace('\t', &tab_spaces); }
                self.editor.tabs[active_index].dirty = true;
                self.refresh_symbols();
            }
        });
    }

    fn bottom_panel(&mut self, ui: &mut egui::Ui) {
        if !self.show_output && !self.show_terminal { return; }
        egui::TopBottomPanel::bottom("bottom").resizable(true).default_height(if narrow { 150.0 } else { 210.0 }).show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(self.show_output, "OUTPUT").clicked() { self.show_output = true; self.show_terminal = false; }
                if ui.selectable_label(self.show_terminal, "TERMINAL").clicked() { self.show_terminal = true; self.show_output = false; }
                ui.separator();
                if ui.small_button("Clear").clicked() { self.execution.clear(); self.terminal_output.clear(); }
                if self.execution.state == RunState::Running && ui.small_button("Stop").clicked() { self.execution.stop(); }
            });
            ui.separator();
            if self.show_output {
                ui.horizontal(|ui| { ui.label(RichText::new(format!("{:?}", self.execution.state)).strong().color(match self.execution.state { RunState::Failed=>Color32::from_rgb(245,100,110), RunState::Completed=>MINT, _=>TEXT })); if let Some(code)=self.execution.exit_code { ui.label(RichText::new(format!("exit {code}")).color(MUTED)); } });
                egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| { ui.monospace(&self.execution.output); if !self.execution.error.is_empty() { ui.label(RichText::new(&self.execution.error).color(Color32::from_rgb(245,120,125))); } });
            } else {
                egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| { for line in &self.terminal_output { ui.monospace(line); } });
                let response = ui.add(TextEdit::singleline(&mut self.terminal_input).hint_text("web-console> help"));
                if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) { let cmd=self.terminal_input.trim().to_owned(); self.terminal_input.clear(); self.run_terminal_command(&cmd); }
            }
        });
    }

    fn run_terminal_command(&mut self, cmd: &str) {
        if cmd.is_empty() { return; }
        self.terminal_output.push(format!("web-console> {cmd}"));
        let mut parts = cmd.split_whitespace();
        match parts.next().unwrap_or("") {
            "help" => self.terminal_output.push("Commands: help, ls, pwd, cat <file>, find <text>, clear, whoami".into()),
            "pwd" => self.terminal_output.push("/workspace".into()),
            "ls" => self.terminal_output.push(self.workspace.file_names().join("  ")),
            "whoami" => self.terminal_output.push(self.auth.active_user.clone().unwrap_or_else(|| "guest".into())),
            "cat" => { if let Some(path)=parts.next() { self.terminal_output.push(self.workspace.files.get(path).cloned().unwrap_or_else(|| "file not found".into())); } else { self.terminal_output.push("usage: cat <file>".into()); } }
            "find" => { let q=parts.collect::<Vec<_>>().join(" ").to_lowercase(); for (p,t) in &self.workspace.files { if t.to_lowercase().contains(&q) { self.terminal_output.push(p.clone()); } } }
            "clear" => self.terminal_output.clear(),
            _ => self.terminal_output.push("Command not available in browser console. The native VYRON build provides OS process execution.".into()),
        }
    }

    fn settings_panel(&mut self, ui: &mut egui::Ui) {
        ui::section_title(ui, "Settings", None);
        ui.label(RichText::new("Appearance").strong().color(TEXT));
        egui::ComboBox::from_label("Theme").selected_text(format!("{:?}", self.settings.theme)).show_ui(ui, |ui| { ui.selectable_value(&mut self.settings.theme, Theme::Graphite, "Graphite"); ui.selectable_value(&mut self.settings.theme, Theme::Midnight, "Midnight"); ui.selectable_value(&mut self.settings.theme, Theme::Light, "Light"); });
        ui.add(egui::Slider::new(&mut self.settings.font_size, 11.0..=22.0).text("Editor font"));
        ui.add(egui::Slider::new(&mut self.settings.tab_size, 2..=8).text("Tab size"));
        ui.checkbox(&mut self.settings.word_wrap, "Word wrap");
        ui.checkbox(&mut self.settings.line_numbers, "Line numbers");
        ui.checkbox(&mut self.settings.minimap, "Minimap");
        ui.checkbox(&mut self.settings.animations, "Smooth animations");
        ui.checkbox(&mut self.settings.autosave, "Autosave");
        self.settings.apply_visuals(ui.ctx());
    }

    fn command_palette(&mut self, ctx: &Context) {
        if !self.show_command_palette { return; }
        egui::Window::new("Command Palette").collapsible(false).resizable(false).default_width(520.0).anchor(egui::Align2::CENTER_TOP, [0.0, 90.0]).show(ctx, |ui| {
            ui.add(TextEdit::singleline(&mut self.command_query).hint_text("Type a command…"));
            let q=self.command_query.to_lowercase();
            let commands=["Open Explorer","Search workspace","Save file","Save all","Run current file","Stop process","Upload file","Toggle terminal","Change theme","Logout"];
            let mut shown=0;
            for cmd in commands.iter().filter(|c| c.to_lowercase().contains(&q)) {
                let selected=shown==self.command_index;
                if ui.selectable_label(selected,*cmd).clicked() { self.execute_command(cmd); self.show_command_palette=false; }
                shown+=1;
            }
            ui.horizontal(|ui| { ui.label(RichText::new("Enter execute • Esc close").size(10.0).color(MUTED)); });
            if ui.input(|i| i.key_pressed(Key::Escape)) { self.show_command_palette=false; }
            if ui.input(|i| i.key_pressed(Key::Enter)) { if let Some(cmd)=commands.iter().find(|c| c.to_lowercase().contains(&q)) { self.execute_command(cmd); self.show_command_palette=false; } }
        });
    }

    fn execute_command(&mut self, cmd: &str) {
        match cmd { "Open Explorer"=>self.view=View::Explorer, "Search workspace"=>self.view=View::Search, "Save file"=>self.save_current(), "Save all"=>{ let _=self.editor.save_into(&mut self.workspace); }, "Run current file"=>self.run_current(), "Stop process"=>self.execution.stop(), "Upload file"=>self.open_upload(), "Toggle terminal"=>{self.show_terminal=true;self.show_output=false;}, "Change theme"=>{self.settings.theme=match self.settings.theme{Theme::Graphite=>Theme::Midnight,Theme::Midnight=>Theme::Light,Theme::Light=>Theme::Graphite};}, "Logout"=>{self.auth.logout();self.screen=Screen::Auth;}, _=>{} }
    }

    fn run_current(&mut self) {
        self.save_current();
        let Some(file)=self.editor.active().cloned() else { return; };
        #[cfg(not(target_arch="wasm32"))] self.execution.run_native(&file.path, &file.text);
        #[cfg(target_arch="wasm32")] self.execution.run_web_preview(&file.path, &file.text);
        self.show_output=true; self.show_terminal=false;
    }

    fn ide_ui(&mut self, ctx: &Context) {
        self.poll_upload();
        let narrow = ctx.screen_rect().width() < 760.0;
        egui::TopBottomPanel::top("topbar").exact_height(48.0).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(10.0);
                ui.label(RichText::new("V").strong().size(19.0).color(MINT));
                ui.label(RichText::new("VYRON").strong().size(13.0).color(TEXT));
                ui.separator();
                ui.label(RichText::new(self.workspace.name.clone()).size(12.0).color(MUTED));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Logout").clicked() { self.auth.logout(); self.screen = Screen::Auth; }
                    if ui.button("⌘").on_hover_text("Command palette • Ctrl/Cmd+Shift+P").clicked() { self.show_command_palette = true; }
                    if ui.button("Upload").clicked() { self.open_upload(); }
                    if ui.button("Run").clicked() { self.run_current(); }
                    if ui.button("Save").clicked() { self.save_current(); }
                });
            });
        });
        egui::SidePanel::left("activity2").exact_width(52.0).resizable(false).show(ctx, |ui| self.activity_bar(ui));
        if !narrow { egui::SidePanel::left("sidebar").default_width(260.0).resizable(true).show(ctx, |ui| match self.view {
            View::Explorer => self.explorer_panel(ui),
            View::Search => self.search_panel(ui),
            View::Hierarchy => self.hierarchy_panel(ui),
            View::Settings => self.settings_panel(ui),
            View::SourceControl => {
                ui::section_title(ui, "Source Control", None);
                ui.label(RichText::new("The native build can run Git commands. GitHub Pages cannot execute local Git processes.\n\nWorkspace changes are still tracked in the editor and persisted locally.").size(11.0).color(MUTED));
            }
            View::Run => {
                ui::section_title(ui, "Run & Debug", None);
                ui.label(RichText::new("Run current file, inspect output, stop execution, and use the native runner for OS processes.").size(11.0).color(MUTED));
                if ui.button("Run current file").clicked() { self.run_current(); }
                if ui.button("Stop").clicked() { self.execution.stop(); }
            }
        }); }
        egui::TopBottomPanel::bottom("output").resizable(true).default_height(if narrow { 150.0 } else { 210.0 }).show(ctx, |ui| {
            if self.show_output || self.show_terminal { self.bottom_panel_contents(ui); }
        });
        egui::TopBottomPanel::bottom("status").exact_height(24.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Ln 1  •  {}", self.editor.active().map(|f| language_for(&f.path)).unwrap_or("No file"))).size(10.0).color(MUTED));
                ui.separator();
                ui.label(RichText::new(format!("{} files", self.workspace.files.len())).size(10.0).color(MUTED));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| { ui.label(RichText::new("VYRON 0.1.0").size(10.0).color(MUTED)); });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| self.editor_area(ui));
        self.command_palette(ctx);
        if self.show_new_file {
            egui::Window::new("New file").collapsible(false).resizable(false).show(ctx, |ui| {
                ui.add(TextEdit::singleline(&mut self.new_file_name).hint_text("src/new_file.rs"));
                if ui.button("Create").clicked() {
                    let name = self.new_file_name.trim().to_owned();
                    if !name.is_empty() && self.workspace.add_file(name.clone(), String::new()).is_ok() { self.open_file(&name); self.show_new_file = false; self.new_file_name.clear(); }
                }
            });
        }
        if self.show_rename {
            egui::Window::new("Rename").collapsible(false).resizable(false).show(ctx, |ui| {
                ui.add(TextEdit::singleline(&mut self.rename_name));
                if ui.button("Rename").clicked() {
                    if let Some(old) = self.editor.active().map(|f| f.path.clone()) {
                        if self.workspace.rename_file(&old, self.rename_name.clone()).is_ok() {
                            if let Some(tab) = self.editor.active_mut() { tab.path = self.rename_name.clone(); }
                            self.show_rename = false;
                            self.refresh_symbols();
                        }
                    }
                }
            });
        }
        if let Some((message, at)) = &self.toast {
            if at.elapsed() < Duration::from_secs(2) {
                egui::Area::new("toast".into()).anchor(egui::Align2::RIGHT_BOTTOM, [-18.0, -36.0]).show(ctx, |ui| { ui.label(RichText::new(message).strong().color(MINT)); });
                ctx.request_repaint_after(Duration::from_millis(100));
            }
        }
    }

    fn bottom_panel_contents(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.selectable_label(self.show_output, "OUTPUT").clicked() { self.show_output = true; self.show_terminal = false; }
            if ui.selectable_label(self.show_terminal, "TERMINAL").clicked() { self.show_terminal = true; self.show_output = false; }
            ui.separator();
            if ui.small_button("Clear").clicked() { self.execution.clear(); self.terminal_output.clear(); }
            if self.execution.state == RunState::Running && ui.small_button("Stop").clicked() { self.execution.stop(); }
        });
        ui.separator();
        if self.show_output {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{:?}", self.execution.state)).strong().color(match self.execution.state { RunState::Failed => Color32::from_rgb(245,100,110), RunState::Completed => MINT, _ => TEXT }));
                if let Some(code) = self.execution.exit_code { ui.label(RichText::new(format!("exit {code}")).color(MUTED)); }
            });
            egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| { ui.monospace(&self.execution.output); if !self.execution.error.is_empty() { ui.label(RichText::new(&self.execution.error).color(Color32::from_rgb(245,120,125))); } });
        } else {
            egui::ScrollArea::vertical().stick_to_bottom(true).max_height(120.0).show(ui, |ui| { for line in &self.terminal_output { ui.monospace(line); } });
            let response = ui.add(TextEdit::singleline(&mut self.terminal_input).hint_text("web-console> help"));
            if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) { let cmd = self.terminal_input.trim().to_owned(); self.terminal_input.clear(); self.run_terminal_command(&cmd); }
        }
    }

}

impl App for VyronApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.settings.apply_visuals(ctx);
        #[cfg(not(target_arch = "wasm32"))]
        self.execution.poll();
        if self.screen == Screen::Boot {
            self.splash_ui(ctx);
            if self.boot_started.elapsed() >= Duration::from_millis(BOOT_MS) { self.screen = if self.auth.active_user.is_some(){Screen::Ide}else{Screen::Auth}; }
            ctx.request_repaint_after(Duration::from_millis(16)); return;
        }
        match self.screen { Screen::Auth=>self.auth_ui(ctx), Screen::Welcome=>{self.welcome_ui(ctx); if self.welcome_started.map(|t|t.elapsed()>=Duration::from_millis(900)).unwrap_or(false){self.screen=Screen::Ide;} ctx.request_repaint_after(Duration::from_millis(16));}, Screen::Ide=>self.ide_ui(ctx), Screen::Boot=>{} }
        if ctx.input(|i| i.modifiers.command || i.modifiers.ctrl) && ctx.input(|i| i.key_pressed(Key::S)) { self.save_current(); }
        if ctx.input(|i| (i.modifiers.command || i.modifiers.ctrl) && i.modifiers.shift && i.key_pressed(Key::P)) { self.show_command_palette=true; }
        if ctx.input(|i| i.key_pressed(Key::F5)) { self.run_current(); }
        if self.settings.autosave && self.editor.active().map(|f|f.dirty).unwrap_or(false) { self.save_current(); }
        if self.upload_requested { ctx.request_repaint_after(Duration::from_millis(50)); }
    }

    fn save(&mut self, storage: &mut dyn Storage) { self.persist(storage); }
}

fn file_glyph(path: &str) -> &'static str { match path.rsplit('.').next().unwrap_or("") { "rs"=>"◆", "js"|"ts"=>"◇", "json"=>"{}", "md"=>"≡", "py"=>"●", _=>"□" } }
fn symbol_glyph(kind: &str) -> &'static str { match kind { "Function"=>"ƒ", "Class"=>"C", "Struct"=>"S", "Enum"=>"E", "Trait"=>"T", "Module"=>"M", _=>"•" } }
