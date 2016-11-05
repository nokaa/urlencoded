extern crate urlencoded;

use urlencoded::parse_urlencoded;

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
