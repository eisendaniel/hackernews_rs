mod hackernews;
use hackernews::{Hackernews, PADDING};

use eframe::{egui, epi};

impl epi::App for Hackernews {
    fn setup(
        &mut self,
        ctx: &eframe::egui::CtxRef,
        _frame: &eframe::epi::Frame,
        _storage: Option<&dyn eframe::epi::Storage>,
    ) {
        self.configure_fonts(ctx);
    }
    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &eframe::epi::Frame) {
        if self.config.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        self.render_top_panel(ctx, frame);
        egui::CentralPanel::default().show(ctx, |ui| {
            render_header(ui);
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.render_news_cards(ui);
            });
        });
    }

    fn name(&self) -> &str {
        "Hackernews"
    }
}

fn render_header(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("Top Stories");
    });
    ui.add_space(PADDING);
    ui.add(egui::Separator::default().spacing(20.));
}

fn main() {
    tracing_subscriber::fmt::init();

    let app = Hackernews::new();
    let mut win_option = eframe::NativeOptions::default();
    win_option.initial_window_size = Some(egui::Vec2::new(600., 800.));
    eframe::run_native(Box::new(app), win_option);
}
