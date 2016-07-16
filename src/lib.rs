#![no_std]

use core::fmt;

#[cfg(feature = "system_time")]
mod system_time;

include!(concat!(env!("OUT_DIR"), "/month_generated.rs"));

/// In seconds since 1970-01-01 00:00:00 UTC.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct UnixTimestamp(i64);

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Utc;

#[derive(Eq, PartialEq, Copy, Clone)]
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

impl<Tz: fmt::Debug> fmt::Debug for DateTime<Tz> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "DateTime({:?}, {:04}-{:02}-{:02} {:02}:{:02}:{:02})",
               self.time_zone, self.year, self.month as u8, self.day,
               self.hour, self.minute, self.second)
    }
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

    fn days_since_unix(&self) -> i32 {
        (self.year - 1970) * DAYS_PER_COMMON_YEAR
        + leap_days_since_y0(self.year) - leap_days_since_y0(1970)
        + self.month.days_since_january_1st(self.year.into())
        + i32::from(self.day - 1)
    }
}

/// Integer divison that rounds towards negative infinity
fn div_floor(dividend: i64, divisor: i64) -> i64 {
    if dividend > 0 {
        dividend / divisor
    } else {
        (dividend + 1 - divisor) / divisor
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

impl From<UnixTimestamp> for DateTime<Utc> {
    fn from(u: UnixTimestamp) -> Self {
        let days_since_unix = div_floor(u.0, SECONDS_PER_DAY) as i32;
        let days = days_since_unix + days_since_d0(1970);
        let year = days * 400 / DAYS_PER_400YEARS;
        let day_of_the_year = days - days_since_d0(year);
        let (month, day) = Month::from_day_of_the_year(day_of_the_year, year.into());
        DateTime {
            time_zone: Utc,
            year: year,
            month: month,
            day: day,
            hour: positive_rem(div_floor(u.0, SECONDS_PER_HOUR), 24) as u8,
            minute: positive_rem(div_floor(u.0, SECONDS_PER_MINUTE), 60) as u8,
            second: positive_rem(u.0, 60) as u8,
        }
    }
}

impl From<DateTime<Utc>> for UnixTimestamp {
    fn from(datetime: DateTime<Utc>) -> Self {
        UnixTimestamp(
            i64::from(datetime.days_since_unix()) * SECONDS_PER_DAY
            + i64::from(datetime.hour) * SECONDS_PER_HOUR
            + i64::from(datetime.minute) * SECONDS_PER_MINUTE
            + i64::from(datetime.second)
        )
    }
}

/// How many leap days occured between January of year 0 and January of the given year
/// (in Gregorian calendar).
//
// FIXME: This may be incorrect for year negative years.
fn leap_days_since_y0(year: i32) -> i32 {
    let year = year - 1;  // Don’t include Feb 29 of the given year, if any.
    // +1 because year 0 is a leap year.
    ((year / 4) - (year / 100) + (year / 400)) + 1
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
mod tests {
    use super::*;
    use super::leap_days_since_y0;
    use Month::*;

    #[test]
    fn fmt() {
        assert_eq!(format!("{:?}", DateTime::new(Utc, 2016, July, 16, 20, 58, 46)),
                   "DateTime(Utc, 2016-07-16 20:58:46)");
    }

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

    #[test]
    fn counting_leap_days() {
        assert_eq!(leap_days_since_y0(1970), 478);
        assert_eq!(leap_days_since_y0(1971), 478);
        assert_eq!(leap_days_since_y0(1972), 478);
        assert_eq!(leap_days_since_y0(1973), 479);
    }

    #[test]
    fn days_since_unix() {
        assert_eq!(DateTime::new(Utc, 1969, December, 31, 0, 0, 0).days_since_unix(), -1);
        assert_eq!(DateTime::new(Utc, 1970, January, 1, 0, 0, 0).days_since_unix(), 0);
        assert_eq!(DateTime::new(Utc, 1970, January, 2, 0, 0, 0).days_since_unix(), 1);
        assert_eq!(DateTime::new(Utc, 1970, February, 1, 0, 0, 0).days_since_unix(), 31);
        assert_eq!(DateTime::new(Utc, 1971, January, 1, 0, 0, 0).days_since_unix(), 365);
        assert_eq!(DateTime::new(Utc, 1972, January, 1, 0, 0, 0).days_since_unix(), 365 * 2);
        // 1972 is a leap year.
        assert_eq!(DateTime::new(Utc, 1973, January, 1, 0, 0, 0).days_since_unix(), 365 * 3 + 1);
        assert_eq!(DateTime::new(Utc, 2016, July, 16, 0, 0, 0).days_since_unix(), 16998);
    }

    #[test]
    fn conversions() {
        macro_rules! assert_convertions {
            ($timestamp: expr, $($e: expr),*) => {
                let timestamp = UnixTimestamp($timestamp);
                let datetime = DateTime::new(Utc, $($e),*);
                assert_eq!(DateTime::<Utc>::from(timestamp), datetime);
                assert_eq!(UnixTimestamp::from(datetime), timestamp);
            }
        }

        // Python:
        // import datetime
        // datetime.datetime.fromutctimestamp(10000000000)

        // GNU coreutils:
        // date +%s -d 2000-1-1T00:00:00Z
        // TZ=Etc/UTC date -d @10000000000

//        assert_convertions!(-100_000_000_000, -1199, February, 15, 14, 22, 41);
        assert_convertions!(-50_000_000_000, 385, July, 25, 7, 6, 40);
        assert_convertions!(-1_000_000_000, 1938, April, 24, 22, 13, 20);
        assert_convertions!(-10_000_000, 1969, September, 7, 6, 13, 20);
        assert_convertions!(-1, 1969, December, 31, 23, 59, 59);
        assert_convertions!(0, 1970, January, 1, 0, 0, 0);
        assert_convertions!(1, 1970, January, 1, 0, 0, 1);
        assert_convertions!(100_000, 1970, January, 2, 3, 46, 40);
        assert_convertions!(1_000_000, 1970, January, 12, 13, 46, 40);
        assert_convertions!(10_000_000, 1970, April, 26, 17, 46, 40);
        assert_convertions!(100_000_000, 1973, March, 3, 9, 46, 40);
        assert_convertions!(946_684_800, 2000, January, 1, 0, 0, 0);
        assert_convertions!(1_000_000_000, 2001, September, 9, 1, 46, 40);
        assert_convertions!(1_468_627_200, 2016, July, 16, 0, 0, 0);
        assert_convertions!(1_468_702_726, 2016, July, 16, 20, 58, 46);
        assert_convertions!(10_000_000_000, 2286, November, 20, 17, 46, 40);
        assert_convertions!(400_000_000_000, 14645, June, 30, 15, 6, 40);
    }

    #[cfg(feature = "system_time")]
    #[test]
    fn system_time() {
        use std::time::{Duration, SystemTime, UNIX_EPOCH};

        assert_eq!(DateTime::<Utc>::from(UNIX_EPOCH),
                   DateTime::new(Utc, 1970, January, 1, 0, 0, 0));

        assert_eq!(DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(1_468_702_726)),
                   DateTime::new(Utc, 2016, July, 16, 20, 58, 46));

        assert_eq!(SystemTime::from(DateTime::new(Utc, 1970, January, 1, 0, 0, 0)),
                   UNIX_EPOCH);

        assert_eq!(SystemTime::from(DateTime::new(Utc, 2016, July, 16, 20, 58, 46)),
                   UNIX_EPOCH + Duration::from_secs(1_468_702_726));
    }
}
