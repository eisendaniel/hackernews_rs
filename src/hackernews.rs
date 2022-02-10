use eframe::{egui, epi};
use futures::future;
use reqwest::Error;
use serde::{Deserialize, Serialize};

pub const PADDING: f32 = 4.0;
const N_STORIES: usize = 32;
const N_MAX: usize = 500;

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub struct AppConfig {
    pub dark_mode: bool,
}

pub struct Hackernews {
    pub start: usize,
    pub stories: Vec<Story>,
    pub config: AppConfig,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Story {
    by: String,
    descendants: u32,
    id: u32,
    // kids: Vec<u32>,
    score: u32,
    // time: u64,
    title: String,
    // #[serde(alias = "type")]
    // item_type: String,
    url: String,
}

async fn get_stories(start: usize) -> Result<Vec<Story>, Error> {
    let api_url = "https://hacker-news.firebaseio.com/v0";
    let list_url = format!("{}/topstories.json", api_url);
    let story_ids = reqwest::get(&list_url).await?.json::<Vec<u32>>().await?;

    let story_resp = future::join_all(story_ids[(start)..(start + N_STORIES)].iter().map(
        |id| async move {
            let url = format!("{}/item/{}.json", api_url, id);
            let resp = reqwest::get(&url).await?;
            resp.json::<Story>().await
        },
    ))
    .await;

    Ok(story_resp
        .into_iter()
        .filter(|result| result.is_ok())
        .map(|story| story.unwrap())
        .collect::<Vec<Story>>())
}

impl Hackernews {
    pub fn new() -> Hackernews {
        Hackernews {
            start: 0usize,
            stories: Vec::new(),
            config: Default::default(),
        }
    }

    pub fn refresh_stories(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            self.stories = get_stories(self.start).await.unwrap();
        });
    }

    pub fn configure_fonts(&self, ctx: &egui::CtxRef) {
        let mut font_def = egui::FontDefinitions::default();
        font_def.font_data.insert(
            "MesloLGS NF".to_string(),
            egui::FontData::from_static(include_bytes!("../fonts/MesloLGS NF Regular.ttf")),
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
        let mut index = self.start;
        for card in &self.stories {
            index += 1;
            ui.add_space(PADDING);
            // render title
            let title = format!("{}. ▶ {}", index, card.title);
            ui.colored_label(
                if self.config.dark_mode {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::BLACK
                },
                title,
            );
            ui.add_space(PADDING);
            ui.add(egui::Label::new(
                egui::RichText::new(format!("{} points by {}", &card.score, &card.by))
                    .text_style(egui::TextStyle::Button),
            ));
            ui.add_space(PADDING);
            ui.add(egui::Hyperlink::from_label_and_url(
                "read more ⤴",
                &card.url,
            ));
            ui.add(egui::Hyperlink::from_label_and_url(
                format!("{} Comments", &card.descendants),
                format!("https://news.ycombinator.com/item?id={}", &card.id),
            ));
            ui.add_space(PADDING);
            ui.add(egui::Separator::default());
        }
    }

    pub fn render_top_panel(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        // define a TopBottomPanel widget
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // ui.add_space(5.);
            egui::menu::bar(ui, |ui| {
                // logo
                ui.with_layout(egui::Layout::left_to_right(), |ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("ﬣ ").text_style(egui::TextStyle::Heading),
                    ));
                });
                // controls
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    let close_btn = ui.add(egui::Button::new(
                        egui::RichText::new(" 窱 ").text_style(egui::TextStyle::Body),
                    ));
                    if close_btn.clicked() {
                        frame.quit();
                    }
                    let refresh_btn = ui.add(egui::Button::new(
                        egui::RichText::new("  ").text_style(egui::TextStyle::Body),
                    ));
                    if refresh_btn.clicked() {
                        self.refresh_stories();
                    }
                    let theme_btn = ui.add(egui::Button::new(
                        egui::RichText::new({
                            if self.config.dark_mode {
                                "  "
                            } else {
                                "  "
                            }
                        })
                        .text_style(egui::TextStyle::Body),
                    ));
                    if theme_btn.clicked() {
                        self.config.dark_mode = !self.config.dark_mode;
                    }
                });
            });
            ui.add_space(5.);
        });
    }

    pub fn render_btm_panel(&mut self, ctx: &egui::CtxRef) {
        egui::TopBottomPanel::bottom("btm_panel").show(ctx, |ui| {
            ui.add_space(5.);
            egui::menu::bar(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(), |ui| {
                    let home_btn = ui.add(egui::Button::new(
                        egui::RichText::new(" ﳐ ").text_style(egui::TextStyle::Heading),
                    ));
                    if home_btn.clicked() && self.start > 0 {
                        self.start = 0usize;
                        self.refresh_stories();
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    let next_btn = ui.add(egui::Button::new(
                        egui::RichText::new("  ").text_style(egui::TextStyle::Body),
                    ));
                    if next_btn.clicked() && self.start + N_STORIES < N_MAX {
                        self.start += N_STORIES;
                        self.refresh_stories();
                    }
                    let prev_btn = ui.add(egui::Button::new(
                        egui::RichText::new("  ").text_style(egui::TextStyle::Body),
                    ));
                    if prev_btn.clicked() && self.start >= N_STORIES {
                        self.start -= N_STORIES;
                        self.refresh_stories();
                    }
                });
            });
            ui.add_space(5.);
        });
    }
}
