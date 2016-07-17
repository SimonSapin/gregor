use super::*;
use time_zones::{days_since_unix, leap_days_since_y0};
use Month::*;
use DayOfTheWeek::*;

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
fn unixy_days() {
    assert_eq!(days_since_unix(&NaiveDateTime::new(1969, December, 31, 0, 0, 0)), -1);
    assert_eq!(days_since_unix(&NaiveDateTime::new(1970, January, 1, 0, 0, 0)), 0);
    assert_eq!(days_since_unix(&NaiveDateTime::new(1970, January, 2, 0, 0, 0)), 1);
    assert_eq!(days_since_unix(&NaiveDateTime::new(1970, February, 1, 0, 0, 0)), 31);
    assert_eq!(days_since_unix(&NaiveDateTime::new(1971, January, 1, 0, 0, 0)), 365);
    assert_eq!(days_since_unix(&NaiveDateTime::new(1972, January, 1, 0, 0, 0)), 365 * 2);
    // 1972 is a leap year.
    assert_eq!(days_since_unix(&NaiveDateTime::new(1973, January, 1, 0, 0, 0)), 365 * 3 + 1);
    assert_eq!(days_since_unix(&NaiveDateTime::new(2016, July, 16, 0, 0, 0)), 16998);
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

    // https://www.wolframalpha.com/input/?i=100000000000+seconds+before+Unix+epoch
    // > 2:13:20 pm UTC  |  Thursday, February 15, 1200 BC (extrapolated Gregorian calendar)
    //
    // For some reason GNU coreutils uses local mean time instead of UTC
    // with TZ=Etc/UTC for year -1199.
    assert_convertions!(-100_000_000_000, -1199, February, 15, 14, 13, 20);

    // Python:
    // import datetime
    // datetime.datetime.fromutctimestamp(10000000000)

    // GNU coreutils:
    // date +%s -d 2000-1-1T00:00:00Z
    // TZ=Etc/UTC date -d @10000000000

    assert_convertions!(-62_167_219_200, 0, January, 1, 0, 0, 0);
    assert_convertions!(-62_162_035_201, 0, February, 29, 23, 59, 59);  // Y0 / 1 BC is leap
    assert_convertions!(-62_162_035_200, 0, March, 1, 0, 0, 0);
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

#[test]
fn fixed_offset_from_utc() {
    let tz = FixedOffsetFromUtc::from_hours_and_minutes(2, 0);
    let t = UnixTimestamp(1468769652);
    let dt = NaiveDateTime::new(2016, July, 17, 17, 34, 12);
    let utc_dt = NaiveDateTime::new(2016, July, 17, 15, 34, 12);
    assert_eq!(tz.to_unambiguous_timestamp(&dt), t);
    assert_eq!(tz.from_timestamp(t), dt);
    assert_eq!(Utc.from_timestamp(t), utc_dt);
}

#[test]
fn numbers() {
    assert_eq!(January.to_number(), 1);
    assert_eq!(December.to_number(), 12);
    assert_eq!(Month::from_number(0), None);
    assert_eq!(Month::from_number(1), Some(January));
    assert_eq!(Month::from_number(12), Some(December));
    assert_eq!(Month::from_number(13), None);

    assert_eq!(Monday.to_iso_number(), 1);
    assert_eq!(Sunday.to_iso_number(), 7);
    assert_eq!(DayOfTheWeek::from_iso_number(0), None);
    assert_eq!(DayOfTheWeek::from_iso_number(1), Some(Monday));
    assert_eq!(DayOfTheWeek::from_iso_number(7), Some(Sunday));
    assert_eq!(DayOfTheWeek::from_iso_number(8), None);
}

#[test]
fn day_of_the_week() {
    assert_eq!(NaiveDateTime::new(2016, July, 17, 0, 0, 0).day_of_the_week(), Sunday);
    assert_eq!(NaiveDateTime::new(2000, January, 1, 0, 0, 0).day_of_the_week(), Saturday);
    assert_eq!(NaiveDateTime::new(1970, January, 1, 0, 0, 0).day_of_the_week(), Thursday);
    assert_eq!(NaiveDateTime::new(1837, May, 3, 0, 0, 0).day_of_the_week(), Wednesday);


    // https://en.wikipedia.org/wiki/Week
    // > Adding one to the remainder after dividing by seven a date's Julian day number
    // > (JD modulo 7 + 1) yields that date's ISO 8601 day of the week,[3]
    //
    // > This is equivalent to saying that JD0,
    // > i.e. 1 January 4713 BC of the proleptic Julian calendar, was a Monday.

    // https://en.wikipedia.org/wiki/Julian_day
    // > Julian day number 0 assigned to the day starting at noon
    // > on January 1, 4713 BC, proleptic Julian calendar
    // > (November 24, 4714 BC, in the proleptic Gregorian calendar)
    assert_eq!(NaiveDateTime::new(-4713, November, 24, 0, 0, 0).day_of_the_week(), Monday);
}
