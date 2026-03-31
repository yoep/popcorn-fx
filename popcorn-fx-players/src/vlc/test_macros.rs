/// Create an application settings instance.
macro_rules! settings {
    ($temp_path:expr) => {{
        use popcorn_fx_core::core::config::ApplicationConfig;
        use popcorn_fx_core::core::config::DecorationType;
        use popcorn_fx_core::core::config::PopcornProperties;
        use popcorn_fx_core::core::config::PopcornSettings;
        use popcorn_fx_core::core::config::SubtitleFamily;
        use popcorn_fx_core::core::config::SubtitleSettings;
        use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;

        let temp_path: &str = $temp_path;

        ApplicationConfig::builder()
            .storage(temp_path)
            .properties(PopcornProperties::default())
            .settings(PopcornSettings {
                subtitle_settings: SubtitleSettings {
                    directory: temp_path.to_string(),
                    auto_cleaning_enabled: false,
                    default_subtitle: SubtitleLanguage::English,
                    font_family: SubtitleFamily::Arial,
                    font_size: 28,
                    decoration: DecorationType::None,
                    bold: false,
                },
                ui_settings: Default::default(),
                server_settings: Default::default(),
                torrent_settings: Default::default(),
                playback_settings: Default::default(),
                tracking_settings: Default::default(),
            })
            .build()
    }};
}
