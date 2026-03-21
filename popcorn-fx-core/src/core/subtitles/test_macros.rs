/// Create subtitle application settings.
macro_rules! settings {
    ($temp_path:expr) => {{
        settings!($temp_path, false)
    }};
    ($temp_path:expr, $auto_cleaning_mode:expr) => {{
        use crate::core::config::{
            ApplicationConfig, DecorationType, PopcornProperties, PopcornSettings, SubtitleFamily,
            SubtitleSettings,
        };
        use crate::core::subtitles::language::SubtitleLanguage;

        let temp_path: &str = $temp_path;
        let auto_cleaning_enabled: bool = $auto_cleaning_mode;

        ApplicationConfig::builder()
            .storage(temp_path)
            .properties(PopcornProperties::default())
            .settings(PopcornSettings {
                subtitle_settings: SubtitleSettings {
                    directory: temp_path.to_string(),
                    auto_cleaning_enabled,
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

/// Create a subtitle manager instance.
macro_rules! subtitle_manager {
    ($settings:expr) => {{
        subtitle_manager!(
            $settings,
            crate::core::subtitles::provider::MockSubtitleProvider::new()
        )
    }};
    ($settings:expr, $provider:expr) => {{
        use crate::core::config::ApplicationConfig;
        use crate::core::subtitles::manager::SubtitleManager;

        let settings: ApplicationConfig = $settings;

        SubtitleManager::builder()
            .settings(settings)
            .provider($provider)
            .build()
    }};
}
