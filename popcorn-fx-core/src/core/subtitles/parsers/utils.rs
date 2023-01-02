use chrono::{NaiveDateTime, NaiveTime, Timelike};

pub fn time_to_millis(time: &NaiveTime) -> u64 {
    let hour = time.hour() as u64;
    let minutes = (hour * 60) + (time.minute() as u64);
    let seconds = (minutes * 60) + (time.second() as u64);
    let millis = time.nanosecond() as u64;

    (seconds * 1000) + (millis / 1000000)
}

pub fn time_from_millis(time: u64) -> NaiveTime {
    NaiveDateTime::from_timestamp_millis(time as i64)
        .expect("Time went in the past")
        .time()
}