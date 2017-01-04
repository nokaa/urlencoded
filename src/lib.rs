//! A byte parser for [urlencoded]() data that takes one pass over the data.
//!
//! # Examples
//!
//! Example usage:
//!
//! ```no_run
//! extern crate urlencoded;
//!
//! fn main() {
//!     let data = b"key=value";
//!     let map = urlencoded::parse_urlencoded(data).unwrap();
//!     let key = map.get(&"key".to_string()).unwrap();
//!     println!("{}", key);
//! }
//! ```
#[macro_use]
extern crate log;
#[macro_use(quick_error)]
extern crate quick_error;

use std::collections::HashMap;
use std::{num, str};

quick_error! {
    #[derive(Debug, PartialEq)]
    pub enum Error {
        /// Error parsing int from string
        ParseInt(err: num::ParseIntError) {
            from()
            description("parse int error")
            display("Parse int error: {}", err)
        }
        /// Error converting Utf8 to string
        StrUtf8(err: str::Utf8Error) {
            from()
            from(err: std::string::FromUtf8Error) -> (err.utf8_error())
            description("str from utf8 error")
            display("Error converting utf8 to str: {}", err)
        }
        /// Unexpected end of input.
        EOI {
            description("unexpected end of input")
            display("Unexpected end of input")
        }
        /// Invalid Hexadecimal character
        InvalidHex {
            description("invalid hex char")
            display("Tried to parse invalid hex char")
        }
        /// Input contained an empty key.
        EmptyKey {
            description("empty key")
            display("Tried to parse an empty key")
        }
        /// Generic `something went wrong!` error.
        InvalidInput {
            description("Generic `something went wrong!`")
            display("Invalid input")
        }
    }
}

/// Parses urlencoded data from a `&str`.
pub fn parse_urlencoded_str(input: &str) -> Result<HashMap<String, String>, Error> {
    parse_urlencoded(input.as_bytes())
}

/// Parses urlencoded data from a byte array.
pub fn parse_urlencoded(input: &[u8]) -> Result<HashMap<String, String>, Error> {
    let mut key_value = HashMap::new();
    let mut index = 0;
    while index < input.len() {
        let key = get_key(input, &mut index)?;
        debug!("key: {}", key);
        let value = get_value(input, &mut index)?;
        debug!("value: {}", value);
        key_value.insert(key, value);
    }
    Ok(key_value)
}

/// Gets a key from the input. Converts all encoded characters in the process.
///
/// Returns an error if there is no key, i.e. the first byte is `=`.
///
/// Returns an error if we encounter an `&`. When reading a key, we should
/// encounter an `=` before an `&` could ever be read.
///
/// Returns an error if we unexpectedly reach end of input.
fn get_key(input: &[u8], mut index: &mut usize) -> Result<String, Error> {
    let mut buf = Vec::new();
    while *index < input.len() {
        match input[*index] {
            // Encoded ` `
            b'+' => {
                buf.push(b' ');
            }
            // Signals hex encoded character
            b'%' => {
                *index += 1;
                let c = parse_hex_char(input, &mut index)?;
                buf.push(c);
            }
            // Signals end of key/beginning of value
            b'=' => {
                // Make sure that the key is not somehow empty
                if buf.is_empty() {
                    return Err(Error::EmptyKey);
                }

                let s = String::from_utf8(buf)?;
                *index += 1;
                return Ok(s);
            }
            // Signals end of key-value pair with another present
            b'&' => {
                return Err(Error::InvalidInput);
            }
            c => {
                buf.push(c);
            }
        }
        *index += 1;
    }

    Err(Error::EOI)
}

/// Gets a value from the input. Converts all encoded characters in the process.
///
/// Returns an error if we encounter an `=`. When reading a value, we should
/// encounter an `&` or end of input before an `=` could ever be read.
fn get_value(input: &[u8], mut index: &mut usize) -> Result<String, Error> {
    let mut buf = Vec::new();
    while *index < input.len() {
        match input[*index] {
            // Encoded ` `
            b'+' => {
                buf.push(b' ');
            }
            // Signals hex encoded character
            b'%' => {
                *index += 1;
                let c = parse_hex_char(input, &mut index)?;
                buf.push(c);
            }
            // Signals end of key/beginning of value
            b'=' => {
                return Err(Error::InvalidInput);
            }
            // Signals end of key-value pair with another present
            b'&' => {
                let s = String::from_utf8(buf)?;
                *index += 1;
                return Ok(s);
            }
            c => {
                buf.push(c);
            }
        }
        *index += 1;
    }

    // Reached end of input
    let s = String::from_utf8(buf)?;
    *index += 1;
    Ok(s)
}

/// Parses a hex encoded character from the input into a byte value. This
/// function is called when `get_key` or `get_value` read a `%`.
fn parse_hex_char(input: &[u8], index: &mut usize) -> Result<u8, Error> {
    // When this function is called, `index` has already been incremented,
    // and is thus pointing at what should be the first of two hex chars.

    // We check to make sure that there are at least two bytes to read before
    // end of input
    if *index + 1 < input.len() {
        // We check to make sure that both bytes are valid hex characters.
        if valid_hex(input[*index]) && valid_hex(input[*index + 1]) {
            // Convert the two bytes to a string and interpret this as
            // a base-16 byte.
            let s = str::from_utf8(&input[*index..*index + 2])?;
            let c = u8::from_str_radix(s, 16)?;
            // We only increment the index by one because it will be
            // incremented again in the calling function
            *index += 1;
            Ok(c)
        } else {
            Err(Error::InvalidHex)
        }
    } else {
        Err(Error::EOI)
    }
}

/// Hex encoded characters are sent in the form of %XY where X and Y are
/// Hexadecimal digits. However, X and Y are chars, so an example might be
/// `%5F`. Valid hex makes sure that one of these is in the valid ascii range
/// for a hex character.
fn valid_hex(input: u8) -> bool {
    match input {
        b'0'...b'9' | b'A'...b'F' => true,
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::parse_hex_char;
    #[test]
    fn parse_cr_nl() {
        let data = b"%0D%0A";
        let mut i = 1;
        assert_eq!(Ok(13), parse_hex_char(data, &mut i));
        i += 2;
        assert_eq!(Ok(10), parse_hex_char(data, &mut i));
    }
}
