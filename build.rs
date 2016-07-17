use std::{env, fs, path};
use std::io::Write;

fn main() {
    let path = path::Path::new(&env::var("OUT_DIR").unwrap()).join("generated_data.rs");
    let mut f = fs::File::create(&path).unwrap();
    macro_rules! w {
        ($($tt: tt)*) => {{ writeln!(f, $($tt)*).unwrap(); }}
    }

    let mut running_sum_common = 0;
    let mut running_sum_leap = 0;

    w!("macro_rules! with_month_data {{");
    w!("    ($macro_name: ident) => {{");
    w!("        $macro_name! {{");
    for (i, &(name, length_common, length_leap)) in [
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
    ].iter().enumerate() {
        w!("{} {{", name);
        w!("    number = {},", i + 1);  // i starts at 0
        w!("    common_years = {{ first_day = {}, last_day = {}, }},",
           running_sum_common,
           running_sum_common + length_common - 1);
        w!("    leap_years = {{ first_day = {}, last_day = {}, }},",
           running_sum_leap,
           running_sum_leap + length_leap - 1);
        w!("}},");
        running_sum_common += length_common;
        running_sum_leap += length_leap;
    }
    w!("        }}");
    w!("    }}");
    w!("}}");
}
