use log::error;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get the current system time as UNIX timestamp in micro seconds.
pub fn now_as_micros() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|e| e.as_micros() as u32)
        .map_err(|e| {
            error!(
                "Unable to get current system time, invalid system time, {}",
                e
            );
            e
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_as_micros() {
        let result = now_as_micros();
        assert_ne!(0, result, "expected the current timestamp in micros");
    }
}
