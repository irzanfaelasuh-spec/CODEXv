use egui::{Color32, RichText, Ui};

pub const MINT: Color32 = Color32::from_rgb(103, 232, 181);
pub const TEXT: Color32 = Color32::from_rgb(226, 230, 237);
pub const MUTED: Color32 = Color32::from_rgb(146, 155, 168);
pub const BORDER: Color32 = Color32::from_rgb(48, 54, 64);

pub fn icon_button(ui: &mut Ui, glyph: &str, tooltip: &str, active: bool) -> egui::Response {
    let text = RichText::new(glyph).size(18.0).color(if active { MINT } else { MUTED });
    ui.add(egui::Button::new(text).frame(false).min_size(egui::vec2(42.0, 42.0))).on_hover_text(tooltip)
}

pub fn section_title(ui: &mut Ui, title: &str, count: Option<usize>) {
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        ui.label(RichText::new(title.to_uppercase()).strong().size(11.0).color(MUTED));
        if let Some(count) = count { ui.label(RichText::new(count.to_string()).size(10.0).color(MUTED)); }
    });
}

pub fn divider(ui: &mut Ui) { ui.separator(); }
