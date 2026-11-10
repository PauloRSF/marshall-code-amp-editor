use egui::{Color32, CornerRadius, FontFamily, FontId, Margin, RichText, Stroke, Theme, Visuals};

pub const GOLD: Color32 = Color32::from_rgb(212, 160, 74);
pub const GOLD_BRIGHT: Color32 = Color32::from_rgb(232, 184, 96);
pub const GOLD_DIM: Color32 = Color32::from_rgb(156, 112, 42);
pub const CREAM: Color32 = Color32::from_rgb(245, 240, 230);
pub const MUTED: Color32 = Color32::from_rgb(150, 142, 128);
pub const BG: Color32 = Color32::from_rgb(18, 18, 18);
pub const PANEL: Color32 = Color32::from_rgb(28, 28, 28);
pub const PANEL_INNER: Color32 = Color32::from_rgb(34, 34, 34);
pub const STROKE: Color32 = Color32::from_rgb(55, 55, 55);
pub const INK_DARK: Color32 = Color32::from_rgb(22, 16, 8);
pub const DANGER: Color32 = Color32::from_rgb(196, 68, 54);
pub const DANGER_HOVER: Color32 = Color32::from_rgb(220, 86, 70);
pub const OK: Color32 = Color32::from_rgb(125, 207, 108);

pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.style_of(Theme::Dark)).clone();
    style.spacing.item_spacing = egui::vec2(12.0, 10.0);
    style.spacing.button_padding = egui::vec2(16.0, 8.0);
    style.spacing.slider_width = 180.0;
    style.visuals = dark_visuals();
    style.text_styles.insert(
        egui::TextStyle::Heading,
        FontId::new(28.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        FontId::new(13.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        FontId::new(11.5, FontFamily::Proportional),
    );
    ctx.set_style_of(Theme::Dark, style);
    ctx.set_theme(Theme::Dark);
}

fn dark_visuals() -> Visuals {
    let mut v = Visuals::dark();
    v.override_text_color = Some(CREAM);
    v.panel_fill = BG;
    v.window_fill = PANEL;
    v.extreme_bg_color = Color32::from_rgb(12, 12, 12);
    v.faint_bg_color = PANEL_INNER;
    v.widgets.inactive.bg_fill = PANEL_INNER;
    v.widgets.inactive.weak_bg_fill = PANEL_INNER;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, MUTED);
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, STROKE);
    v.widgets.hovered.bg_fill = Color32::from_rgb(42, 42, 42);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, GOLD_DIM);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, CREAM);
    v.widgets.active.bg_fill = GOLD_DIM;
    v.widgets.active.fg_stroke = Stroke::new(1.0, INK_DARK);
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(212, 160, 74, 70);
    v.selection.stroke = Stroke::new(1.0, GOLD);
    v.slider_trailing_fill = true;
    v.handle_shape = egui::style::HandleShape::Circle;
    v.window_corner_radius = CornerRadius::same(14);
    v.menu_corner_radius = CornerRadius::same(8);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, STROKE);
    v
}

pub fn brand(text: &str) -> RichText {
    RichText::new(text).color(GOLD).size(42.0).strong()
}

pub fn subtitle(text: &str) -> RichText {
    RichText::new(text).color(MUTED).size(14.0)
}

pub fn field_label(text: &str) -> RichText {
    RichText::new(text).color(MUTED).size(11.0).strong()
}

pub fn block_index(text: &str) -> RichText {
    RichText::new(text).color(GOLD).size(18.0).strong()
}

pub fn block_title(text: &str) -> RichText {
    RichText::new(text).color(CREAM).size(18.0).strong()
}

pub fn toolbar_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1.0, STROKE))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::symmetric(16, 12))
}

pub fn block_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1.0, STROKE))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(Margin::symmetric(18, 14))
}
