use std::time::{Duration as StdDuration, SystemTime, UNIX_EPOCH};
use super::{UnixTimestamp, DateTime, TimeZone, UnambiguousTimeZone};

impl From<SystemTime> for UnixTimestamp {
    fn from(t: SystemTime) -> Self {
        UnixTimestamp(match t.duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs() as i64,
            Err(error) => -(error.duration().as_secs() as i64)
        })
    }
}

impl From<UnixTimestamp> for SystemTime {
    fn from(t: UnixTimestamp) -> Self {
        if t.0 >= 0 {
            UNIX_EPOCH + StdDuration::from_secs(t.0 as u64)
        } else {
            UNIX_EPOCH - StdDuration::from_secs((-t.0) as u64)
        }
    }
}

impl<Tz: Default + TimeZone> From<SystemTime> for DateTime<Tz> {
    fn from(t: SystemTime) -> Self {
        UnixTimestamp::from(t).into()
    }
}

impl<Tz: UnambiguousTimeZone> From<DateTime<Tz>> for SystemTime {
    fn from(d: DateTime<Tz>) -> Self {
        UnixTimestamp::from(d).into()
    }
}
