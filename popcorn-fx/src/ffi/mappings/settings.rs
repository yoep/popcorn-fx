use std::os::raw::c_char;
use std::path::PathBuf;
use std::ptr;

use log::trace;

use popcorn_fx_core::{from_c_owned, from_c_string, into_c_owned, into_c_string};
use popcorn_fx_core::core::config::{ApplicationConfigEvent, CleaningMode, DecorationType, LastSync, MediaTrackingSyncState, PlaybackSettings, PopcornSettings, Quality, ServerSettings, SubtitleFamily, SubtitleSettings, TorrentSettings, TrackingSettings, UiScale, UiSettings};
use popcorn_fx_core::core::media::Category;
use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;

/// The C callback for the setting events.
pub type ApplicationConfigCallbackC = extern "C" fn(ApplicationConfigEventC);

/// The C compatible application events.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum ApplicationConfigEventC {
    /// Invoked when the application settings have been reloaded or loaded
    SettingsLoaded,
    /// Invoked when the subtitle settings have been changed
    SubtitleSettingsChanged(SubtitleSettingsC),
    /// Invoked when the torrent settings have been changed
    TorrentSettingsChanged(TorrentSettingsC),
    /// Invoked when the ui settings have been changed
    UiSettingsChanged(UiSettingsC),
    /// Invoked when the server settings have been changed
    ServerSettingsChanged(ServerSettingsC),
    /// Invoked when the playback settings have been changed
    PlaybackSettingsChanged(PlaybackSettingsC),
    /// Invoked when the tracking settings have been changed
    TrackingSettingsChanged(TrackingSettingsC),
}

impl From<ApplicationConfigEvent> for ApplicationConfigEventC {
    fn from(value: ApplicationConfigEvent) -> Self {
        match value {
            ApplicationConfigEvent::SettingsLoaded => ApplicationConfigEventC::SettingsLoaded,
            ApplicationConfigEvent::SubtitleSettingsChanged(e) => ApplicationConfigEventC::SubtitleSettingsChanged(SubtitleSettingsC::from(&e)),
            ApplicationConfigEvent::TorrentSettingsChanged(e) => ApplicationConfigEventC::TorrentSettingsChanged(TorrentSettingsC::from(&e)),
            ApplicationConfigEvent::UiSettingsChanged(e) => ApplicationConfigEventC::UiSettingsChanged(UiSettingsC::from(&e)),
            ApplicationConfigEvent::ServerSettingsChanged(e) => ApplicationConfigEventC::ServerSettingsChanged(ServerSettingsC::from(&e)),
            ApplicationConfigEvent::PlaybackSettingsChanged(e) => ApplicationConfigEventC::PlaybackSettingsChanged(PlaybackSettingsC::from(&e)),
            ApplicationConfigEvent::TrackingSettingsChanged(e) => ApplicationConfigEventC::TrackingSettingsChanged(TrackingSettingsC::from(&e)),
        }
    }
}

/// The C compatible application settings.
#[repr(C)]
#[derive(Debug)]
pub struct PopcornSettingsC {
    /// The subtitle settings of the application
    pub subtitle_settings: SubtitleSettingsC,
    /// The torrent settings of the application
    pub torrent_settings: TorrentSettingsC,
    /// The ui settings of the application
    pub ui_settings: UiSettingsC,
    /// The api server settings of the application
    pub server_settings: ServerSettingsC,
    /// The playback settings of the application
    pub playback_settings: PlaybackSettingsC,
    /// The tracking settings of the application
    pub tracking_settings: TrackingSettingsC,
}

impl From<PopcornSettings> for PopcornSettingsC {
    fn from(value: PopcornSettings) -> Self {
        trace!("Converting PopcornSettings to C for {:?}", value);
        Self {
            subtitle_settings: SubtitleSettingsC::from(value.subtitle()),
            torrent_settings: TorrentSettingsC::from(value.torrent()),
            ui_settings: UiSettingsC::from(value.ui()),
            server_settings: ServerSettingsC::from(value.server()),
            playback_settings: PlaybackSettingsC::from(value.playback()),
            tracking_settings: TrackingSettingsC::from(value.tracking()),
        }
    }
}

/// The C compatible subtitle settings.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct SubtitleSettingsC {
    /// The directory path for storing subtitles
    pub directory: *mut c_char,
    /// Indicates if the subtitle directory will be cleaned
    /// when the application is closed
    pub auto_cleaning: bool,
    /// The default selected subtitle language
    pub default_subtitle: SubtitleLanguage,
    /// The subtitle font to use
    pub font_family: SubtitleFamily,
    /// The subtitle font size to use
    pub font_size: u32,
    /// The subtitle rendering decoration type
    pub decoration: DecorationType,
    /// Indicates if the subtitle should be rendered in a bold font
    pub bold: bool,
}

impl From<&SubtitleSettings> for SubtitleSettingsC {
    fn from(value: &SubtitleSettings) -> Self {
        Self {
            directory: into_c_string(value.directory.clone()),
            auto_cleaning: value.auto_cleaning_enabled,
            default_subtitle: value.default_subtitle,
            font_family: value.font_family,
            font_size: value.font_size,
            decoration: value.decoration,
            bold: value.bold,
        }
    }
}

impl From<SubtitleSettingsC> for SubtitleSettings {
    fn from(value: SubtitleSettingsC) -> Self {
        Self {
            directory: from_c_string(value.directory),
            auto_cleaning_enabled: value.auto_cleaning,
            default_subtitle: value.default_subtitle,
            font_family: value.font_family,
            font_size: value.font_size,
            decoration: value.decoration,
            bold: value.bold,
        }
    }
}

/// The C compatible torrent settings.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct TorrentSettingsC {
    /// The torrent directory to store the torrents
    pub directory: *mut c_char,
    /// Indicates if the torrents directory will be cleaned on closure
    pub cleaning_mode: CleaningMode,
    /// The max number of connections
    pub connections_limit: u32,
    /// The download rate limit
    pub download_rate_limit: u32,
    /// The upload rate limit
    pub upload_rate_limit: u32,
}

impl From<&TorrentSettings> for TorrentSettingsC {
    fn from(value: &TorrentSettings) -> Self {
        Self {
            directory: into_c_string(value.directory().to_str().unwrap().to_string()),
            cleaning_mode: value.cleaning_mode.clone(),
            connections_limit: value.connections_limit,
            download_rate_limit: value.download_rate_limit,
            upload_rate_limit: value.upload_rate_limit,
        }
    }
}

impl From<TorrentSettingsC> for TorrentSettings {
    fn from(value: TorrentSettingsC) -> Self {
        Self {
            directory: PathBuf::from(from_c_string(value.directory)),
            cleaning_mode: value.cleaning_mode,
            connections_limit: value.connections_limit,
            download_rate_limit: value.download_rate_limit,
            upload_rate_limit: value.upload_rate_limit,
        }
    }
}

/// The C compatible ui settings
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct UiSettingsC {
    /// The default language of the application
    pub default_language: *mut c_char,
    /// The ui scale of the application
    pub ui_scale: UiScale,
    /// The default start screen of the application
    pub start_screen: Category,
    /// The indication if the UI was maximized the last time the application was closed
    pub maximized: bool,
    /// The indication if the UI should use a native window rather than the borderless stage
    pub native_window_enabled: bool,
}

impl From<&UiSettings> for UiSettingsC {
    fn from(value: &UiSettings) -> Self {
        Self {
            default_language: into_c_string(value.default_language.clone()),
            ui_scale: value.ui_scale.clone(),
            start_screen: value.start_screen.clone(),
            maximized: value.maximized,
            native_window_enabled: value.native_window_enabled,
        }
    }
}

impl From<UiSettingsC> for UiSettings {
    fn from(value: UiSettingsC) -> Self {
        Self {
            default_language: from_c_string(value.default_language),
            ui_scale: value.ui_scale,
            start_screen: value.start_screen,
            maximized: value.maximized,
            native_window_enabled: value.native_window_enabled,
        }
    }
}

/// The C compatible server settings.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ServerSettingsC {
    /// The configured api server to use, can be `ptr::null()`
    pub api_server: *mut c_char,
}

impl From<&ServerSettings> for ServerSettingsC {
    fn from(value: &ServerSettings) -> Self {
        Self {
            api_server: match value.api_server() {
                None => ptr::null_mut(),
                Some(e) => into_c_string(e.clone())
            },
        }
    }
}

impl From<ServerSettingsC> for ServerSettings {
    fn from(value: ServerSettingsC) -> Self {
        let api_server = if !value.api_server.is_null() {
            Some(from_c_string(value.api_server))
        } else {
            None
        };

        Self {
            api_server,
        }
    }
}

/// The C compatible playback settings
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct PlaybackSettingsC {
    /// The default playback quality
    pub quality: *mut Quality,
    /// Indicates if the playback will be opened in fullscreen mode
    pub fullscreen: bool,
    /// Indicates if the next episode of the show will be played
    pub auto_play_next_episode_enabled: bool,
}

impl From<&PlaybackSettings> for PlaybackSettingsC {
    fn from(value: &PlaybackSettings) -> Self {
        let quality = match &value.quality {
            None => ptr::null_mut(),
            Some(e) => into_c_owned(e.clone())
        };

        Self {
            quality,
            fullscreen: value.fullscreen,
            auto_play_next_episode_enabled: value.auto_play_next_episode_enabled,
        }
    }
}

impl From<PlaybackSettingsC> for PlaybackSettings {
    fn from(value: PlaybackSettingsC) -> Self {
        let quality = if !value.quality.is_null() {
            Some(from_c_owned(value.quality))
        } else {
            None
        };

        Self {
            quality,
            fullscreen: value.fullscreen,
            auto_play_next_episode_enabled: value.auto_play_next_episode_enabled,
        }
    }
}

/// Represents the C-compatible struct for tracking settings.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct TrackingSettingsC {
    /// Pointer to the last sync.
    pub last_sync: *mut LastSyncC,
}

impl From<&TrackingSettings> for TrackingSettingsC {
    /// Converts from `TrackingSettings` to `TrackingSettingsC`.
    ///
    /// # Arguments
    ///
    /// * `value` - The `TrackingSettings` value to convert.
    ///
    /// # Returns
    ///
    /// Returns the converted `TrackingSettingsC` value.
    fn from(value: &TrackingSettings) -> Self {
        let last_sync = if let Some(e) = value.last_sync() {
            into_c_owned(LastSyncC::from(e.clone()))
        } else {
            ptr::null_mut()
        };

        Self {
            last_sync,
        }
    }
}

/// Represents the C-compatible struct for the last sync.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct LastSyncC {
    /// The number of non-leap seconds since January 1, 1970 0:00:00 UTC = Unix timestamp.
    pub time: i64,
    /// The state of media tracking sync.
    pub state: MediaTrackingSyncState,
}

impl From<LastSync> for LastSyncC {
    /// Converts from `LastSync` to `LastSyncC`.
    ///
    /// # Arguments
    ///
    /// * `value` - The `LastSync` value to convert.
    ///
    /// # Returns
    ///
    /// Returns the converted `LastSyncC` value.
    fn from(value: LastSync) -> Self {
        Self {
            time: value.time.timestamp(),
            state: MediaTrackingSyncState::Success,
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use chrono::{Local, Utc};

    use popcorn_fx_core::core::config::SubtitleFamily;
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;

    use crate::from_c_string;

    use super::*;

    #[test]
    fn test_from_application_settings_event() {
        let subtitle_directory = "/tmp/lorem/ipsum_subtitles";
        let subtitle = SubtitleSettings {
            directory: subtitle_directory.to_string(),
            auto_cleaning_enabled: false,
            default_subtitle: SubtitleLanguage::None,
            font_family: SubtitleFamily::Arial,
            font_size: 22,
            decoration: DecorationType::None,
            bold: false,
        };
        let loaded_event = ApplicationConfigEvent::SettingsLoaded;
        let subtitle_event = ApplicationConfigEvent::SubtitleSettingsChanged(subtitle.clone());

        let loaded_result = ApplicationConfigEventC::from(loaded_event);
        let subtitle_result = ApplicationConfigEventC::from(subtitle_event);

        assert_eq!(ApplicationConfigEventC::SettingsLoaded, loaded_result);
        match subtitle_result {
            ApplicationConfigEventC::SubtitleSettingsChanged(result) => {
                let subtitle_result = SubtitleSettings::from(result);
                assert_eq!(subtitle, subtitle_result)
            }
            _ => assert!(false, "expected ApplicationConfigEventC::SubtitleSettingsChanged")
        }
    }

    #[test]
    fn test_from_subtitle_settings() {
        let directory = "/var/lorem/ipsum";
        let font_family = SubtitleFamily::Verdana;
        let subtitle_language = SubtitleLanguage::French;
        let settings = SubtitleSettings {
            directory: directory.to_string(),
            auto_cleaning_enabled: false,
            default_subtitle: subtitle_language.clone(),
            font_family: font_family.clone(),
            font_size: 28,
            decoration: DecorationType::Outline,
            bold: true,
        };

        let result = SubtitleSettingsC::from(&settings);

        assert_eq!(directory.to_string(), from_c_string(result.directory));
        assert_eq!(false, result.auto_cleaning);
        assert_eq!(subtitle_language, result.default_subtitle);
        assert_eq!(font_family, result.font_family);
        assert_eq!(28, result.font_size);
        assert_eq!(DecorationType::Outline, result.decoration);
        assert_eq!(true, result.bold);
    }

    #[test]
    fn test_from_subtitle_settings_c() {
        let directory = "/var/lorem/ipsum/dolor";
        let font_size = 32;
        let settings = SubtitleSettingsC {
            directory: into_c_string(directory.to_string()),
            auto_cleaning: true,
            default_subtitle: SubtitleLanguage::German,
            font_family: SubtitleFamily::ComicSans,
            font_size,
            decoration: DecorationType::OpaqueBackground,
            bold: true,
        };
        let expected_result = SubtitleSettings {
            directory: directory.to_string(),
            auto_cleaning_enabled: true,
            default_subtitle: SubtitleLanguage::German,
            font_family: SubtitleFamily::ComicSans,
            font_size,
            decoration: DecorationType::OpaqueBackground,
            bold: true,
        };

        let result = SubtitleSettings::from(settings);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_torrent_settings_c_from() {
        let directory = "/tmp/lorem/torrent";
        let settings = TorrentSettings {
            directory: PathBuf::from(directory),
            cleaning_mode: CleaningMode::Off,
            connections_limit: 100,
            download_rate_limit: 0,
            upload_rate_limit: 0,
        };

        let result = TorrentSettingsC::from(&settings);

        assert_eq!(directory.to_string(), from_c_string(result.directory));
        assert_eq!(CleaningMode::Off, result.cleaning_mode);
        assert_eq!(100, result.connections_limit);
    }

    #[test]
    fn test_torrent_settings_from() {
        let directory = "/tmp/lorem/torrent";
        let connections_limit = 200;
        let settings = TorrentSettingsC {
            directory: into_c_string(directory.to_string()),
            cleaning_mode: CleaningMode::Watched,
            connections_limit,
            download_rate_limit: 10,
            upload_rate_limit: 20,
        };
        let expected_result = TorrentSettings {
            directory: PathBuf::from(directory),
            cleaning_mode: CleaningMode::Watched,
            connections_limit,
            download_rate_limit: 10,
            upload_rate_limit: 20,
        };

        let result = TorrentSettings::from(settings);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_from_ui_settings() {
        let language = "en";
        let ui_scale = UiScale::new(1.0).unwrap();
        let settings = UiSettings {
            default_language: language.to_string(),
            ui_scale: ui_scale.clone(),
            start_screen: Category::Movies,
            maximized: true,
            native_window_enabled: false,
        };

        let result = UiSettingsC::from(&settings);

        assert_eq!(language.to_string(), from_c_string(result.default_language));
        assert_eq!(ui_scale, result.ui_scale);
        assert_eq!(Category::Movies, result.start_screen);
        assert_eq!(true, result.maximized);
        assert_eq!(false, result.native_window_enabled);
    }

    #[test]
    fn test_from_ui_settings_c() {
        let ui_scale = UiScale::new(1.0).unwrap();
        let settings = UiSettingsC {
            default_language: into_c_string("en".to_string()),
            ui_scale: ui_scale.clone(),
            start_screen: Category::Series,
            maximized: true,
            native_window_enabled: false,
        };
        let expected_result = UiSettings {
            default_language: "en".to_string(),
            ui_scale,
            start_screen: Category::Series,
            maximized: true,
            native_window_enabled: false,
        };

        let result = UiSettings::from(settings);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_from_server_settings() {
        let api_server = "http://localhost:8080";
        let settings = ServerSettings {
            api_server: Some(api_server.to_string()),
        };

        let result = ServerSettingsC::from(&settings);

        assert_eq!(api_server.to_string(), from_c_string(result.api_server))
    }

    #[test]
    fn test_from_server_settings_none_api_server() {
        let settings = ServerSettings {
            api_server: None,
        };

        let result = ServerSettingsC::from(&settings);

        assert_eq!(ptr::null(), result.api_server)
    }

    #[test]
    fn test_from_server_settings_c() {
        let api_server = "http://localhost:8080";
        let settings = ServerSettingsC {
            api_server: into_c_string(api_server.to_string()),
        };
        let expected_result = ServerSettings {
            api_server: Some(api_server.to_string()),
        };

        let result = ServerSettings::from(settings);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_from_playback_settings() {
        let settings = PlaybackSettings {
            quality: Some(Quality::P1080),
            fullscreen: true,
            auto_play_next_episode_enabled: false,
        };

        let result = PlaybackSettingsC::from(&settings);

        assert_eq!(Quality::P1080, from_c_owned(result.quality));
        assert_eq!(true, result.fullscreen);
        assert_eq!(false, result.auto_play_next_episode_enabled);
    }

    #[test]
    fn test_from_playback_settings_c() {
        let settings = PlaybackSettingsC {
            quality: ptr::null_mut(),
            fullscreen: true,
            auto_play_next_episode_enabled: true,
        };
        let expected_result = PlaybackSettings {
            quality: None,
            fullscreen: true,
            auto_play_next_episode_enabled: true,
        };

        let result = PlaybackSettings::from(settings);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_tracking_settings_c_from() {
        let time = Local::now().with_timezone(&Utc);
        let timestamp = time.timestamp();
        let settings = TrackingSettings::builder()
            .last_sync(LastSync {
                time,
                state: MediaTrackingSyncState::Success,
            })
            .build();

        let result = TrackingSettingsC::from(&settings);
        let last_sync = from_c_owned(result.last_sync);
        
        assert_eq!(timestamp, last_sync.time);
        assert_eq!(MediaTrackingSyncState::Success, last_sync.state);
    }
}