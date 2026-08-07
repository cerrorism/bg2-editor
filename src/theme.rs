//! Custom dark-parchment/gold `egui::Visuals`/`Style`, applied once at
//! startup. No extracted game art here — just color/shape tuning, meant
//! to evoke the Infinity Engine's warm dark-stone-and-gold UI without
//! reproducing its actual chrome graphics.

use egui::{Color32, CornerRadius, Stroke};

const PANEL_BG: Color32 = Color32::from_rgb(30, 24, 18);
const WINDOW_BG: Color32 = Color32::from_rgb(38, 30, 22);
const EXTREME_BG: Color32 = Color32::from_rgb(24, 19, 14);
const WIDGET_BG: Color32 = Color32::from_rgb(52, 42, 30);
const WIDGET_HOVER_BG: Color32 = Color32::from_rgb(70, 56, 38);
const WIDGET_ACTIVE_BG: Color32 = Color32::from_rgb(90, 70, 44);
const GOLD_ACCENT: Color32 = Color32::from_rgb(201, 162, 95);
const TEXT_COLOR: Color32 = Color32::from_rgb(224, 210, 180);
const SELECTION_BG: Color32 = Color32::from_rgb(110, 84, 40);

pub fn setup(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = PANEL_BG;
    visuals.window_fill = WINDOW_BG;
    visuals.extreme_bg_color = EXTREME_BG;
    visuals.override_text_color = Some(TEXT_COLOR);
    visuals.window_stroke = Stroke::new(1.0_f32, GOLD_ACCENT);
    visuals.selection.bg_fill = SELECTION_BG;
    visuals.selection.stroke = Stroke::new(1.0_f32, GOLD_ACCENT);

    visuals.widgets.noninteractive.bg_fill = PANEL_BG;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, GOLD_ACCENT.gamma_multiply(0.8));
    visuals.widgets.inactive.bg_fill = WIDGET_BG;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT_COLOR);
    visuals.widgets.hovered.bg_fill = WIDGET_HOVER_BG;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, GOLD_ACCENT);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, GOLD_ACCENT);
    visuals.widgets.active.bg_fill = WIDGET_ACTIVE_BG;
    visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, GOLD_ACCENT);
    visuals.widgets.open.bg_fill = WIDGET_HOVER_BG;

    for w in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(4);
    }

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    ctx.set_style(style);
}
