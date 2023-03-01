fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "hackernews_rs",
        eframe::NativeOptions {
            initial_window_size: Some([500., 720.].into()),
            min_window_size: Some([240., 24.].into()),
            transparent: true,
            centered: true,
            // vsync: false, // defaults to true
            ..Default::default()
        },
        Box::new(|_cc| Box::<hackernews_rs::MainApp>::default()),
    )
}
