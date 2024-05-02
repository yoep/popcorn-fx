use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

const PAYLOAD_TYPE_LOAD: &str = "LOAD";
const METADATA_TYPE_MOVIE: i16 = 1;
const METADATA_TYPE_TV_SHOW: i16 = 2;

/// Represents a command to load media content on the Chromecast device.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadCommand {
    /// The unique identifier for the request.
    pub request_id: u64,
    /// The session identifier.
    pub session_id: String,
    /// The type of payload.
    #[serde(rename = "type", serialize_with = "serialize_payload_type")]
    pub payload_type: (),
    /// The media content to load.
    pub media: Media,
    /// Indicates whether autoplay is enabled.
    pub autoplay: bool,
    /// The current playback time in milliseconds.
    pub current_time: f32,
    /// The IDs of the active text tracks.
    pub active_track_ids: Option<Vec<u32>>,
}

/// Represents media content to be loaded on the Chromecast device.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    /// The URL of the media content.
    #[serde(rename = "contentId")]
    pub url: String,
    /// The type of stream.
    pub stream_type: StreamType,
    /// The MIME type of the media content.
    pub content_type: String,
    /// The duration of the media content in seconds.
    pub duration: Option<f32>,
    /// The metadata of the media content.
    pub metadata: Option<Metadata>,
    /// Additional custom data associated with the media content.
    pub custom_data: Option<HashMap<String, String>>,
    /// The style settings for text tracks.
    pub text_track_style: Option<TextTrackStyle>,
    /// The tracks associated with the media content.
    /// This can be subtitles associated with the media content.
    pub tracks: Option<Vec<Track>>,
}

/// Represents metadata associated with media content.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Metadata {
    /// Metadata for a movie.
    Movie(MovieMetadata),
    /// Placeholder for TV show metadata.
    TvShow,
}

/// Represents the type of stream.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum StreamType {
    /// No stream type.
    None,
    /// Buffered stream type.
    Buffered,
    /// Live stream type.
    Live,
}

/// Represents metadata for a movie.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovieMetadata {
    /// The type of metadata.
    #[serde(serialize_with = "serialize_movie_metadata_type")]
    pub metadata_type: (),
    /// The title of the movie.
    pub title: Option<String>,
    /// The subtitle of the movie.
    pub subtitle: Option<String>,
    /// The studio producing the movie.
    pub studio: Option<String>,
    /// Images associated with the movie if present, else an empty array.
    /// This field is always required.
    pub images: Vec<Image>,
    /// The release date of the movie.
    pub release_date: Option<String>,
    /// The URL of the movie thumbnail.
    pub thumb: Option<String>,
    /// The URL of the movie thumbnail.
    pub thumbnail_url: Option<String>,
    /// The URL of the movie poster.
    pub poster_url: Option<String>,
}

/// Represents an image associated with media content.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// The URL of the image.
    pub url: String,
    /// The height of the image.
    pub height: Option<i32>,
    /// The width of the image.
    pub width: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub track_id: u64,
    #[serde(rename = "type")]
    pub track_type: TrackType,
    pub track_content_id: String,
    pub track_content_type: String,
    pub subtype: TextTrackType,
    pub language: String,
    pub name: String,
}

/// Possible track types.
/// https://developers.google.com/cast/docs/reference/web_sender/chrome.cast.media#.TrackType
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TrackType {
    Text,
    Audio,
    Video,
}

/// Possible text track types.
/// https://developers.google.com/cast/docs/reference/web_sender/chrome.cast.media#.TextTrackType
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TextTrackType {
    /// Transcription or translation of the dialogue, suitable for when the sound is available but not understood (e.g. because the user does not understand
    /// the language of the media resource's soundtrack).
    Subtitles,
    /// Transcription or translation of the dialogue, sound effects, relevant musical cues, and other relevant audio information, suitable for when the
    /// soundtrack is unavailable (e.g. because it is muted or because the user is deaf). Displayed over the video; labeled as appropriate for the
    /// hard-of-hearing.
    Captions,
    /// Textual descriptions of the video component of the media resource, intended for audio synthesis when the visual component is unavailable (e.g. because
    /// the user is interacting with the application without a screen, or because the user is blind). Synthesized as separate audio track.
    Descriptions,
    /// Chapter titles, intended to be used for navigating the media resource.
    Chapters,
    /// Tracks intended for use from script.
    Metadata,
}

/// The style of the subtitle track.
/// https://developers.google.com/cast/docs/reference/web_sender/chrome.cast.media.TextTrackStyle
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextTrackStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_type: Option<TextTrackEdgeType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_scale: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TextTrackEdgeType {
    None,
    Outline,
    DropShadow,
    Raised,
    Depressed,
}

/// Represents errors that occur during media parsing.
#[derive(Debug, Clone, Error)]
pub enum MediaParseError {
    /// Indicates that a media error code is not supported.
    #[error("media error code {0} is not supported")]
    UnsupportedErrorCode(i32),
}

/// Represents an error encountered during media operations.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaError {
    /// The detailed error code associated with the media error.
    pub detailed_error_code: MediaDetailedErrorCode,
    /// The type of the error message.
    #[serde(rename = "type")]
    pub message_type: String,
}

/// The detailed media error codes.
/// https://developers.google.com/android/reference/com/google/android/gms/cast/MediaError.DetailedErrorCode#constants
#[derive(Debug, Clone, PartialEq)]
pub enum MediaDetailedErrorCode {
    /// An error occurs outside of the framework (e.g., if an event handler throws an error).
    App = 900,
    /// Break clip load interceptor fails.
    BreakClipLoadingError = 901,
    /// Break seek interceptor fails.
    BreakSeekInterceptorError = 902,
    /// A DASH manifest contains invalid segment info.
    DashInvalidSegmentInfo = 423,
    /// A DASH manifest is missing a MimeType.
    DashManifestNoMimeType = 422,
    /// A DASH manifest is missing periods.
    DashManifestNoPeriods = 421,
    /// An unknown error occurs while parsing a DASH manifest.
    DashManifestUnknown = 420,
    /// An unknown network error occurs while handling a DASH stream.
    DashNetwork = 321,
    /// A DASH stream is missing an init.
    DashNoInit = 322,
    /// Returned when an unknown error occurs.
    Generic = 999,
    /// An error occurs while parsing an HLS master manifest.
    HlsManifestMaster = 411,
    /// An error occurs while parsing an HLS playlist.
    HlsManifestPlaylist = 412,
    /// An HLS segment is invalid.
    HlsNetworkInvalidSegment = 315,
    /// A request for an HLS key fails before it is sent.
    HlsNetworkKeyLoad = 314,
    /// An HLS master playlist fails to download.
    HlsNetworkMasterPlaylist = 311,
    /// An HLS key fails to download.
    HlsNetworkNoKeyResponse = 313,
    /// An HLS playlist fails to download.
    HlsNetworkPlaylist = 312,
    /// An HLS segment fails to parse.
    HlsSegmentParsing = 316,
    /// When an image fails to load.
    ImageError = 903,
    /// A load command failed.
    LoadFailed = 905,
    /// A load was interrupted by an unload, or by another load.
    LoadInterrupted = 904,
    /// An unknown error occurs while parsing a manifest.
    ManifestUnknown = 400,
    /// There is a media keys failure due to a network issue.
    MediakeysNetwork = 201,
    /// There is an unknown error with media keys.
    MediakeysUnknown = 200,
    /// A MediaKeySession object cannot be created.
    MediakeysUnsupported = 202,
    /// Crypto failed.
    MediakeysWebcrypto = 203,
    /// The fetching process for the media resource was aborted by the user agent at the user's request.
    MediaAborted = 101,
    /// An error occurred while decoding the media resource, after the resource was established to be usable.
    MediaDecode = 102,
    /// An error message was sent to the sender.
    MediaErrorMessage = 906,
    /// A network error caused the user agent to stop fetching the media resource, after the resource was established to be usable.
    MediaNetwork = 103,
    /// The media resource indicated by the src attribute was not suitable.
    MediaSrcNotSupported = 104,
    /// The HTMLMediaElement throws an error, but CAF does not recognize the specific error.
    MediaUnknown = 100,
    /// There was an unknown network issue.
    NetworkUnknown = 300,
    /// A segment fails to download.
    SegmentNetwork = 301,
    /// An unknown segment error occurs.
    SegmentUnknown = 500,
    /// An error occurs while parsing a Smooth manifest.
    SmoothManifest = 431,
    /// An unknown network error occurs while handling a Smooth stream.
    SmoothNetwork = 331,
    /// A Smooth stream is missing media data.
    SmoothNoMediaData = 332,
    /// A source buffer cannot be added to the MediaSource.
    SourceBufferFailure = 110,
    /// An unknown error occurred with a text stream.
    TextUnknown = 600,
}

impl TryFrom<i32> for MediaDetailedErrorCode {
    type Error = MediaParseError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            900 => Ok(MediaDetailedErrorCode::App),
            901 => Ok(MediaDetailedErrorCode::BreakClipLoadingError),
            902 => Ok(MediaDetailedErrorCode::BreakSeekInterceptorError),
            423 => Ok(MediaDetailedErrorCode::DashInvalidSegmentInfo),
            422 => Ok(MediaDetailedErrorCode::DashManifestNoMimeType),
            421 => Ok(MediaDetailedErrorCode::DashManifestNoPeriods),
            420 => Ok(MediaDetailedErrorCode::DashManifestUnknown),
            321 => Ok(MediaDetailedErrorCode::DashNetwork),
            322 => Ok(MediaDetailedErrorCode::DashNoInit),
            999 => Ok(MediaDetailedErrorCode::Generic),
            411 => Ok(MediaDetailedErrorCode::HlsManifestMaster),
            412 => Ok(MediaDetailedErrorCode::HlsManifestPlaylist),
            315 => Ok(MediaDetailedErrorCode::HlsNetworkInvalidSegment),
            314 => Ok(MediaDetailedErrorCode::HlsNetworkKeyLoad),
            311 => Ok(MediaDetailedErrorCode::HlsNetworkMasterPlaylist),
            313 => Ok(MediaDetailedErrorCode::HlsNetworkNoKeyResponse),
            312 => Ok(MediaDetailedErrorCode::HlsNetworkPlaylist),
            316 => Ok(MediaDetailedErrorCode::HlsSegmentParsing),
            903 => Ok(MediaDetailedErrorCode::ImageError),
            905 => Ok(MediaDetailedErrorCode::LoadFailed),
            904 => Ok(MediaDetailedErrorCode::LoadInterrupted),
            400 => Ok(MediaDetailedErrorCode::ManifestUnknown),
            201 => Ok(MediaDetailedErrorCode::MediakeysNetwork),
            200 => Ok(MediaDetailedErrorCode::MediakeysUnknown),
            202 => Ok(MediaDetailedErrorCode::MediakeysUnsupported),
            203 => Ok(MediaDetailedErrorCode::MediakeysWebcrypto),
            101 => Ok(MediaDetailedErrorCode::MediaAborted),
            102 => Ok(MediaDetailedErrorCode::MediaDecode),
            906 => Ok(MediaDetailedErrorCode::MediaErrorMessage),
            103 => Ok(MediaDetailedErrorCode::MediaNetwork),
            104 => Ok(MediaDetailedErrorCode::MediaSrcNotSupported),
            100 => Ok(MediaDetailedErrorCode::MediaUnknown),
            300 => Ok(MediaDetailedErrorCode::NetworkUnknown),
            301 => Ok(MediaDetailedErrorCode::SegmentNetwork),
            500 => Ok(MediaDetailedErrorCode::SegmentUnknown),
            431 => Ok(MediaDetailedErrorCode::SmoothManifest),
            331 => Ok(MediaDetailedErrorCode::SmoothNetwork),
            332 => Ok(MediaDetailedErrorCode::SmoothNoMediaData),
            110 => Ok(MediaDetailedErrorCode::SourceBufferFailure),
            600 => Ok(MediaDetailedErrorCode::TextUnknown),
            _ => Err(MediaParseError::UnsupportedErrorCode(value)),
        }
    }
}

impl<'de> Deserialize<'de> for MediaDetailedErrorCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> {
        let value: i32 = Deserialize::deserialize(deserializer)?;
        MediaDetailedErrorCode::try_from(value)
            .map_err(|e| serde::de::Error::custom(e))
    }
}

/// Serializes the payload type for the LoadCommand.
fn serialize_payload_type<S: Serializer>(_: &(), serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(PAYLOAD_TYPE_LOAD)
}

/// Serializes the metadata type for movie metadata.
fn serialize_movie_metadata_type<S: Serializer>(_: &(), serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_i16(METADATA_TYPE_MOVIE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_media_error() {
        let expected_result = MediaError {
            message_type: "ERROR".to_string(),
            detailed_error_code: MediaDetailedErrorCode::MediaSrcNotSupported,
        };
        
        let result = serde_json::from_str::<MediaError>("{\"type\":\"ERROR\",\"detailedErrorCode\":104,\"itemId\":1}").unwrap();
        
        assert_eq!(expected_result, result);
    }
}
