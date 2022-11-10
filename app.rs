use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui;
use poll_promise::Promise;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Copy)]
enum Catagory {
    Top,
    New,
    Best,
}

impl std::fmt::Display for Catagory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Catagory::Top => "Top",
                Catagory::New => "New",
                Catagory::Best => "Best",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Story {
    by: String,
    descendants: Option<u32>,
    kids: Option<Vec<u32>>,
    score: u32,
    time: u64,
    title: String,
    text: Option<String>,
    #[serde(rename = "type")]
    item_type: String,
    url: Option<String>,
}

struct Card {
    id: u32,
    story_promise: Promise<ehttp::Result<Story>>,
}

impl Card {
    fn new(id: u32, ctx: &egui::Context) -> Self {
        let ctx = ctx.clone();
        let (sender, story_promise) = Promise::new();
        let request = ehttp::Request::get(format!(
            "https://hacker-news.firebaseio.com/v0/item/{}.json",
            id
        ));
        ehttp::fetch(request, move |response| {
            ctx.request_repaint(); // wake up UI thread
            let story =
                response.map(|response| serde_json::from_str(response.text().unwrap()).unwrap());
            sender.send(story);
        });

        Self { id, story_promise }
    }

    fn draw(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if let Some(result) = self.story_promise.ready_mut() {
            match result {
                Ok(story) => {
                    let url = match &story.url {
                        Some(url) => url.to_owned(),
                        None => format!("https://news.ycombinator.com/item?id={}", self.id),
                    };
                    let url = url::Url::parse(&url).expect(&format!("{:?}", story.url));

                    ui.heading(&story.title);
                    ui.horizontal(|ui| {
                        ui.label(format!("{} points  ⚫", story.score));
                        ui.hyperlink_to(url.domain().unwrap_or(url.as_str()), url.as_str());
                        let mins = (SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Time Travel")
                            .as_secs()
                            - story.time)
                            / 60;
                        let when = if mins < 1 {
                            "⚫  just now".into()
                        } else if mins < 2 {
                            "⚫  1 min".into()
                        } else if mins < 60 {
                            format!("⚫  {} mins", mins)
                        } else if mins < 120 {
                            "⚫  1 hr".into()
                        } else if mins < 1440 {
                            format!("⚫  {} hrs", mins / 60)
                        } else if mins < 2880 {
                            "⚫  1 day".into()
                        } else {
                            format!("⚫  {} days", mins / 1440)
                        };
                        ui.label(when);
                    });
                }
                Err(_) => {
                    self.story_promise = Self::new(self.id, ctx).story_promise;
                }
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.spinner();
            });
        }
        ui.separator();
    }

    fn cards_from_response(response: ehttp::Response, ctx: &egui::Context) -> Vec<Self> {
        serde_json::from_str::<Vec<u32>>(response.text().unwrap())
            .unwrap()
            .into_iter()
            .map(|id| Self::new(id, ctx))
            .collect()
    }
}

pub struct MainApp {
    stories_promise: Option<Promise<ehttp::Result<Vec<Card>>>>,
    category: Catagory,
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            stories_promise: None,
            category: Catagory::Top,
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        // self._display_fps(ctx, _frame);

        frame.set_window_title(&format!("HN::{} Stories", self.category));

        egui::TopBottomPanel::top("Bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.separator();
                if ui.button("↺").clicked() || self.stories_promise.is_none() {
                    self.refresh_stories(ctx);
                }
                ui.separator();
                egui::containers::ComboBox::from_id_source("select story list")
                    .selected_text(self.category.to_string() + " Stories")
                    .show_ui(ui, |ui| {
                        let options = [Catagory::Top, Catagory::New, Catagory::Best];
                        let mut changed = false;
                        for option in options {
                            changed |= ui
                                .selectable_value(
                                    &mut self.category,
                                    option,
                                    option.to_string() + " Stories",
                                )
                                .changed()
                        }
                        if changed {
                            self.refresh_stories(ctx)
                        }
                    });
            });
        });
        egui::CentralPanel::default()
            .frame(egui::Frame::canvas(&ctx.style()).inner_margin(16.))
            .show(ctx, |ui| self.body_ui(ui, ctx));
    }
}

impl MainApp {
    fn refresh_stories(&mut self, ctx: &egui::Context) {
        let ctx = ctx.clone();
        let (sender, promise) = Promise::new();
        let request = ehttp::Request::get(format!(
            "https://hacker-news.firebaseio.com/v0/{}stories.json",
            &self.category.to_string().to_lowercase()
        ));
        ehttp::fetch(request, move |response| {
            ctx.request_repaint(); // wake up UI thread
            let stories = response.map(|response| Card::cards_from_response(response, &ctx));
            sender.send(stories);
        });
        self.stories_promise = Some(promise);
    }

    fn body_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if let Some(promise) = &mut self.stories_promise {
            if let Some(result) = promise.ready_mut() {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| match result {
                        Ok(deck) => {
                            for card in deck {
                                card.draw(ui, ctx);
                            }
                        }
                        Err(e) => {
                            ui.label("↺\tRefresh to retry...");
                            ui.separator();
                            ui.colored_label(
                                ui.visuals().error_fg_color,
                                if e.is_empty() { "Error" } else { e },
                            );
                        }
                    });
            } else {
                ui.vertical_centered(|ui| {
                    ui.spinner();
                });
            }
        } else {
            ui.vertical_centered(|ui| ui.label("↺\tRefresh to retry..."));
        }
    }
}
