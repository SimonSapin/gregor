#![no_std]

#[cfg(any(test, feature = "system_time"))] #[macro_use] extern crate std;

#[cfg(feature = "system_time")] mod system_time;
#[cfg(test)] mod tests;
mod time_zones;

pub use time_zones::{TimeZone, Utc, FixedOffsetFromUtc};
use core::fmt;

include!(concat!(env!("OUT_DIR"), "/month_generated.rs"));

/// In seconds since 1970-01-01 00:00:00 UTC.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct UnixTimestamp(pub i64);

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct DateTime<Tz: TimeZone> {
    pub naive: NaiveDateTime,
    pub time_zone: Tz,
}

/// A date and time without associated time zone information.
#[derive(Eq, PartialEq, Copy, Clone)]
pub struct NaiveDateTime {
    /// Year number per ISO 8601.
    ///
    /// For example, 2016 AC is +2016, 1 AC is +1, 1 BC is 0, 2 BC is -1, etc.
    pub year: i32,

    pub month: Month,

    /// 1st of the month is day 1
    pub day: u8,

    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl<Tz: fmt::Debug + TimeZone> fmt::Debug for DateTime<Tz> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "DateTime({:?}, {:?})", self.time_zone, self.naive)
    }
}

impl fmt::Debug for NaiveDateTime {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
               self.year, self.month as u8, self.day,
               self.hour, self.minute, self.second)
    }
}

impl<Tz: TimeZone> DateTime<Tz> {
    pub fn new(time_zone: Tz, year: i32, month: Month, day: u8,
               hour: u8, minute: u8, second: u8)
               -> Self {
        DateTime {
            naive: NaiveDateTime::new(year, month, day, hour, minute, second),
            time_zone: time_zone,
        }
    }

    pub fn year(&self) -> i32 { self.naive.year }
    pub fn month(&self) -> Month { self.naive.month }
    pub fn day(&self) -> u8 { self.naive.day }
    pub fn hour(&self) -> u8 { self.naive.hour }
    pub fn minute(&self) -> u8 { self.naive.minute }
    pub fn second(&self) -> u8 { self.naive.second }

    pub fn from_timestamp(t: UnixTimestamp, time_zone: Tz) -> Self {
        DateTime {
            naive: time_zone.from_timestamp(t),
            time_zone: time_zone,
        }
    }
}

impl NaiveDateTime {
    pub fn new(year: i32, month: Month, day: u8, hour: u8, minute: u8, second: u8) -> Self {
        NaiveDateTime {
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second,
        }
    }
}

impl<Tz: Default + TimeZone> From<UnixTimestamp> for DateTime<Tz> {
    fn from(u: UnixTimestamp) -> Self {
        let tz = Tz::default();
        DateTime {
            naive: tz.from_timestamp(u),
            time_zone: tz,
        }
    }
}

impl<Tz: TimeZone> From<DateTime<Tz>> for UnixTimestamp {
    fn from(datetime: DateTime<Tz>) -> Self {
        datetime.time_zone.to_timestamp(&datetime.naive)
    }
}


#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum YearKind {
    Common,
    Leap,
}

impl From<i32> for YearKind {
    fn from(year: i32) -> Self {
        fn is_multiple(n: i32, divisor: i32) -> bool {
            n % divisor == 0
        }

        if is_multiple(year, 4) && (!is_multiple(year, 100) || is_multiple(year, 400)) {
            YearKind::Leap
        } else {
            YearKind::Common
        }
    }
}
