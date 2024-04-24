use std::collections::HashMap;

use serde::{Serialize, Serializer};

const PAYLOAD_TYPE_LOAD: &str = "LOAD";
const METADATA_TYPE_MOVIE: i16 = 1;
const METADATA_TYPE_TV_SHOW: i16 = 2;

/// Represents a command to load media content on the Chromecast device.
#[derive(Debug, Serialize)]
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
#[derive(Debug, Serialize)]
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
    pub text_track_style: Option<HashMap<String, String>>,
}

/// Represents metadata associated with media content.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Metadata {
    /// Metadata for a movie.
    Movie(MovieMetadata),
    /// Placeholder for TV show metadata.
    TvShow,
}

/// Represents the type of stream.
#[derive(Debug, Serialize)]
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
#[derive(Debug, Serialize)]
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
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// The URL of the image.
    pub url: String,
    /// The height of the image.
    pub height: Option<i32>,
    /// The width of the image.
    pub width: Option<i32>,
}

/// Serializes the payload type for the LoadCommand.
fn serialize_payload_type<S: Serializer>(_: &(), serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(PAYLOAD_TYPE_LOAD)
}

/// Serializes the metadata type for movie metadata.
fn serialize_movie_metadata_type<S: Serializer>(_: &(), serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_i16(METADATA_TYPE_MOVIE)
}
