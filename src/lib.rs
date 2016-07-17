#![no_std]

use core::fmt;

#[cfg(feature = "system_time")]
mod system_time;

include!(concat!(env!("OUT_DIR"), "/month_generated.rs"));

/// In seconds since 1970-01-01 00:00:00 UTC.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct UnixTimestamp(pub i64);

pub trait TimeZone {
    fn from_timestamp(&self, t: UnixTimestamp) -> NaiveDateTime;
    fn to_timestamp(&self, d: &NaiveDateTime) -> UnixTimestamp;
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
pub struct Utc;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct DateTime<Tz> {
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

impl<Tz: fmt::Debug> fmt::Debug for DateTime<Tz> {
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

impl<Tz> DateTime<Tz> {
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

    fn days_since_unix(&self) -> i32 {
        (self.year - 1970) * DAYS_PER_COMMON_YEAR
        + leap_days_since_y0(self.year) - leap_days_since_y0(1970)
        + self.month.days_since_january_1st(self.year.into())
        + i32::from(self.day - 1)
    }
}

/// Integer divison that rounds towards negative infinity
// This is a macro in order to work with either i32 or i64.
// Generic integers with traits are a pain.
macro_rules! div_floor {
    ($dividend: expr, $divisor: expr) => {
        {
            let dividend = $dividend;
            let divisor = $divisor;
            if dividend > 0 {
                dividend / divisor
            } else {
                (dividend + 1 - divisor) / divisor
            }
        }
    }
}

/// Remainder within range 0..divisor, even for negative dividend
fn positive_rem(dividend: i64, divisor: i64) -> i64 {
    let rem = dividend % divisor;
    if rem < 0 {
        rem + divisor
    } else {
        rem
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

impl TimeZone for Utc {
    fn from_timestamp(&self, u: UnixTimestamp) -> NaiveDateTime {
        let days_since_unix = div_floor!(u.0, SECONDS_PER_DAY) as i32;
        let days = days_since_unix + days_since_d0(1970);
        let year = div_floor!(days * 400, DAYS_PER_400YEARS) as i32;
        let day_of_the_year = days - days_since_d0(year);
        let (month, day) = Month::from_day_of_the_year(day_of_the_year, year.into());
        let hour = positive_rem(div_floor!(u.0, SECONDS_PER_HOUR), 24) as u8;
        let minute = positive_rem(div_floor!(u.0, SECONDS_PER_MINUTE), 60) as u8;
        let second = positive_rem(u.0, 60) as u8;
        NaiveDateTime::new(year, month, day, hour, minute, second)
    }

    fn to_timestamp(&self, d: &NaiveDateTime) -> UnixTimestamp {
        UnixTimestamp(
            i64::from(d.days_since_unix()) * SECONDS_PER_DAY
            + i64::from(d.hour) * SECONDS_PER_HOUR
            + i64::from(d.minute) * SECONDS_PER_MINUTE
            + i64::from(d.second)
        )
    }
}

/// How many leap days occured between January of year 0 and January of the given year
/// (in Gregorian calendar).
fn leap_days_since_y0(year: i32) -> i32 {
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

#[cfg(test)] #[macro_use] extern crate std;

#[cfg(test)]
mod tests;
