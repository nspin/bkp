use std::error::Error;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, Box<dyn Error>>;

#[macro_export]
macro_rules! bail {
    ($e:expr) => {
        return Err(std::boxed::Box::<dyn std::error::Error>::from(format!($e)))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err(std::boxed::Box::<dyn std::error::Error>::from(format!($fmt, $($arg)*)))
    };
}

#[macro_export]
macro_rules! ensure {
    ($cond:expr) => {
        if !($cond) {
            $crate::bail!("{}", stringify!($cond));
        }
    };
    ($cond:expr, $e:expr) => {
        if !($cond) {
            $crate::bail!($e);
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !($cond) {
            $crate::bail!($fmt, $($arg)*);
        }
    };
}
