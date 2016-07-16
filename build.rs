use std::{env, fs, path};
use std::io::Write;

const MONTHS: [(&'static str, u32, u32); 12] = [
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
];

fn main() {
    let path = path::Path::new(&env::var("OUT_DIR").unwrap()).join("month_generated.rs");
    let mut f = fs::File::create(&path).unwrap();
    macro_rules! w {
        ($($tt: tt)*) => {{ writeln!(f, $($tt)*).unwrap(); }}
    }
    w!("#[derive(Debug, Eq, PartialEq, Copy, Clone)]");
    w!("pub enum Month {{");
    for &(name, _, _) in &MONTHS {
        w!("    {},", name);
    }
    w!("}}");
    w!("");
    w!("impl Month {{");
    w!("    pub fn days_since_january_1st(self, year_kind: YearKind) -> i32 {{");
    w!("        match (self, year_kind) {{");
    let mut running_sum_common = 0;
    let mut running_sum_leap = 0;
    for &(name, days_common, days_leap) in &MONTHS {
        w!("            (Month::{}, YearKind::Common) => {},", name, running_sum_common);
        w!("            (Month::{}, YearKind::Leap) => {},", name, running_sum_leap);
        running_sum_common += days_common;
        running_sum_leap += days_leap;
    }
    w!("        }}");
    w!("    }}");
    w!("}}");
}
