use super::{NaiveDateTime, UnixTimestamp, Month};
use num::{div_floor, positive_rem};

pub trait TimeZone {
    fn from_timestamp(&self, t: UnixTimestamp) -> NaiveDateTime;
    fn to_timestamp(&self, d: &NaiveDateTime) -> UnixTimestamp;
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
pub struct Utc;

/// The offset is typically positive east of Greenwhich (longitude 0°), negative west.
///
/// For example, Japan Standard Time is UTC+09:00:
///
/// ```rust
/// use gregor::FixedOffsetFromUtc;
/// let jst = FixedOffsetFromUtc::from_hours_and_minutes(9, 0);
/// ```
#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
pub struct FixedOffsetFromUtc {
    seconds_ahead_of_utc: i32,
}

impl FixedOffsetFromUtc {
    pub fn from_hours_and_minutes(hours: i32, minutes: i32) -> Self {
        FixedOffsetFromUtc {
            seconds_ahead_of_utc: (hours * 60 + minutes) * 60,
        }
    }
}

impl TimeZone for FixedOffsetFromUtc {
    fn from_timestamp(&self, u: UnixTimestamp) -> NaiveDateTime {
        Utc.from_timestamp(UnixTimestamp(u.0 + i64::from(self.seconds_ahead_of_utc)))
    }

    fn to_timestamp(&self, d: &NaiveDateTime) -> UnixTimestamp {
        UnixTimestamp(Utc.to_timestamp(d).0 - i64::from(self.seconds_ahead_of_utc))
    }
}

impl TimeZone for Utc {
    fn from_timestamp(&self, u: UnixTimestamp) -> NaiveDateTime {
        let days_since_unix = div_floor(u.0, SECONDS_PER_DAY) as i32;
        let days = days_since_unix + days_since_d0(1970);
        let year = div_floor(days * 400, DAYS_PER_400YEARS) as i32;
        let day_of_the_year = days - days_since_d0(year);
        let (month, day) = Month::from_day_of_the_year(day_of_the_year, year.into());
        let hour = positive_rem(div_floor(u.0, SECONDS_PER_HOUR), 24) as u8;
        let minute = positive_rem(div_floor(u.0, SECONDS_PER_MINUTE), 60) as u8;
        let second = positive_rem(u.0, 60) as u8;
        NaiveDateTime::new(year, month, day, hour, minute, second)
    }

    fn to_timestamp(&self, d: &NaiveDateTime) -> UnixTimestamp {
        UnixTimestamp(
            i64::from(days_since_unix(d)) * SECONDS_PER_DAY
            + i64::from(d.hour) * SECONDS_PER_HOUR
            + i64::from(d.minute) * SECONDS_PER_MINUTE
            + i64::from(d.second)
        )
    }
}

pub fn days_since_unix(d: &NaiveDateTime) -> i32 {
    (d.year - 1970) * DAYS_PER_COMMON_YEAR
    + leap_days_since_y0(d.year) - leap_days_since_y0(1970)
    + d.month.days_since_january_1st(d.year.into())
    + i32::from(d.day - 1)
}

/// How many leap days occured between January of year 0 and January of the given year
/// (in Gregorian calendar).
pub fn leap_days_since_y0(year: i32) -> i32 {
    if year > 0 {
        let year = year - 1;  // Don’t include Feb 29 of the given year, if any.
        // +1 because year 0 is a leap year.
        ((year / 4) - (year / 100) + (year / 400)) + 1
    } else {
        let year = -year;
        -((year / 4) - (year / 100) + (year / 400))
    }
}

/// Days between January 1st of year 0 and January 1st of the given year.
fn days_since_d0(year: i32) -> i32 {
    year * DAYS_PER_COMMON_YEAR + leap_days_since_y0(year)
}

const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = SECONDS_PER_MINUTE * 60;
const SECONDS_PER_DAY: i64 = SECONDS_PER_HOUR * 24;

/// The leap year schedule of the Gregorian calendar cycles every 400 years.
/// In one cycle, there are:
///
/// * 100 years multiple of 4
/// * 4 years multiple of 100
/// * 1 year multiple of 400
const LEAP_DAYS_PER_400YEARS: i32 = 100 - 4 + 1;

const DAYS_PER_COMMON_YEAR: i32 = 365;
const DAYS_PER_400YEARS: i32 = DAYS_PER_COMMON_YEAR * 400 + LEAP_DAYS_PER_400YEARS;
