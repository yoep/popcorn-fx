/// Create application settings instance.
macro_rules! settings {
    ($temp_path:expr) => {{
        use popcorn_fx_core::core::config::ApplicationConfig;

        ApplicationConfig::builder().storage($temp_path).build()
    }};
}

/// Create a subtitle manager instance.
macro_rules! subtitle_manager {
    ($settings:expr) => {{
        subtitle_manager!(
            $settings,
            popcorn_fx_core::core::subtitles::MockSubtitleProvider::new()
        )
    }};
    ($settings:expr, $provider:expr) => {{
        use popcorn_fx_core::core::config::ApplicationConfig;
        use popcorn_fx_core::core::subtitles::SubtitleManager;

        let settings: ApplicationConfig = $settings;

        SubtitleManager::builder()
            .settings(settings)
            .provider($provider)
            .build()
    }};
}
