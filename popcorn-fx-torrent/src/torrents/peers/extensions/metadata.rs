use crate::torrents::peers::extensions::Extension;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

pub const EXTENSION_NAME_METADATA: &str = "ut_metadata";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct MetadataExtensionMessage {
    pub piece: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_size: Option<u32>,
    #[serde(
        serialize_with = "serialize_metadata_type",
        deserialize_with = "deserialize_metadata_type"
    )]
    pub msg_type: MetadataMessageType,
}

#[derive(Debug, Clone)]
pub struct MetadataExtension {}

impl Extension for MetadataExtension {
    fn name(&self) -> String {
        EXTENSION_NAME_METADATA.to_string()
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetadataMessageType {
    Request = 0,
    Data = 1,
    Reject = 2,
}

fn serialize_metadata_type<S>(
    message_type: &MetadataMessageType,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u8(message_type.clone() as u8)
}

fn deserialize_metadata_type<'de, D>(deserializer: D) -> Result<MetadataMessageType, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u8::deserialize(deserializer)?;
    match value {
        0 => Ok(MetadataMessageType::Request),
        1 => Ok(MetadataMessageType::Data),
        2 => Ok(MetadataMessageType::Reject),
        _ => Err(de::Error::custom(format!(
            "Invalid message type {} specified",
            value
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let extension = MetadataExtensionMessage {
            piece: 0,
            total_size: None,
            msg_type: MetadataMessageType::Request,
        };
        let expected_result = "d5:piecei0e4:typei0ee";

        let result = serde_bencode::to_string(&extension).unwrap();

        assert_eq!(expected_result, result.as_str());
    }

    #[test]
    fn test_deserialize() {
        let message = "d5:piecei5e4:typei1ee";
        let expected_result = MetadataExtensionMessage {
            piece: 5,
            total_size: None,
            msg_type: MetadataMessageType::Data,
        };

        let result = serde_bencode::from_bytes(message.as_bytes()).unwrap();

        assert_eq!(expected_result, result);
    }
}
