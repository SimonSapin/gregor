use std::{env, fmt, fs, path};
use std::io::Write;

fn main() {
    // The total number of days in the year up to the current month, inclusive,
    // for common (non-leap) years and leap years.
    let mut running_sum_common = 0;
    let mut running_sum_leap = 0;
    let month_data = [
        // Name of the month with its length (number of days) in common years and leap years.
        ("January", 31, 31),
        ("February", 28, 29),
        ("March", 31, 31),
        ("April", 30, 30),
        ("May", 31, 31),
        ("June", 30, 30),
        ("July", 31, 31),
        ("August", 31, 31),
        ("September", 30, 30),
        ("October", 31, 31),
        ("November", 30, 30),
        ("December", 31, 31),
    ].iter().enumerate().map(|(i, &(name, length_common, length_leap))| {
        running_sum_common += length_common;
        running_sum_leap += length_leap;
        (
            Ident(name),
            /* number = */ i + 1,  // i starts at 0
            length_common,
            length_leap,
            /* first_day_in_common_years = */ running_sum_common - length_common,
            /* last_day_in_common_years = */ running_sum_common - 1,
            /* first_day_in_leap_years = */ running_sum_leap - length_leap,
            /* last_day_in_leap_years = */ running_sum_leap - 1,
        )
    }).collect::<Vec<_>>();

    let day_of_the_week_data = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ].iter().enumerate().map(|(i, &name)| (Ident(name), i + 1)).collect::<Vec<_>>();

    let path = path::Path::new(&env::var("OUT_DIR").unwrap()).join("generated_data.rs");
    let mut file = fs::File::create(&path).unwrap();

    macro_rules! with {
        ($variable: ident) => {
            writeln!(
                file,
                "macro_rules! with_{} {{ ($macro_name: ident) => {{ $macro_name!({:?}); }} }}",
                stringify!($variable),
                $variable
            ).unwrap()
        }
    }
    with!(month_data);
    with!(day_of_the_week_data);
}

/// Wrap a string to format without quotes.
struct Ident(&'static str);

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
