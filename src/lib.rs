#[macro_use]
extern crate nom;
#[macro_use]
extern crate quick_error;

use nom::{alphanumeric, rest, IResult};
use std::collections::HashMap;
use std::str;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Parse(err: nom::Err) {
            description("parse error")
            display("Parse error: {:?}", err)
        }
    }
}

named!(key_value <&[u8], (&str, &str)>,
       do_parse!(
           key: map_res!(alphanumeric, str::from_utf8) >>
               tag!("=") >>
           val: map_res!(alphanumeric, str::from_utf8) >>
               alt_complete!(tag!("&") | rest) >>
           (key, val)
       )
);

named!(keys_and_values_aggregator<&[u8], Vec<(&str, &str)>>, many0!(key_value));

pub fn keys_and_values(input: &[u8]) -> IResult<&[u8], HashMap<&str, &str>> {
    let mut map = HashMap::new();
    match keys_and_values_aggregator(input) {
        IResult::Done(i, tuple_vec) => {
            for &(k, v) in &tuple_vec {
                map.insert(k, v);
            }
            IResult::Done(i, map)
        }
        IResult::Incomplete(a) => IResult::Incomplete(a),
        IResult::Error(a) => IResult::Error(a),
    }
}

#[cfg(test)]
mod tests {
    use super::keys_and_values;
    use nom::IResult;
    use std::collections::HashMap;

    #[test]
    fn test_one() {
        let data = b"key=val";
        let empty: &[u8] = &[];

        let mut map = HashMap::new();
        map.insert("key", "val");

        //let result = keys_and_values(data);
        //println!("{:?}", result);
        assert_eq!(keys_and_values(data), IResult::Done(empty, map));
    }

    #[test]
    fn test_two() {
        let data = b"key=val&key1=val1";
        let empty: &[u8] = &[];

        let mut map = HashMap::new();
        map.insert("key", "val");
        map.insert("key1", "val1");

        assert_eq!(keys_and_values(data), IResult::Done(empty, map));
    }

    #[test]
    fn test_three() {
        let data = b"";
        let empty: &[u8] = &[];

        let map = HashMap::new();

        assert_eq!(keys_and_values(data), IResult::Done(empty, map));
    }

    #[test]
    fn test_four() {
        let data = b"key=&key1=";
        let empty: &[u8] = &[];

        let mut map = HashMap::new();
        map.insert("key", "");
        map.insert("key1", "");
        let result = keys_and_values(data);
        println!("{:?}", result);

        //assert_eq!(keys_and_values(data), IResult::Done(empty, map));
        assert_eq!(result, IResult::Done(empty, map));
    }
}
