use core::fmt;
use super::{NaiveDateTime, DateTime, UnixTimestamp, Month, DayOfTheWeek};
use num::{div_floor, positive_rem};

pub trait TimeZone {
    fn from_timestamp(&self, t: UnixTimestamp) -> NaiveDateTime;
    fn to_timestamp(&self, d: &NaiveDateTime) -> Result<UnixTimestamp, LocalTimeConversionError>;
}

/// When a time zone makes clock jump forward or back at any instant in time
/// (for example twice a year with daylight-saving time, a.k.a. summer-time period)
/// This error is returned when either:
///
/// * Clocks went back and this local time occurred at multiple instants in time,
///   making its interpretation or conversion ambiguous.
///
/// * Clocks jumped forward and this local time did not occur.
///   It does not represent any real instant in time.
///   It could be argued that a range of local times all represent the same instant,
///   but this library does not implement the conversion that way.
#[derive(Eq, PartialEq)]
pub struct LocalTimeConversionError {
    /// Make the type opaque to allow for future extensions
    _private: (),
}

impl fmt::Debug for LocalTimeConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "LocalTimeConversionError")
    }
}

/// Implemented for time zones where `LocalTimeConversionError` never occurs,
/// namely for `Utc` and `FixedOffsetFromUtc`.
///
/// Any UTC-offset change in a time zone creates local times that either don’t occur or occur twice.
/// `TimeZone::to_timestamp` returns `Err(LocalTimeConversionError)` for such local times.
pub trait UnambiguousTimeZone: TimeZone {
    fn to_unambiguous_timestamp(&self, d: &NaiveDateTime) -> UnixTimestamp {
        self.to_timestamp(d).unwrap()
    }
}

/// The *Coordinated Universal Time* time time zone.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
pub struct Utc;

impl UnambiguousTimeZone for Utc {}

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

    fn to_timestamp(&self, d: &NaiveDateTime) -> Result<UnixTimestamp, LocalTimeConversionError> {
        Ok(UnixTimestamp(
            i64::from(days_since_unix(d)) * SECONDS_PER_DAY
            + i64::from(d.hour) * SECONDS_PER_HOUR
            + i64::from(d.minute) * SECONDS_PER_MINUTE
            + i64::from(d.second)
        ))
    }
}

/// The offset is typically positive east of Greenwich (longitude 0°), negative west.
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

impl UnambiguousTimeZone for FixedOffsetFromUtc {}

impl TimeZone for FixedOffsetFromUtc {
    fn from_timestamp(&self, u: UnixTimestamp) -> NaiveDateTime {
        // When local time is ahead of UTC (positive offset)
        // that instant happened before midnight UTC
        // so there are more seconds since then.
        // (Add the offset rather than subtract it.)

        // Seconds since *this time zone*’s midnight of 1970-01-01.
        let seconds = u.0 + i64::from(self.seconds_ahead_of_utc);

        // This is not really a Unix timestamp or a UTC date-time,
        // but the two errors compensate to give a date-time in this time zone.
        Utc.from_timestamp(UnixTimestamp(seconds))
    }

    fn to_timestamp(&self, d: &NaiveDateTime) -> Result<UnixTimestamp, LocalTimeConversionError> {
        // Pretend this is UTC to obtain seconds since *this time zone*’s midnight of 1970-01-01.
        let seconds = Utc.to_unambiguous_timestamp(d).0;

        // For positives offsets (ahead of UTC) this is earlier in time than UTC midnight
        // (with more seconds), so *subtract* the offset to make a Unix timestamp.
        Ok(UnixTimestamp(seconds - i64::from(self.seconds_ahead_of_utc)))
    }
}

pub trait DaylightSaving {
    fn offset_outside_dst(&self) -> FixedOffsetFromUtc;
    fn offset_during_dst(&self) -> FixedOffsetFromUtc;
    fn is_in_dst(&self, t: UnixTimestamp) -> bool;
}

impl<Tz: DaylightSaving> TimeZone for Tz {
    fn from_timestamp(&self, u: UnixTimestamp) -> NaiveDateTime {
        let offset = if self.is_in_dst(u) {
            self.offset_during_dst()
        } else {
            self.offset_outside_dst()
        };
        offset.from_timestamp(u)
    }

    fn to_timestamp(&self, d: &NaiveDateTime) -> Result<UnixTimestamp, LocalTimeConversionError> {
        // The actual timestamp/instant is one of these two:
        let assuming_outside = self.offset_outside_dst().to_unambiguous_timestamp(d);
        let assuming_during = self.offset_during_dst().to_unambiguous_timestamp(d);

        // Let’s take Central Europe for example.
        // When converted to UTC, `assuming_outside` and `assuming_during` respectively
        // represent date-times one hour and two hours before `d`.
        // They are one hour apart.
        //
        // If both timestamps are in the same DST period (during DST or outside)
        // then we know for sure which of `assuming_outside` or `assuming_during` is correct.
        //
        // If they disagree, that means their one hour span contains a DST change:
        //
        // * 1 am UTC is between `d - 2 hours` and `d - 1 hour`
        // * `d - 2 hours` < 1am UTC, and 1am UTC <= `d - 1 hour`
        // * `d` < 3 am local time, and 2 am local time <= `d`
        // * `d` is between 2 am and 3 am local time.
        //
        // * In October when clocks go "back", this kind of local time happens twice the same day:
        //   it’s ambiguous.
        // * In March when clocks go "forward", that hour is skipped entirely.
        //   This kind of local time does not exist. This `d` value might come from buggy code.
        match (self.is_in_dst(assuming_outside), self.is_in_dst(assuming_during)) {
            (true, true) => Ok(assuming_during),
            (false, false) => Ok(assuming_outside),
            _ => Err(LocalTimeConversionError { _private: () }),
        }
    }
}

/// CET (Central European Time) / CEST (Central European Summer Time)
#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
pub struct CentralEurope;

impl DaylightSaving for CentralEurope {
    fn offset_outside_dst(&self) -> FixedOffsetFromUtc {
        FixedOffsetFromUtc::from_hours_and_minutes(1, 0)
    }

    fn offset_during_dst(&self) -> FixedOffsetFromUtc {
        FixedOffsetFromUtc::from_hours_and_minutes(2, 0)
    }

    fn is_in_dst(&self, t: UnixTimestamp) -> bool {
        use Month::*;

        let d = DateTime::from_timestamp(t, Utc);

        // Directive 2000/84/EC of the European Parliament and of the Council
        // of 19 January 2001 on summer-time arrangements
        // http://eur-lex.europa.eu/legal-content/EN/ALL/?uri=CELEX:32000L0084
        //
        // > Article 1
        //
        // > For the purposes of this Directive "summer-time period"
        // > shall mean the period of the year
        // > during which clocks are put forward by 60 minutes compared with the rest of the year.
        // >
        // > Article 2
        // >
        // > From 2002 onwards, the summer-time period shall begin, in every Member State,
        // > at 1.00 a.m., Greenwich Mean Time, on the last Sunday in March.
        // >
        // > Article 3
        // >
        // > From 2002 onwards, the summer-time period shall end, in every Member State,
        // > at 1.00 a.m., Greenwich Mean Time, on the last Sunday in October.
        if d.month() < March || d.month() > October {
            false
        } else if d.month() > March && d.month() < October {
            true
        } else if d.month() == March {
            !before_last_sunday_1_am(&d)
        } else if d.month() == October {
            before_last_sunday_1_am(&d)
        } else {
            unreachable!()
        }
    }
}

fn before_last_sunday_1_am(d: &DateTime<Utc>) -> bool {
    let last_sunday = last_of_the_month(d, DayOfTheWeek::Sunday);
    d.day() < last_sunday || (
        d.day() == last_sunday &&
        (d.hour(), d.minute(), d.second()) < (1, 0, 0)
    )
}

fn last_of_the_month(d: &DateTime<Utc>, requested_dow: DayOfTheWeek) -> u8 {
    let last_day = d.month().length(d.year().into());
    let last_dow = NaiveDateTime::new(d.year(), d.month(), last_day, 0, 0, 0).day_of_the_week();
    let difference = i32::from(last_dow.to_iso_number()) - i32::from(requested_dow.to_iso_number());
    last_day - (positive_rem(difference, 7) as u8)
}

pub fn days_since_unix(d: &NaiveDateTime) -> i32 {
    (d.year - 1970) * DAYS_PER_COMMON_YEAR
    + leap_days_since_y0(d.year) - leap_days_since_y0(1970)
    + d.month.days_since_january_1st(d.year.into())
    + i32::from(d.day - 1)
}

/// How many leap days occurred between January of year 0 and January of the given year
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
