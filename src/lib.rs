#[macro_use]
extern crate quick_error;

use std::collections::HashMap;
use std::str;

/*quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Parse(err: nom::Err) {
            description("parse error")
            display("Parse error: {:?}", err)
        }
    }
}*/

pub fn parse_urlencoded_str(input: &str) -> Result<HashMap<String, String>, String> {
    parse_urlencoded(input.as_bytes())
}

pub fn parse_urlencoded(input: &[u8]) -> Result<HashMap<String, String>, String> {
    let mut key_value = HashMap::new();
    let mut index = 0;
    while index < input.len() {
        let key = get_key(input, &mut index).unwrap();
        let value = get_value(input, &mut index).unwrap();
        key_value.insert(key, value);
    }
    Ok(key_value)
}

fn get_key(input: &[u8], mut index: &mut usize) -> Result<String, String> {
    let mut buf = Vec::new();
    while *index < input.len() {
        match input[*index] {
            b'+' => {
                buf.push(b' ');
            }
            b'%' => {
                *index += 1;
                let c = parse_hex_char(input, &mut index).unwrap();
                buf.push(c);
            }
            b'=' => {
                // Make sure that the key is not somehow empty
                if buf.is_empty() {
                    return Err(String::from("Invalid input"));
                }

                let s = String::from_utf8(buf).unwrap();
                *index += 1;
                return Ok(s);
            }
            b'&' => {
                return Err(String::from("Invalid input"));
            }
            c @ _ => {
                buf.push(c);
            }
        }
        *index += 1;
    }

    Err(String::from("Could not get key"))
}

fn get_value(input: &[u8], mut index: &mut usize) -> Result<String, String> {
    let mut buf = Vec::new();
    while *index < input.len() {
        match input[*index] {
            b'+' => {
                buf.push(b' ');
            }
            b'%' => {
                *index += 1;
                let c = parse_hex_char(input, &mut index).unwrap();
                buf.push(c);
            }
            b'=' => {
                return Err(String::from("Invalid input"));
            }
            b'&' => {
                let s = String::from_utf8(buf).unwrap();
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
    let s = String::from_utf8(buf).unwrap();
    *index += 1;
    return Ok(s);
}

fn parse_hex_char(input: &[u8], index: &mut usize) -> Result<u8, String> {
    if *index + 1 < input.len() {
        if valid_hex(input[*index]) && valid_hex(input[*index + 1]) {
            let s = str::from_utf8(&input[*index..*index + 2]).unwrap();
            let c = u8::from_str_radix(s, 16).unwrap();
            *index += 1;
            return Ok(c);
        } else {
            return Err(String::from("Tried to parse invalid hex char"));
        }
    } else {
        return Err(String::from("Unexpected end of input"));
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
