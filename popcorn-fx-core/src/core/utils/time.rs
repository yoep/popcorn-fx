use chrono::{DateTime, NaiveTime, Timelike};
use derive_more::Display;
use log::error;
use thiserror::Error;

#[derive(Debug, Display, Error, PartialEq)]
pub struct ParseTimeError {}

/// Converts a `NaiveTime` into milliseconds since midnight.
///
/// # Arguments
///
/// * `time` - A reference to the `NaiveTime` to convert.
///
/// # Returns
///
/// Returns the number of milliseconds since midnight represented by the given `NaiveTime`.
pub fn parse_millis_from_time(time: &NaiveTime) -> u64 {
    let hour = time.hour() as u64;
    let minutes = (hour * 60) + (time.minute() as u64);
    let seconds = (minutes * 60) + (time.second() as u64);
    let millis = time.nanosecond() as u64;

    (seconds * 1000) + (millis / 1_000_000)
}

/// Converts milliseconds since midnight into a `NaiveTime`.
///
/// # Arguments
///
/// * `time` - The number of milliseconds since midnight.
///
/// # Returns
///
/// Returns the corresponding `NaiveTime`.
pub fn parse_time_from_millis(time: u64) -> NaiveTime {
    DateTime::from_timestamp_millis(time as i64)
        .expect("Time went in the past")
        .time()
}

/// Parses a time value in the format HH:MM:SS.
///
/// # Arguments
///
/// * `time` - A string slice containing the time value to parse.
///
/// # Returns
///
/// Returns a `NaiveTime` instance if parsing is successful, or a `ParseTimeError` if parsing fails.
pub fn parse_time_from_str(time: &str) -> Result<NaiveTime, ParseTimeError> {
    NaiveTime::parse_from_str(time, "%H:%M:%S")
        .map_err(|e| {
            error!("Failed to parse time: {}", e);
            ParseTimeError {}
        })
}

/// Converts a `NaiveTime` into a string representation in the format "HH:MM:SS".
///
/// # Arguments
///
/// * `time` - A reference to the `NaiveTime` to be converted into a string.
///
/// # Returns
///
/// A string representing the time in the format "HH:MM:SS".
pub fn parse_str_from_time(time: &NaiveTime) -> String {
    time.format("%H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_millis_from_time() {
        let time = NaiveTime::from_hms_opt(1, 20, 45).unwrap();

        let result = parse_millis_from_time(&time);

        assert_eq!(4845000, result);
    }

    #[test]
    fn test_parse_time_from_millis() {
        let expected_time = NaiveTime::from_hms_opt(1, 20, 45).unwrap();
        let time = parse_millis_from_time(&expected_time);

        let result = parse_time_from_millis(time);

        assert_eq!(expected_time, result);
    }

    #[test]
    fn test_parse_time() {
        let expected_result = NaiveTime::from_hms_opt(0, 19, 26).unwrap();

        let result = parse_time_from_str("00:19:26").unwrap();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_parse_time_invalid() {
        let result = parse_time_from_str("00:19");

        assert_eq!(Err(ParseTimeError{}), result);
    }
    
    #[test]
    fn test_parse_str_from_time() {
        let time = NaiveTime::from_hms_opt(1, 20, 45).unwrap();
        let expected_result = "01:20:45";
        
        let result = parse_str_from_time(&time);
        
        assert_eq!(expected_result, result.as_str());
    }
}