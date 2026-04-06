use std::sync::Once;

pub static INIT: Once = Once::new();

/// Initializes the logger with the specified log level.
macro_rules! init_logger {
    () => {{
        init_logger!(log::LevelFilter::Trace)
    }};
    ($level:expr) => {{
        use log4rs::Config;
        use log4rs::append::console::ConsoleAppender;
        use log4rs::config::Appender;
        use log4rs::config::Root;
        use log4rs::encode::pattern::PatternEncoder;
        use log::LevelFilter;

        let level: LevelFilter = $level;

        crate::test_macros::INIT.call_once(|| {
            log4rs::init_config(Config::builder()
                .appender(Appender::builder().build("stdout", Box::new(ConsoleAppender::builder()
                    .encoder(Box::new(PatternEncoder::new("\x1B[37m{d(%Y-%m-%d %H:%M:%S%.3f)}\x1B[0m {h({l:>5.5})} \x1B[35m{I:>6.6}\x1B[0m \x1B[37m---\x1B[0m \x1B[37m[{T:>15.15}]\x1B[0m \x1B[36m{t:<60.60}\x1B[0m \x1B[37m:\x1B[0m {m}{n}")))
                    .build())))
                .build(Root::builder().appender("stdout").build(level))
                .unwrap())
                .unwrap();
        })
    }};
}

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
