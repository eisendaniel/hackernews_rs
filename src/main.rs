mod hackernews;
use hackernews::{Hackernews, PADDING};
use eframe::{egui, epi};

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
        });
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

#[tokio::main]
async fn main() {
    let app = Hackernews::new().await;
    let mut win_option = eframe::NativeOptions::default();
    win_option.initial_window_size = Some(egui::Vec2::new(600., 800.));
    eframe::run_native(Box::new(app), win_option);
}
