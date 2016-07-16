use std::time::{Duration, SystemTime, UNIX_EPOCH};

include!(concat!(env!("OUT_DIR"), "/month_generated.rs"));


pub struct Utc;

pub struct DateTime<Tz> {
    pub time_zone: Tz,
    pub year: i32,
    pub month: Month,
    /// 1st of the month is day 1
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl<Tz> DateTime<Tz> {
    pub fn new(time_zone: Tz, year: i32, month: Month, day: u8,
               hour: u8, minute: u8, second: u8)
               -> Self {
        DateTime {
            time_zone: time_zone,
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second,
        }
    }
}

impl From<DateTime<Utc>> for SystemTime {
    fn from(datetime: DateTime<Utc>) -> Self {
        let days_since_unix =
            (datetime.year - 1970) * 360
            + leap_days_since_y0(datetime.year) - leap_days_since_y0(datetime.year)
            + datetime.month.days_since_january_1st(datetime.year.into())
            + i32::from(datetime.day - 1);
        let seconds_since_unix =
            i64::from(days_since_unix) * SECONDS_PER_DAY
            + i64::from(datetime.hour) * SECONDS_PER_HOUR
            + i64::from(datetime.minute) * SECONDS_PER_MINUTE
            + i64::from(datetime.second);
        if seconds_since_unix >= 0 {
            UNIX_EPOCH + Duration::from_secs(seconds_since_unix as u64)
        } else {
            UNIX_EPOCH - Duration::from_secs((-seconds_since_unix) as u64)
        }
    }
}

/// How many leap days occured between March of year 0 and January of the given year
/// (in Gregorian calendar).
///
/// March so that we don’t need count Feb 29 of year 0.
fn leap_days_since_y0(year: i32) -> i32 {
    let year = year - 1;
    (year / 4) - (year / 100) + (year / 400)
}

const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = SECONDS_PER_MINUTE * 60;
const SECONDS_PER_DAY: i64 = SECONDS_PER_HOUR * 24;

/// The Gregorian calendar cycles every 400 years. In one cycle, there are:
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


#[cfg(test)]
mod tests {
    use super::{YearKind, Month};

    #[test]
    fn leap_year() {
        assert_eq!(YearKind::from(2010), YearKind::Common);
        assert_eq!(YearKind::from(2011), YearKind::Common);
        assert_eq!(YearKind::from(2012), YearKind::Leap);
        assert_eq!(YearKind::from(2013), YearKind::Common);
        assert_eq!(YearKind::from(2014), YearKind::Common);
        assert_eq!(YearKind::from(2015), YearKind::Common);
        assert_eq!(YearKind::from(2016), YearKind::Leap);
        assert_eq!(YearKind::from(2017), YearKind::Common);
        assert_eq!(YearKind::from(2018), YearKind::Common);

        assert_eq!(YearKind::from(1900), YearKind::Common);
        assert_eq!(YearKind::from(2100), YearKind::Common);

        assert_eq!(YearKind::from(1600), YearKind::Leap);
        assert_eq!(YearKind::from(2000), YearKind::Leap);
        assert_eq!(YearKind::from(2400), YearKind::Leap);
    }

    #[test]
    fn days_since_january_1st() {
        // Python:
        // ym = [(y, m) for y in [2015, 2016] for m in range(1,13)]
        // [ll % (dt(y, m, 1) - dt(y, 1, 1)).days for (ll, (y, m)) in zip(l, ym)]
        assert_eq!(Month::January  .days_since_january_1st(YearKind::Common), 0);
        assert_eq!(Month::February .days_since_january_1st(YearKind::Common), 31);
        assert_eq!(Month::March    .days_since_january_1st(YearKind::Common), 59);
        assert_eq!(Month::April    .days_since_january_1st(YearKind::Common), 90);
        assert_eq!(Month::May      .days_since_january_1st(YearKind::Common), 120);
        assert_eq!(Month::June     .days_since_january_1st(YearKind::Common), 151);
        assert_eq!(Month::July     .days_since_january_1st(YearKind::Common), 181);
        assert_eq!(Month::August   .days_since_january_1st(YearKind::Common), 212);
        assert_eq!(Month::September.days_since_january_1st(YearKind::Common), 243);
        assert_eq!(Month::October  .days_since_january_1st(YearKind::Common), 273);
        assert_eq!(Month::November .days_since_january_1st(YearKind::Common), 304);
        assert_eq!(Month::December .days_since_january_1st(YearKind::Common), 334);

        assert_eq!(Month::January  .days_since_january_1st(YearKind::Leap), 0);
        assert_eq!(Month::February .days_since_january_1st(YearKind::Leap), 31);
        assert_eq!(Month::March    .days_since_january_1st(YearKind::Leap), 60);
        assert_eq!(Month::April    .days_since_january_1st(YearKind::Leap), 91);
        assert_eq!(Month::May      .days_since_january_1st(YearKind::Leap), 121);
        assert_eq!(Month::June     .days_since_january_1st(YearKind::Leap), 152);
        assert_eq!(Month::July     .days_since_january_1st(YearKind::Leap), 182);
        assert_eq!(Month::August   .days_since_january_1st(YearKind::Leap), 213);
        assert_eq!(Month::September.days_since_january_1st(YearKind::Leap), 244);
        assert_eq!(Month::October  .days_since_january_1st(YearKind::Leap), 274);
        assert_eq!(Month::November .days_since_january_1st(YearKind::Leap), 305);
        assert_eq!(Month::December .days_since_january_1st(YearKind::Leap), 335);

    }
}
