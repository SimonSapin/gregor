use std::{env, fs, path};
use std::io::Write;

fn main() {
    let months = [
        // Length (number of days) in months in common (non-leap) years, then leap years.
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
    ].iter().scan((1, 0, 0), |&mut (ref mut number, ref mut running_sum_common, ref mut running_sum_leap),
                           &(name, length_common, length_leap)| {
        let month = (
            name,
            *number,
            // First and last day of the month, where 0 is January 1st and 355 or 366 is December 31st.
            (*running_sum_common, *running_sum_common + length_common - 1),
            (*running_sum_leap, *running_sum_leap + length_leap - 1)
        );
        *number += 1;
        *running_sum_common += length_common;
        *running_sum_leap += length_leap;
        Some(month)
    }).collect::<Vec<_>>();

    let path = path::Path::new(&env::var("OUT_DIR").unwrap()).join("month_generated.rs");
    let mut f = fs::File::create(&path).unwrap();
    macro_rules! w {
        ($($tt: tt)*) => {{ writeln!(f, $($tt)*).unwrap(); }}
    }

    w!("#[derive(Debug, Eq, PartialEq, Copy, Clone)]");
    w!("pub enum Month {{");
    for &(name, number, _, _) in &months {
        w!("    {} = {},", name, number);
    }
    w!("}}");
    w!("");
    w!("impl Month {{");
    w!("    /// Days between Jan 1st and the first day of this month.");
    w!("    fn days_since_january_1st(self, year_kind: YearKind) -> i32 {{");
    w!("        match year_kind {{");
    w!("            YearKind::Common => match self {{");
    for &(name, _, (first, _), _) in &months {
        w!("                Month::{} => {},", name, first);
    }
    w!("            }},");
    w!("            YearKind::Leap => match self {{");
    for &(name, _, _, (first, _)) in &months {
        w!("                Month::{} => {},", name, first);
    }
    w!("            }},");
    w!("        }}");
    w!("    }}");
    w!("");
    w!("    /// In: 0 for Jan 1st, 365 or 366 for Dec 31.");
    w!("    /// Out: Month and day of the month (1 for the first day).");
    w!("    fn from_day_of_the_year(day: i32, year_kind: YearKind) -> (Month, u8) {{");
    w!("        match year_kind {{");
    w!("            YearKind::Common => match day {{");
    for &(name, _, (first, last), _) in &months {
        w!("                {}...{} => (Month::{}, (day - {} + 1) as u8),", first, last, name, first);
    }
    w!("                _ => panic!(\"Day #{{}} of the year is out of range\", day)");
    w!("            }},");
    w!("            YearKind::Leap => match day {{");
    for &(name, _, _, (first, last)) in &months {
        w!("                {}...{} => (Month::{}, (day - {} + 1) as u8),", first, last, name, first);
    }
    w!("                _ => panic!(\"Day #{{}} of the year is out of range\", day)");
    w!("            }},");
    w!("        }}");
    w!("    }}");
    w!("}}");
}
