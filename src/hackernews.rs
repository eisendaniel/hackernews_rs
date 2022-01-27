use eframe::{egui, epi};
use serde::{Deserialize, Serialize};

pub const PADDING: f32 = 4.0;

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub struct AppConfig {
    pub dark_mode: bool,
}

pub struct Hackernews {
    pub stories: Vec<NewsCardData>,
    pub config: AppConfig,
}

pub struct NewsCardData {
    pub title: String,
    pub desc: String,
    pub url: String,
}

impl Hackernews {
    pub fn new() -> Hackernews {
        let stories = (0..20)
            .into_iter()
            .map(|i| NewsCardData {
                title: format!("Title: {}", i),
                desc: format!("Description: {}", i),
                url: format!("https:://example.com/{}", i),
            })
            .collect();
        let config: AppConfig = confy::load("hackernews").unwrap_or_default();

        Hackernews { stories, config }
    }

    fn save_config(&self) {
        if let Err(e) = confy::store("hackernews", self.config) {
            tracing::error!("{}", e);
        }
    }

    pub fn configure_fonts(&self, ctx: &egui::CtxRef) {
        let mut font_def = egui::FontDefinitions::default();
        font_def.font_data.insert(
            "MesloLGS NF".to_string(),
            egui::FontData::from_static(include_bytes!(
                "/home/daniel/.local/share/fonts/MesloLGS NF Regular.ttf"
            )),
        );
        font_def
            .family_and_size
            .insert(egui::TextStyle::Heading, (egui::FontFamily::Monospace, 35.));
        font_def
            .family_and_size
            .insert(egui::TextStyle::Body, (egui::FontFamily::Monospace, 20.));
        font_def
            .fonts_for_family
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "MesloLGS NF".to_string());
        ctx.set_fonts(font_def);
    }

    pub fn render_news_cards(&self, ui: &mut eframe::egui::Ui) {
        ui.style_mut().visuals.hyperlink_color = if self.config.dark_mode {
            egui::Color32::LIGHT_BLUE
        } else {
            egui::Color32::RED
        };
        for card in &self.stories {
            ui.add_space(PADDING);
            // render title
            let title = format!("▶ {}", card.title);
            ui.colored_label(
                if self.config.dark_mode {
                    egui::Color32::BLACK
                } else {
                    egui::Color32::WHITE
                },
                title,
            );
            // render desc
            ui.add_space(PADDING);
            let desc = egui::Label::new(
                egui::RichText::new(&card.desc).text_style(egui::TextStyle::Button),
            );
            ui.add(desc);

            // render hyperlinks

            ui.add_space(PADDING);
            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                ui.add(egui::Hyperlink::from_label_and_url(
                    "read more ⤴",
                    &card.url,
                ));
            });
            ui.add_space(PADDING);
            ui.add(egui::Separator::default());
        }
    }

    pub fn render_top_panel(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        // define a TopBottomPanel widget
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            egui::menu::bar(ui, |ui| {
                // logo
                ui.with_layout(egui::Layout::left_to_right(), |ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("📓").text_style(egui::TextStyle::Heading),
                    ));
                });
                // controls
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    let close_btn = ui.add(egui::Button::new(
                        egui::RichText::new("❌").text_style(egui::TextStyle::Body),
                    ));
                    if close_btn.clicked() {
                        frame.quit();
                    }
                    let _refresh_btn = ui.add(egui::Button::new(
                        egui::RichText::new("🔄").text_style(egui::TextStyle::Body),
                    ));
                    let theme_btn = ui.add(egui::Button::new(
                        egui::RichText::new({
                            if self.config.dark_mode {
                                "🌞"
                            } else {
                                "🌙"
                            }
                        })
                        .text_style(egui::TextStyle::Body),
                    ));
                    if theme_btn.clicked() {
                        self.save_config();
                        self.config.dark_mode = !self.config.dark_mode;
                    }
                });
            });
            ui.add_space(10.);
        });
    }
}
