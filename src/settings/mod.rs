use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Graphite,
    Midnight,
    Light,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub theme: Theme,
    pub font_size: f32,
    pub tab_size: usize,
    pub word_wrap: bool,
    pub minimap: bool,
    pub line_numbers: bool,
    pub animations: bool,
    pub autosave: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Graphite,
            font_size: 15.0,
            tab_size: 4,
            word_wrap: false,
            minimap: true,
            line_numbers: true,
            animations: true,
            autosave: false,
        }
    }
}

impl Settings {
    pub fn apply_visuals(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.button_padding = egui::vec2(9.0, 6.0);
        style.visuals = match self.theme {
            Theme::Graphite => graphite_visuals(),
            Theme::Midnight => midnight_visuals(),
            Theme::Light => light_visuals(),
        };
        ctx.set_style(style);
    }
}

fn graphite_visuals() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    v.override_text_color = Some(egui::Color32::from_rgb(226, 230, 237));
    v.panel_fill = egui::Color32::from_rgb(23, 25, 29);
    v.window_fill = egui::Color32::from_rgb(28, 31, 36);
    v.extreme_bg_color = egui::Color32::from_rgb(17, 19, 22);
    v.faint_bg_color = egui::Color32::from_rgb(28, 31, 36);
    v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(34, 38, 45);
    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(34, 38, 45);
    v.widgets.hovered.bg_fill = egui::Color32::from_rgb(43, 49, 57);
    v.widgets.active.bg_fill = egui::Color32::from_rgb(53, 62, 70);
    v.selection.bg_fill = egui::Color32::from_rgb(44, 93, 76);
    v.hyperlink_color = egui::Color32::from_rgb(103, 232, 181);
    v
}

fn midnight_visuals() -> egui::Visuals {
    let mut v = graphite_visuals();
    v.panel_fill = egui::Color32::from_rgb(11, 16, 25);
    v.window_fill = egui::Color32::from_rgb(17, 23, 34);
    v.extreme_bg_color = egui::Color32::from_rgb(7, 11, 18);
    v
}

fn light_visuals() -> egui::Visuals {
    let mut v = egui::Visuals::light();
    v.override_text_color = Some(egui::Color32::from_rgb(35, 39, 46));
    v.panel_fill = egui::Color32::from_rgb(244, 246, 248);
    v.window_fill = egui::Color32::from_rgb(255, 255, 255);
    v.selection.bg_fill = egui::Color32::from_rgb(190, 235, 216);
    v.hyperlink_color = egui::Color32::from_rgb(24, 125, 93);
    v
}
