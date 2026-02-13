mod app;
mod disk;
mod flash;
mod image;

use app::InstallerApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 480.0])
            .with_min_inner_size([600.0, 480.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "PiNAS Installer",
        options,
        Box::new(|cc| {
            let ctx = &cc.egui_ctx;

            // Dark theme
            ctx.set_visuals(egui::Visuals::dark());

            // Modern style: bigger text, rounder widgets, more padding
            let mut style = (*ctx.style()).clone();
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::proportional(24.0),
            );
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::proportional(15.0),
            );
            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::proportional(15.0),
            );
            style.text_styles.insert(
                egui::TextStyle::Small,
                egui::FontId::proportional(12.0),
            );
            style.spacing.button_padding = egui::vec2(16.0, 8.0);
            style.spacing.item_spacing = egui::vec2(10.0, 8.0);
            let rounding = egui::Rounding::same(8);
            style.visuals.widgets.inactive.corner_radius = rounding;
            style.visuals.widgets.hovered.corner_radius = rounding;
            style.visuals.widgets.active.corner_radius = rounding;
            style.visuals.widgets.noninteractive.corner_radius = rounding;
            style.visuals.selection.bg_fill = egui::Color32::from_rgb(50, 100, 200);
            ctx.set_style(style);

            Ok(Box::new(InstallerApp::new()))
        }),
    )
}
