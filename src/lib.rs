#[macro_use]
extern crate nom;

use nom::IResult;
use std::collections::HashMap;
use std::str;

pub fn parse_url_encoded(data: &[u8]) -> HashMap<String, String> {
    let mut form_data = HashMap::new();
    let mut data = parse_rec(data, &mut form_data).unwrap();
    while data != [] {
        data = parse_rec(data, &mut form_data).unwrap();
    }
    form_data
}

fn parse_rec<'a>(data: &'a [u8], map: &mut HashMap<String, String>) -> Result<&'a [u8], u32> {
    // TODO(nokaa): Actual error handling. Maybe just return the IResult
    // and let parse_url_encoded handle inserting
    match get_key(data) {
        IResult::Done(d, key) => {
            match get_value(d) {
                IResult::Done(d, val) => {
                    let key = str::from_utf8(key).unwrap();
                    let val = str::from_utf8(val).unwrap();
                    map.insert(key.to_string(), val.to_string());
                    return Ok(d);
                }
                IResult::Incomplete(_) => return Err(3),
                IResult::Error(_) => {
                    let (d, val) = nom::rest(d).unwrap();
                    let key = str::from_utf8(key).unwrap();
                    let val = str::from_utf8(val).unwrap();
                    map.insert(key.to_string(), val.to_string());
                    return Ok(d);
                }
            }
        }
        IResult::Incomplete(_) => return Err(1),
        IResult::Error(_) => return Err(2),
    };
}

/// Parse for the key, consuming all bytes read until the first `=`.
/// The consumed bytes are returned as the output. The `=` is then consumed
/// and the remainder of the data is returned as the input.
///
/// E.g. `get_key(b"key=value") -> IResult::Done(b"value", b"key")`
named!(get_key( &[u8] ) -> &[u8], take_until_and_consume!(b"="));
named!(get_value( &[u8] ) -> &[u8], take_until_and_consume!(b"&"));

#[cfg(test)]
mod tests {
    use super::parse_url_encoded;
    use std::collections::HashMap;

    #[test]
    fn test_one() {
        let data = b"key=val";

        let mut map = HashMap::new();
        map.insert(String::from("key"), String::from("val"));

        assert_eq!(parse_url_encoded(data), map);
    }

    #[test]
    fn test_two() {
        let data = b"key=val&key1=val1";

        let mut map = HashMap::new();
        map.insert(String::from("key"), String::from("val"));
        map.insert(String::from("key1"), String::from("val1"));

        assert_eq!(parse_url_encoded(data), map);
    }

    #[test]
    fn test_three() {
        let data = b"";

        let map = HashMap::new();

        assert_eq!(parse_url_encoded(data), map);
    }

    #[test]
    fn test_four() {
        let data = b"key=&key1=";

        let mut map = HashMap::new();
        map.insert(String::from("key"), String::from(""));
        map.insert(String::from("key1"), String::from(""));

        assert_eq!(parse_url_encoded(data), map);
    }
}
