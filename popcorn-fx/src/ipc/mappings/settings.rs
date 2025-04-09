use crate::ipc::protobuf::settings::application_settings::{
    subtitle_settings, uisettings, PlaybackSettings, ServerSettings, SubtitleSettings,
    TorrentSettings, TrackingSettings, UISettings,
};
use crate::ipc::protobuf::settings::ApplicationSettings;
use crate::ipc::protobuf::subtitle::subtitle::Language;
use popcorn_fx_core::core::config::{DecorationType, PopcornSettings, SubtitleFamily};
use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
use protobuf::{EnumOrUnknown, MessageField};

impl From<&PopcornSettings> for ApplicationSettings {
    fn from(value: &PopcornSettings) -> Self {
        let mut settings = Self::new();

        let mut subtitle = SubtitleSettings::new();
        subtitle.directory = value.subtitle_settings.directory.clone();
        subtitle.auto_cleaning_enabled = value.subtitle_settings.auto_cleaning_enabled;
        subtitle.default_subtitle =
            EnumOrUnknown::from(Language::from(&value.subtitle_settings.default_subtitle));
        subtitle.font_family = EnumOrUnknown::from(subtitle_settings::SubtitleFamily::from(
            &value.subtitle_settings.font_family,
        ));
        subtitle.font_size = value.subtitle_settings.font_size as i32;
        subtitle.decoration = EnumOrUnknown::from(subtitle_settings::DecorationType::from(
            &value.subtitle_settings.decoration,
        ));
        subtitle.bold = value.subtitle_settings.bold;

        let mut torrent = TorrentSettings::new();

        let mut ui = UISettings::new();
        ui.default_language = value.ui_settings.default_language.clone();
        ui.scale = value.ui_settings.ui_scale.value();
        ui.start_screen =
            EnumOrUnknown::from(uisettings::Category::from(&value.ui_settings.start_screen));
        ui.maximized = value.ui_settings.maximized;
        ui.native_window_enabled = value.ui_settings.native_window_enabled;

        let mut server = ServerSettings::new();

        let mut playback = PlaybackSettings::new();

        let mut tracking = TrackingSettings::new();

        settings.subtitle = MessageField::some(subtitle);
        settings.ui = MessageField::some(ui);
        settings.server = MessageField::some(server);
        settings.torrent = MessageField::some(torrent);
        settings.playback = MessageField::some(playback);
        settings.tracking = MessageField::some(tracking);

        settings
    }
}

impl From<&SubtitleLanguage> for Language {
    fn from(value: &SubtitleLanguage) -> Self {
        match value {
            SubtitleLanguage::None => Language::NONE,
            SubtitleLanguage::Custom => Language::CUSTOM,
            SubtitleLanguage::Arabic => Language::ARABIC,
            SubtitleLanguage::Bulgarian => Language::BULGARIAN,
            SubtitleLanguage::Bosnian => Language::BOSNIAN,
            SubtitleLanguage::Czech => Language::CZECH,
            SubtitleLanguage::Danish => Language::DANISH,
            SubtitleLanguage::German => Language::GERMAN,
            SubtitleLanguage::ModernGreek => Language::MODERN_GREEK,
            SubtitleLanguage::English => Language::ENGLISH,
            SubtitleLanguage::Spanish => Language::SPANISH,
            SubtitleLanguage::Estonian => Language::ESTONIAN,
            SubtitleLanguage::Basque => Language::BASQUE,
            SubtitleLanguage::Persian => Language::PERSIAN,
            SubtitleLanguage::Finnish => Language::FINNISH,
            SubtitleLanguage::French => Language::FRENCH,
            SubtitleLanguage::Hebrew => Language::HEBREW,
            SubtitleLanguage::Croatian => Language::CROATIAN,
            SubtitleLanguage::Hungarian => Language::HUNGARIAN,
            SubtitleLanguage::Indonesian => Language::INDONESIAN,
            SubtitleLanguage::Italian => Language::ITALIAN,
            SubtitleLanguage::Lithuanian => Language::LITHUANIAN,
            SubtitleLanguage::Dutch => Language::DUTCH,
            SubtitleLanguage::Norwegian => Language::NORWEGIAN,
            SubtitleLanguage::Polish => Language::POLISH,
            SubtitleLanguage::Portuguese => Language::PORTUGUESE,
            SubtitleLanguage::PortugueseBrazil => Language::PORTUGUESE_BRAZIL,
            SubtitleLanguage::Romanian => Language::ROMANIAN,
            SubtitleLanguage::Russian => Language::RUSSIAN,
            SubtitleLanguage::Slovene => Language::SLOVENE,
            SubtitleLanguage::Serbian => Language::SERBIAN,
            SubtitleLanguage::Swedish => Language::SWEDISH,
            SubtitleLanguage::Thai => Language::THAI,
            SubtitleLanguage::Turkish => Language::TURKISH,
            SubtitleLanguage::Ukrainian => Language::UKRAINIAN,
            SubtitleLanguage::Vietnamese => Language::VIETNAMESE,
        }
    }
}

impl From<&SubtitleFamily> for subtitle_settings::SubtitleFamily {
    fn from(value: &SubtitleFamily) -> Self {
        match value {
            SubtitleFamily::Arial => subtitle_settings::SubtitleFamily::ARIAL,
            SubtitleFamily::ComicSans => subtitle_settings::SubtitleFamily::COMIC_SANS,
            SubtitleFamily::Georgia => subtitle_settings::SubtitleFamily::GEORGIA,
            SubtitleFamily::Tahoma => subtitle_settings::SubtitleFamily::TAHOMA,
            SubtitleFamily::TrebuchetMs => subtitle_settings::SubtitleFamily::TREBUCHET_MS,
            SubtitleFamily::Verdana => subtitle_settings::SubtitleFamily::VERDANA,
        }
    }
}

impl From<&DecorationType> for subtitle_settings::DecorationType {
    fn from(value: &DecorationType) -> Self {
        match value {
            DecorationType::None => subtitle_settings::DecorationType::NONE,
            DecorationType::Outline => subtitle_settings::DecorationType::OUTLINE,
            DecorationType::OpaqueBackground => {
                subtitle_settings::DecorationType::OPAQUE_BACKGROUND
            }
            DecorationType::SeeThroughBackground => {
                subtitle_settings::DecorationType::SEE_THROUGH_BACKGROUND
            }
        }
    }
}
