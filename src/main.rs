use hackernews_rs::MainApp;

fn main() {
    eframe::run_native(
        "hackernews_rs",
        eframe::NativeOptions {
            initial_window_size: Some([400., 600.].into()),
            min_window_size: Some([240., 24.].into()),
            transparent: true,
            // vsync: false, // defaults to true
            ..Default::default()
        },
        Box::new(|_cc| Box::new(MainApp::default())),
    )
}
