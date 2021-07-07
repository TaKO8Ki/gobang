#[macro_export]
macro_rules! outln {
    ($($expr:expr),+) => {{
        use std::io::{Write};
        use std::fs::OpenOptions;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open("gobang.log")
            .unwrap();
        writeln!(file, $($expr),+).expect("Can't write output");
    }}
}
