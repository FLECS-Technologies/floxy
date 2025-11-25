#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!("[INFO] {}", format!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        eprintln!("[WARN] {}", format!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("[ERROR] {}", format!($($arg)*))
    };
}
