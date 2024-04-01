use chrono::{DateTime, NaiveTime, Timelike};

/// Converts a `NaiveTime` into milliseconds since midnight.
///
/// # Arguments
///
/// * `time` - A reference to the `NaiveTime` to convert.
///
/// # Returns
///
/// Returns the number of milliseconds since midnight represented by the given `NaiveTime`.
pub fn time_to_millis(time: &NaiveTime) -> u64 {
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
pub fn time_from_millis(time: u64) -> NaiveTime {
    DateTime::from_timestamp_millis(time as i64)
        .expect("Time went in the past")
        .time()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_to_millis() {
        let time = NaiveTime::from_hms_opt(1, 20, 45).unwrap();

        let result = time_to_millis(&time);
        
        assert_eq!(4845000, result);
    }
    
    #[test]
    fn test_time_from_millis() {
        let expected_time = NaiveTime::from_hms_opt(1, 20, 45).unwrap();
        let time = time_to_millis(&expected_time);

        let result = time_from_millis(time);
        
        assert_eq!(expected_time, result);
    }
}