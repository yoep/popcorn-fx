/// Create application settings instance.
macro_rules! settings {
    ($temp_path:expr) => {{
        use popcorn_fx_core::core::config::{ApplicationConfig, PopcornSettings};
        use popcorn_fx_core::core::config::{DecorationType, SubtitleFamily, UiScale};
        use popcorn_fx_core::core::media::Category;
        use popcorn_fx_core::core::subtitles::language::SubtitleLanguage::English;

        let temp_path: &str = $temp_path;

        ApplicationConfig::builder()
            .storage($temp_path)
            .settings(PopcornSettings {
                subtitle_settings: SubtitleSettings {
                    directory: temp_path.to_string(),
                    auto_cleaning_enabled: false,
                    default_subtitle: English,
                    font_family: SubtitleFamily::Arial,
                    font_size: 28,
                    decoration: DecorationType::None,
                    bold: false,
                },
                ui_settings: UiSettings {
                    default_language: "en".to_string(),
                    ui_scale: UiScale::new(1f32).expect("Expected ui scale to be valid"),
                    start_screen: Category::Movies,
                    maximized: false,
                    native_window_enabled: false,
                },
                server_settings: ServerSettings::default(),
                torrent_settings: TorrentSettings::default(),
                playback_settings: Default::default(),
                tracking_settings: Default::default(),
            })
            .build()
    }};
}
