// This module contains general utilities meant to be shared across the other
// modules.

use crate::err::UziErr;
use std::str::FromStr;
use std::time::Duration;

// A function to parse time as milliseconds, with parse errors mapped to thne
// UziErr::BadMillis error.
pub(crate) fn to_millis(word: &str, opt_name: &str) -> Result<Duration, UziErr> {
    word.parse::<u64>()
        .map_err(|_| UziErr::BadMillis(opt_name.into(), word.into()))
        .map(|ms| Duration::from_millis(ms))
}

// A function to parse a generic number which maps an error to a
// UziErr::BadNumber error.
pub(crate) fn to_number<T: FromStr>(word: &str) -> Result<T, UziErr> {
    word.parse::<T>().map_err(|_| UziErr::BadNumber)
}

// A function to parse a bool and map the error to UziErr::BadBool.
pub(crate) fn to_bool(word: &str) -> Result<bool, UziErr> {
    bool::from_str(word).map_err(|_| UziErr::BadBool)
}
