#![no_std]

#[cfg(any(test, feature = "system_time"))] #[macro_use] extern crate std;

mod num;
#[cfg(feature = "system_time")] mod system_time;
#[cfg(test)] mod tests;
mod time_zones;

use core::fmt;
use num::positive_rem;
use time_zones::days_since_unix;
pub use time_zones::{TimeZone, LocalTimeConversionError, UnambiguousTimeZone, DaylightSaving,
                     Utc, FixedOffsetFromUtc, CentralEurope};

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
               self.year, self.month.to_number(), self.day,
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

    pub fn day_of_the_week(&self) -> DayOfTheWeek { self.naive.day_of_the_week() }

    pub fn from_timestamp(t: UnixTimestamp, time_zone: Tz) -> Self {
        DateTime {
            naive: time_zone.from_timestamp(t),
            time_zone: time_zone,
        }
    }

    pub fn to_timestamp(&self) -> Result<UnixTimestamp, LocalTimeConversionError> {
        self.time_zone.to_timestamp(&self.naive)
    }

    pub fn convert_time_zone<NewTz: TimeZone>(&self, new_time_zone: NewTz)
                                              -> Result<DateTime<NewTz>, LocalTimeConversionError> {
        Ok(DateTime::from_timestamp(try!(self.to_timestamp()), new_time_zone))
    }
}

impl<Tz: UnambiguousTimeZone> DateTime<Tz> {
    pub fn to_unambiguous_timestamp(&self) -> UnixTimestamp {
        self.time_zone.to_unambiguous_timestamp(&self.naive)
    }

    pub fn convert_unambiguous_time_zone<NewTz: TimeZone>(&self, new_time_zone: NewTz) -> DateTime<NewTz> {
        DateTime::from_timestamp(self.to_unambiguous_timestamp(), new_time_zone)
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

    pub fn day_of_the_week(&self) -> DayOfTheWeek {
        const JANUARY_1ST_1970: DayOfTheWeek = DayOfTheWeek::Thursday;
        JANUARY_1ST_1970.add_days(days_since_unix(self))
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

impl<Tz: UnambiguousTimeZone> From<DateTime<Tz>> for UnixTimestamp {
    fn from(datetime: DateTime<Tz>) -> Self {
        datetime.time_zone.to_unambiguous_timestamp(&datetime.naive)
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

macro_rules! declare_month {
    ([ $((
        $name: ident,
        $number: expr,
        $length_in_common_years: expr,
        $length_in_leap_years: expr,
        $first_day_in_common_years: expr,
        $last_day_in_common_years: expr,
        $first_day_in_leap_years: expr,
        $last_day_in_leap_years: expr
    )),+ ]) => {
        #[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
        pub enum Month {
            $(
                $name = $number,
            )+
        }

        impl Month {
            /// Return the month from its number, between 1 and 12.
            pub fn from_number(n: u8) -> Option<Self> {
                match n {
                    $(
                        $number => Some(Month::$name),
                    )+
                    _ => None
                }
            }

            /// Return the number of this month, between 1 and 12.
            pub fn to_number(self) -> u8 {
                match self {
                    $(
                        Month::$name => $number,
                    )+
                }
            }

            pub fn length(self, year_kind: YearKind) -> u8 {
                match year_kind {
                    YearKind::Common => match self {
                        $(
                            Month::$name => $length_in_common_years,
                        )+
                    },
                    YearKind::Leap => match self {
                        $(
                            Month::$name => $length_in_leap_years,
                        )+
                    },
                }
            }

            /// Days between Jan 1st and the first day of this month.
            fn days_since_january_1st(self, year_kind: YearKind) -> i32 {
                match year_kind {
                    YearKind::Common => match self {
                        $(
                            Month::$name => $first_day_in_common_years,
                        )+
                    },
                    YearKind::Leap => match self {
                        $(
                            Month::$name => $first_day_in_leap_years,
                        )+
                    },
                }
            }

            /// In: 0 for Jan 1st, 365 or 366 for Dec 31.
            /// Out: Month and day of the month (1 for the first day).
            fn from_day_of_the_year(day: i32, year_kind: YearKind) -> (Month, u8) {
                match year_kind {
                    YearKind::Common => match day {
                        $(
                            $first_day_in_common_years ... $last_day_in_common_years => {
                                (Month::$name, (day - $first_day_in_common_years + 1) as u8)
                            }
                        )+
                        _ => panic!("Day #{} of the year is out of range", day)
                    },
                    YearKind::Leap => match day {
                        $(
                            $first_day_in_leap_years ... $last_day_in_leap_years => {
                                (Month::$name, (day - $first_day_in_leap_years + 1) as u8)
                            }
                        )+
                        _ => panic!("Day #{} of the year is out of range", day)
                    },
                }
            }
        }
    }
}

macro_rules! declare_day_of_the_week {
    ([ $((
        $name: ident,
        $number: expr
    )),+ ]) => {
        #[derive(Debug, Eq, PartialEq, Copy, Clone)]
        pub enum DayOfTheWeek {
            $(
                $name = $number,
            )+
        }

        impl DayOfTheWeek {
            /// Return the day of the week from its number, where Monday to Sunday are 1 to 7
            /// in accordance with ISO 8601.
            pub fn from_iso_number(n: u8) -> Option<Self> {
                match n {
                    $(
                        $number => Some(DayOfTheWeek::$name),
                    )+
                    _ => None
                }
            }

            /// Return the number of this day of the week, where Monday to Sunday are 1 to 7
            /// in accordance with ISO 8601.
            pub fn to_iso_number(self) -> u8 {
                match self {
                    $(
                        DayOfTheWeek::$name => $number,
                    )+
                }
            }

            // What day of the week is it this many days after this day of the week?
            fn add_days(self, days: i32) -> Self {
                let number = i32::from(self.to_iso_number()) + days;
                let number = positive_rem((number - 1), 7) + 1;  // Normalize to 1...7
                DayOfTheWeek::from_iso_number(number as u8).unwrap()
            }
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/generated_data.rs"));

with_month_data!(declare_month);
with_day_of_the_week_data!(declare_day_of_the_week);
