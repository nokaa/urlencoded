#[macro_use]
extern crate quick_error;

use std::collections::HashMap;
use std::str;
use std::string;
use std::num::ParseIntError;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        ParseInt(err: ParseIntError) {
            from()
            description("parse int error")
            display("Parse int error: {}", err)
        }
        StrUtf8(err: str::Utf8Error) {
            from()
            description("str from utf8 error")
            display("Error converting utf8 to str: {}", err)
        }
        StringUtf8(err: string::FromUtf8Error) {
            from()
            description("string from utf8 error")
            display("Error converting utf8 to string: {}", err)
        }
        EOI {
            description("unexpected end of input")
            display("Unexpected end of input")
        }
        InvalidHex {
            description("invalid hex char")
            display("Tried to parse invalid hex char")
        }
        EmptyKey {
            description("empty key")
            display("Tried to parse an empty key")
        }
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
        let key = try!(get_key(input, &mut index));
        let value = try!(get_value(input, &mut index));
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
                let c = try!(parse_hex_char(input, &mut index));
                buf.push(c);
            }
            // Signals end of key/beginning of value
            b'=' => {
                // Make sure that the key is not somehow empty
                if buf.is_empty() {
                    return Err(Error::EmptyKey);
                }

                let s = try!(String::from_utf8(buf));
                *index += 1;
                return Ok(s);
            }
            // Signals end of key-value pair with another present
            b'&' => {
                return Err(Error::InvalidInput);
            }
            c @ _ => {
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
                let c = try!(parse_hex_char(input, &mut index));
                buf.push(c);
            }
            // Signals end of key/beginning of value
            b'=' => {
                return Err(Error::InvalidInput);
            }
            // Signals end of key-value pair with another present
            b'&' => {
                let s = try!(String::from_utf8(buf));
                *index += 1;
                return Ok(s);
            }
            c @ _ => {
                buf.push(c);
            }
        }
        *index += 1;
    }

    // Reached end of input
    let s = try!(String::from_utf8(buf));
    *index += 1;
    return Ok(s);
}

/// Parses a hex encoded character from the input into a byte value. This
/// function is called when `get_key` or `get_value` read a `%`.
fn parse_hex_char(input: &[u8], index: &mut usize) -> Result<u8, Error> {
    // When these functions are called, `index` has already been incremented,
    // and is thus pointing at what should be the first of two hex chars.

    // We check to make sure that there are at least two bytes to read before
    // end of input
    if *index + 1 < input.len() {
        // We check to make sure that both bytes are valid hex characters.
        if valid_hex(input[*index]) && valid_hex(input[*index + 1]) {
            // Convert the two bytes to a string and interpret this as
            // a base-16 byte.
            let s = try!(str::from_utf8(&input[*index..*index + 2]));
            let c = try!(u8::from_str_radix(s, 16));
            // We only increment the index by one because it will be
            // incremented again in the calling function
            *index += 1;
            return Ok(c);
        } else {
            return Err(Error::InvalidHex);
        }
    } else {
        return Err(Error::EOI);
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
mod tests {
    use super::parse_urlencoded;
    use std::collections::HashMap;

    #[test]
    fn parse_single_kv() {
        let data = b"key=val";

        let mut map = HashMap::new();
        map.insert(String::from("key"), String::from("val"));

        assert_eq!(parse_urlencoded(data), Ok(map));
    }

    #[test]
    fn parse_multiple_kv() {
        let data = b"key=val&key1=val1";

        let mut map = HashMap::new();
        map.insert(String::from("key"), String::from("val"));
        map.insert(String::from("key1"), String::from("val1"));

        assert_eq!(parse_urlencoded(data), Ok(map));
    }

    #[test]
    fn parse_empty_input() {
        let data = b"";

        let map = HashMap::new();

        assert_eq!(parse_urlencoded(data), Ok(map));
    }

    #[test]
    #[should_panic]
    fn parse_empty_key_value() {
        let data = b"=val";

        let mut map = HashMap::new();
        map.insert(String::from(""), String::from("val"));

        assert_eq!(parse_urlencoded(data), Ok(map));
    }

    #[test]
    fn parse_key_empty_value() {
        let data = b"key=";

        let mut map = HashMap::new();
        map.insert(String::from("key"), String::from(""));

        assert_eq!(parse_urlencoded(data), Ok(map));
    }

    #[test]
    fn parse_multiple_key_empty_value() {
        let data = b"key=&key1=";

        let mut map = HashMap::new();
        map.insert(String::from("key"), String::from(""));
        map.insert(String::from("key1"), String::from(""));

        assert_eq!(parse_urlencoded(data), Ok(map));
    }

    #[test]
    fn parse_escaped_key() {
        // 早く=val
        let data = b"%E6%97%A9%E3%81%8F=val";

        let mut map = HashMap::new();
        map.insert(String::from("早く"), String::from("val"));

        assert_eq!(parse_urlencoded(data), Ok(map));
    }

    #[test]
    fn parse_escaped_val() {
        // key=早く
        let data = b"key=%E6%97%A9%E3%81%8F";

        let mut map = HashMap::new();
        map.insert(String::from("key"), String::from("早く"));

        assert_eq!(parse_urlencoded(data), Ok(map));
    }

    // Since I know some nerd will try this
    #[test]
    fn parse_emoji_key() {
        // 😱=val
        let data = b"%F0%9F%98%B1=val";

        let mut map = HashMap::new();
        map.insert(String::from("😱"), String::from("val"));

        assert_eq!(parse_urlencoded(data), Ok(map));
    }

    #[test]
    fn parse_emoji_val() {
        // key=😱
        let data = b"key=%F0%9F%98%B1";

        let mut map = HashMap::new();
        map.insert(String::from("key"), String::from("😱"));

        assert_eq!(parse_urlencoded(data), Ok(map));
    }

    #[test]
    fn parse_space_key() {
        // 早く test=val
        let data = b"%E6%97%A9%E3%81%8F+test=val";

        let mut map = HashMap::new();
        map.insert(String::from("早く test"), String::from("val"));

        assert_eq!(parse_urlencoded(data), Ok(map));
    }

    #[test]
    fn parse_space_val() {
        // key=早く test
        let data = b"key=%E6%97%A9%E3%81%8F+test";

        let mut map = HashMap::new();
        map.insert(String::from("key"), String::from("早く test"));

        assert_eq!(parse_urlencoded(data), Ok(map));
    }
}
