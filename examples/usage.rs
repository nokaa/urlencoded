extern crate urlencoded;

fn main() {
    let data = b"key=value";
    let map = urlencoded::parse_urlencoded(data);
    assert!(map.is_ok());
    let map = map.unwrap();
    let key = "key".to_string();
    let value = map.get(&key);
    assert!(value.is_some());
    assert_eq!(value.unwrap(), &"value".to_string());
}
