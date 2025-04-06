use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::Path;

use log::{debug, error};
use thiserror::Error;
use warp::http::HeaderValue;

/// The default known mime types supported by HTTPD.
/// More info: https://svn.apache.org/viewvc/httpd/httpd/trunk/docs/conf/mime.types
const MIME_TYPES: &str = include_str!("../../../../resources/mime.types");

/// The media type result.
pub type MediaTypeResult<T> = Result<T, MediaTypeError>;

/// The media type specific errors that can occur.
#[derive(Debug, Clone, Error)]
pub enum MediaTypeError {
    #[error("Value \"{0}\" is not a valid media type")]
    InvalidMediaType(String),
    #[error("Filename \"{0}\" is invalid and cannot be converted to a media type")]
    InvalidFile(String),
    #[error("Media type couldn't be found for extension {0}")]
    NotFound(String),
}

/// Represent a MIME type, as originally defined in RFC 2046
/// and subsequently used in other Internet protocols including HTTP.
#[derive(Debug, Clone)]
pub struct MediaType {
    mime_type: String,
    subtype: String,
}

impl MediaType {
    /// Parse the given value into a media type.
    /// This will verify if the given value is valid or not.
    ///
    /// Example:
    /// ```rust
    /// use popcorn_fx_core::core::torrents::stream::MediaType;
    ///
    /// let value = "application/ecmascript";
    /// let media_type = MediaType::parse(value).unwrap();
    /// ```
    pub fn parse(value: &str) -> MediaTypeResult<Self> {
        let tokens = value
            .split("/")
            .filter(|e| !e.is_empty())
            .collect::<Vec<&str>>();
        if tokens.len() != 2 {
            return Err(MediaTypeError::InvalidMediaType(value.to_string()));
        }

        Ok(Self {
            mime_type: tokens[0].to_string(),
            subtype: tokens[1].to_string(),
        })
    }

    /// Retrieve the octet-stream media type.
    pub fn octet_stream() -> Self {
        Self {
            mime_type: "application".to_string(),
            subtype: "octet-stream".to_string(),
        }
    }
}

impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.mime_type, self.subtype)
    }
}

impl From<MediaType> for HeaderValue {
    fn from(value: MediaType) -> Self {
        HeaderValue::from_str(value.to_string().as_str()).unwrap()
    }
}

/// The media type factory which can convert file extensions into [MediaType].
#[derive(Debug)]
pub struct MediaTypeFactory {
    /// The extension to media type mapping
    media_types: HashMap<String, MediaType>,
}

impl MediaTypeFactory {
    /// Retrieve the media type for the given filename.
    /// The mime type will be determined based on the extension of the file.
    ///
    /// It returns the [MediaType] if found, else the [MediaTypeError].
    pub fn media_type(&self, filename: &str) -> MediaTypeResult<MediaType> {
        let filename = filename.to_lowercase();

        match Path::new(&filename).extension().and_then(|e| e.to_str()) {
            Some(extension) => match self.media_types.get(extension) {
                None => Err(MediaTypeError::NotFound(extension.to_string())),
                Some(e) => Ok(e.clone()),
            },
            None => {
                debug!("Unable to extract extension from {}", &filename);
                Err(MediaTypeError::InvalidFile(filename))
            }
        }
    }
}

impl Default for MediaTypeFactory {
    fn default() -> Self {
        let mut media_types = HashMap::new();

        for line in MIME_TYPES.lines() {
            if line.is_empty() || &line[0..1] == "#" {
                continue;
            }

            let tokens = line
                .split_terminator(&['\t', ' '][..])
                .filter(|e| !e.is_empty())
                .collect::<Vec<&str>>();

            match MediaType::parse(tokens[0]) {
                Ok(media_type) => {
                    for i in 1..tokens.len() {
                        let extension = tokens[i].to_lowercase();
                        media_types.insert(extension, media_type.clone());
                    }
                }
                Err(e) => error!("Failed to parse mime type line {}, {}", line, e),
            }
        }

        debug!(
            "Media type factory has a total of {} known mime types",
            media_types.len()
        );
        Self { media_types }
    }
}

#[cfg(test)]
mod test {
    use crate::init_logger;

    use super::*;

    #[test]
    fn test_media_factory_media_type() {
        init_logger!();
        let filename = "video.mp4";
        let factory = MediaTypeFactory::default();

        let result = factory
            .media_type(filename)
            .expect("expected a mime type to be returned");

        assert_eq!("video/mp4".to_string(), result.to_string())
    }

    #[test]
    fn test_media_factory_media_type_not_found() {
        init_logger!();
        let filename = "ipsum.lorem";
        let factory = MediaTypeFactory::default();

        let result = factory.media_type(filename);

        assert!(result.is_err(), "expected an error to be returned");
        match result.err().unwrap() {
            MediaTypeError::NotFound(_) => {}
            _ => assert!(
                false,
                "expected MediaTypeError::NotFound to have been returned"
            ),
        }
    }

    #[test]
    fn test_media_factory_media_type_no_extension() {
        init_logger!();
        let filename = "my-file";
        let factory = MediaTypeFactory::default();

        let result = factory.media_type(filename);

        assert!(result.is_err(), "expected an error to be returned");
        match result.err().unwrap() {
            MediaTypeError::InvalidFile(_) => {}
            _ => assert!(
                false,
                "expected MediaTypeError::InvalidFile to have been returned"
            ),
        }
    }

    #[test]
    fn test_media_type_parse_invalid_value() {
        let result = MediaType::parse("lorem ipsum");

        assert!(result.is_err(), "expected an error to be returned");
        match result.err().unwrap() {
            MediaTypeError::InvalidMediaType(_) => {}
            _ => assert!(
                false,
                "expected MediaTypeError::InvalidMediaType to have been returned"
            ),
        }
    }
}
