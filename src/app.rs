use crate::frame_history::FrameHistory;
use eframe::{egui, epaint};
use poll_promise::Promise;
use serde::{Deserialize, Serialize};

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
            "https://hacker-news.firebaseio.com/v0/item/{}.json?print=pretty",
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
        egui::Frame::popup(ui.style())
            .stroke(epaint::Stroke::none())
            .shadow(epaint::Shadow::default())
            .rounding(8.)
            .show(ui, |ui| {
                if let Some(result) = self.story_promise.ready() {
                    match result {
                        Ok(story) => {
                            // let mut text = format!("{:?}", story);
                            ui.heading(&story.title);
                            let url = match story.url.clone() {
                                Some(url) => url,
                                None => format!("https://news.ycombinator.com/item?id={}", self.id),
                            };
                            ui.hyperlink(&url);
                        }
                        Err(e) => {
                            ui.colored_label(
                                ui.visuals().error_fg_color,
                                if e.is_empty() { "Error" } else { e },
                            );
                            self.story_promise = Self::new(self.id, ctx).story_promise;
                        }
                    }
                } else {
                    ui.spinner();
                }
            });
    }

    fn cards_from_response(response: ehttp::Response, ctx: &egui::Context) -> Vec<Card> {
        serde_json::from_str::<Vec<u32>>(response.text().unwrap())
            .unwrap()
            .iter()
            .map(|id| Self::new(*id, ctx))
            .collect()
    }
}

pub struct MainApp {
    frame_history: FrameHistory,
    stories_promise: Option<Promise<ehttp::Result<Vec<Card>>>>,
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            frame_history: FrameHistory::default(),
            stories_promise: None,
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.display_fps(ctx, frame);
        egui::TopBottomPanel::top("Bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.separator();
                if ui.button("↺").clicked() || self.stories_promise.is_none() {
                    self.refresh_stories(ctx);
                }
                ui.separator();
                // egui::containers::ComboBox::from_label("TEST")
            });
        });
        egui::CentralPanel::default()
            .frame(egui::Frame::canvas(&ctx.style()).inner_margin(16.))
            .show(ctx, |ui| self.body_ui(ui, ctx));
    }
}

impl MainApp {
    fn display_fps(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.frame_history
            .on_new_frame(ctx.input().time, frame.info().cpu_usage);

        //variable frame rate: min of 1fps and a max of vsync_max or event_count/s
        ctx.request_repaint_after(std::time::Duration::from_millis(900));
        frame.set_window_title(&format!(
            "{:.2} fps, {:.2} ms / frame",
            self.frame_history.fps(),
            self.frame_history.mean_frame_time() * 1000.
        ));
    }

    fn refresh_stories(&mut self, ctx: &egui::Context) {
        let ctx = ctx.clone();
        let (sender, promise) = Promise::new();
        let request = ehttp::Request::get(
            "https://hacker-news.firebaseio.com/v0/topstories.json?print=pretty",
        );
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
                ui.vertical_centered(|ui| {
                    egui::ScrollArea::vertical().auto_shrink([false; 2]).show(
                        ui,
                        |ui| match result {
                            Ok(stories) => {
                                for story in stories {
                                    story.draw(ui, ctx);
                                }
                            }
                            Err(e) => {
                                ui.colored_label(
                                    ui.visuals().error_fg_color,
                                    if e.is_empty() { "Error" } else { e },
                                );
                            }
                        },
                    );
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.spinner();
                });
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.label("↺\tRefresh for more...");
            });
        }
    }
}
