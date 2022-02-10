mod hackernews;
use eframe::{egui, epi};
use hackernews::{Hackernews, PADDING};

impl epi::App for Hackernews {
    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        storage: Option<&dyn epi::Storage>,
    ) {
        if let Some(storage) = storage {
            self.config = epi::get_value(storage, "hackernews").unwrap_or_default();
        }
        self.configure_fonts(ctx);
        self.refresh_stories();
    }
    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
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
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
            });
        });
        self.render_btm_panel(ctx);
    }

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, "hackernews", &self.config);
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
    let app = Hackernews::new();
    let win_option = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(600., 800.)),
        ..Default::default()
    };
    eframe::run_native(Box::new(app), win_option);
}
