use std::collections::HashMap;

use serde::{Serialize, Serializer};

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
    Video
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

/// Serializes the payload type for the LoadCommand.
fn serialize_payload_type<S: Serializer>(_: &(), serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(PAYLOAD_TYPE_LOAD)
}

/// Serializes the metadata type for movie metadata.
fn serialize_movie_metadata_type<S: Serializer>(_: &(), serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_i16(METADATA_TYPE_MOVIE)
}
