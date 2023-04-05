#![allow(dead_code)]
use chrono::prelude::DateTime;
use chrono::Utc;

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{fmt, num::ParseIntError};

pub fn timestamp_as_string(timestamp: u32) -> String {
    // Convert block timestamp to something readable
    let seconds: u64 = timestamp.into();
    let d = UNIX_EPOCH + Duration::from_secs(seconds);
    let datetime = DateTime::<Utc>::from(d);
    let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    timestamp_str
}

pub fn timestamp_age_as_sec(timestamp: u32) -> u64 {
    // Return the age of the block timestamp (against current time) in seconds
    let block_timestamp: u64 = timestamp.into();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if now > block_timestamp {
        now - block_timestamp
    } else {
        0
    }
}

// Decode hex
// from https://play.rust-lang.org/?version=stable&mode=debug&edition=2015&gist=e241493d100ecaadac3c99f37d0f766f

pub fn decode_hexstr(s: &str) -> Result<Vec<u8>, DecodeHexError> {
    if s.len() % 2 != 0 {
        Err(DecodeHexError::OddLength)
    } else {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.into()))
            .collect()
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeHexError {
    OddLength,
    ParseInt(ParseIntError),
}

impl From<ParseIntError> for DecodeHexError {
    fn from(e: ParseIntError) -> Self {
        DecodeHexError::ParseInt(e)
    }
}

impl fmt::Display for DecodeHexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodeHexError::OddLength => "input string has an odd number of bytes".fmt(f),
            DecodeHexError::ParseInt(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for DecodeHexError {}
