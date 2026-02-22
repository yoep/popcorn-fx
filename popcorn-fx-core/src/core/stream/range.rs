use crate::core::stream::{Error, Result};
use std::fmt::{Display, Formatter};

const BYTES_PREFIX: &str = "bytes=";
const BYTES_LEN: usize = BYTES_PREFIX.len();

/// The HTTP range information according to rfc7233.
/// The requested range only allows for `bytes` type, any other types will result in an [Err].
#[derive(Debug, Clone)]
pub struct Range {
    pub start: u64,
    pub end: Option<u64>,
}

impl Range {
    pub fn parse(value: &str) -> Result<Vec<Self>> {
        if !value.starts_with(BYTES_PREFIX) {
            return Err(Error::InvalidRange);
        }

        let range_value = &value[BYTES_LEN..];
        let mut ranges = vec![];

        for value in range_value.split(",") {
            ranges.push(Self::parse_value(value)?);
        }

        Ok(ranges)
    }

    fn parse_value(value: &str) -> Result<Self> {
        let values: Vec<&str> = value.split("-").collect();
        let start = values[0]
            .parse::<u64>()
            .map_err(|e| Error::Parse(e.to_string()))?;
        let end_value = values[1];
        let mut end = None;

        if !end_value.is_empty() {
            end = Some(
                end_value
                    .parse::<u64>()
                    .map_err(|e| Error::Parse(e.to_string()))?,
            );
        }

        Ok(Self { start, end })
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}",
            self.start,
            self.end
                .map(|e| e.to_string())
                .or_else(|| Some("".to_string()))
                .unwrap()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let value = "bytes=0-1023";

        let ranges = Range::parse(value).expect("expected a valid range");
        let range = ranges.first().expect("expected 1 range");

        assert_eq!(0, range.start);
        assert_eq!(1023, range.end.unwrap())
    }

    #[test]
    fn test_parse_invalid_prefix() {
        let value = "kb=0-1485";

        let ranges = Range::parse(value);

        assert!(ranges.is_err(), "expected an error to be returned");
        match ranges.err().unwrap() {
            Error::InvalidRange => {}
            _ => assert!(false, "expected the RangeError::InvalidValue"),
        }
    }

    #[test]
    fn test_parse_invalid_start_value() {
        let value = "bytes=lorem-1023";

        let ranges = Range::parse(value);

        assert!(ranges.is_err(), "expected an error to have been returned");
        match ranges.err().unwrap() {
            Error::Parse(_) => {}
            _ => assert!(false, "expected the RangeError::Parse"),
        }
    }

    #[test]
    fn test_parse_invalid_end_value() {
        let value = "bytes=10-lorem";

        let ranges = Range::parse(value);

        assert!(ranges.is_err(), "expected an error to have been returned");
        match ranges.err().unwrap() {
            Error::Parse(_) => {}
            _ => assert!(false, "expected the RangeError::Parse"),
        }
    }

    #[test]
    fn test_parse_no_end_value() {
        let value = "bytes=0-";

        let ranges = Range::parse(value).expect("expected a valid range");
        let range = ranges.first().expect("expected 1 range");

        assert_eq!(None, range.end);
    }
}
