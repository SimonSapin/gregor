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

const SECONDS_PER_MINUTE: u32 = 60;
const SECONDS_PER_HOUR: u32 = SECONDS_PER_MINUTE * 60;
const SECONDS_PER_DAY: u32 = SECONDS_PER_HOUR * 24;

impl YearKind {
    pub fn days(self) -> u32 {
        match self {
            YearKind::Common => 365,
            YearKind::Leap => 366,
        }
    }

    pub fn seconds(self) -> u32 {
        self.days() * SECONDS_PER_DAY
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl Month {
    pub fn days(self, year_kind: YearKind) -> u32 {
        match self {
            Month::January => 31,
            Month::February => match year_kind {
                YearKind::Common => 28,
                YearKind::Leap => 29,
            },
            Month::March => 31,
            Month::April => 30,
            Month::May => 31,
            Month::June => 30,
            Month::July => 31,
            Month::August => 31,
            Month::September => 30,
            Month::October => 31,
            Month::November => 30,
            Month::December => 31,
        }
    }

    pub fn seconds(self, year_kind: YearKind) -> u32 {
        self.days(year_kind) * SECONDS_PER_DAY
    }

    fn days_since_january(self, year_kind: YearKind) -> u32 {
        use Month::*;
        macro_rules! sum {
            ( $( $earlier_month: ident )* ) => {
                (0 $( + $earlier_month.days(year_kind) )*)
            }
        }
        match self {
            January => sum!(),
            February => sum!(January),
            March => sum!(January February),
            April => sum!(January February March),
            May => sum!(January February March April),
            June => sum!(January February March April May),
            July => sum!(January February March April May June),
            August => sum!(January February March April May June July),
            September => sum!(January February March April May June July August),
            October => sum!(January February March April May June July August September),
            November => sum!(January February March April May June July August September October),
            December => sum!(January February March April May June July August September October November),
        }
    }

    fn seconds_since_january(self, year_kind: YearKind) -> u32 {
        self.days_since_january(year_kind) * SECONDS_PER_DAY
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Add;
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
    fn yearly_seconds() {
        assert_eq!(YearKind::from(2000).seconds(), 31_622_400);
        assert_eq!(YearKind::from(2001).seconds(), 31_536_000);
    }

    #[test]
    fn monthly_seconds() {
        // Python
        // y = 2015
        // for i in range(1, 12):
        //   print(i, (datetime.datetime(y, i + 1, 1) - datetime.datetime(y, i, 1)).total_seconds())
        assert_eq!(Month::January  .seconds(YearKind::Common), 2_678_400);
        assert_eq!(Month::February .seconds(YearKind::Common), 2_419_200);
        assert_eq!(Month::March    .seconds(YearKind::Common), 2_678_400);
        assert_eq!(Month::April    .seconds(YearKind::Common), 2_592_000);
        assert_eq!(Month::May      .seconds(YearKind::Common), 2_678_400);
        assert_eq!(Month::June     .seconds(YearKind::Common), 2_592_000);
        assert_eq!(Month::July     .seconds(YearKind::Common), 2_678_400);
        assert_eq!(Month::August   .seconds(YearKind::Common), 2_678_400);
        assert_eq!(Month::September.seconds(YearKind::Common), 2_592_000);
        assert_eq!(Month::October  .seconds(YearKind::Common), 2_678_400);
        assert_eq!(Month::November .seconds(YearKind::Common), 2_592_000);
        assert_eq!(Month::December .seconds(YearKind::Common), 2_678_400);

        assert_eq!(Month::January  .seconds(YearKind::Leap), 2_678_400);
        assert_eq!(Month::February .seconds(YearKind::Leap), 2_505_600);
        assert_eq!(Month::March    .seconds(YearKind::Leap), 2_678_400);
        assert_eq!(Month::April    .seconds(YearKind::Leap), 2_592_000);
        assert_eq!(Month::May      .seconds(YearKind::Leap), 2_678_400);
        assert_eq!(Month::June     .seconds(YearKind::Leap), 2_592_000);
        assert_eq!(Month::July     .seconds(YearKind::Leap), 2_678_400);
        assert_eq!(Month::August   .seconds(YearKind::Leap), 2_678_400);
        assert_eq!(Month::September.seconds(YearKind::Leap), 2_592_000);
        assert_eq!(Month::October  .seconds(YearKind::Leap), 2_678_400);
        assert_eq!(Month::November .seconds(YearKind::Leap), 2_592_000);
        assert_eq!(Month::December .seconds(YearKind::Leap), 2_678_400);
    }

    #[test]
    fn yearly_monthly_consistency() {
        use super::Month::*;
        let twelve = [January, February, March, April, May, June,
                      July, August, September, October, November, December];

        assert_eq!(twelve.iter().map(|&m| m.seconds(YearKind::Common)).fold(0, Add::add),
                   YearKind::Common.seconds());
        assert_eq!(twelve.iter().map(|&m| m.seconds(YearKind::Leap)).fold(0, Add::add),
                   YearKind::Leap.seconds());
    }

    #[test]
    fn seconds_since_january() {
        // Python:
        // ym = [(y, m) for y in [2015, 2016] for m in range(1,13)]
        // [ll % int((dt(y, m, 1) - dt(y, 1, 1)).total_seconds()) for (ll, (y, m)) in zip(l, ym)]
        assert_eq!(Month::January  .seconds_since_january(YearKind::Common), 0);
        assert_eq!(Month::February .seconds_since_january(YearKind::Common), 2_678_400);
        assert_eq!(Month::March    .seconds_since_january(YearKind::Common), 5_097_600);
        assert_eq!(Month::April    .seconds_since_january(YearKind::Common), 7_776_000);
        assert_eq!(Month::May      .seconds_since_january(YearKind::Common), 10_368_000);
        assert_eq!(Month::June     .seconds_since_january(YearKind::Common), 13_046_400);
        assert_eq!(Month::July     .seconds_since_january(YearKind::Common), 15_638_400);
        assert_eq!(Month::August   .seconds_since_january(YearKind::Common), 18_316_800);
        assert_eq!(Month::September.seconds_since_january(YearKind::Common), 20_995_200);
        assert_eq!(Month::October  .seconds_since_january(YearKind::Common), 23_587_200);
        assert_eq!(Month::November .seconds_since_january(YearKind::Common), 26_265_600);
        assert_eq!(Month::December .seconds_since_january(YearKind::Common), 28_857_600);

        assert_eq!(Month::January  .seconds_since_january(YearKind::Leap), 0);
        assert_eq!(Month::February .seconds_since_january(YearKind::Leap), 2_678_400);
        assert_eq!(Month::March    .seconds_since_january(YearKind::Leap), 5_184_000);
        assert_eq!(Month::April    .seconds_since_january(YearKind::Leap), 7_862_400);
        assert_eq!(Month::May      .seconds_since_january(YearKind::Leap), 10_454_400);
        assert_eq!(Month::June     .seconds_since_january(YearKind::Leap), 13_132_800);
        assert_eq!(Month::July     .seconds_since_january(YearKind::Leap), 15_724_800);
        assert_eq!(Month::August   .seconds_since_january(YearKind::Leap), 18_403_200);
        assert_eq!(Month::September.seconds_since_january(YearKind::Leap), 21_081_600);
        assert_eq!(Month::October  .seconds_since_january(YearKind::Leap), 23_673_600);
        assert_eq!(Month::November .seconds_since_january(YearKind::Leap), 26_352_000);
        assert_eq!(Month::December .seconds_since_january(YearKind::Leap), 28_944_000);
    }
}
