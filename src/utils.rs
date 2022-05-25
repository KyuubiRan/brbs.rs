use std::time::SystemTime;

use actix_web::web::Bytes;
use json::JsonValue;

pub fn current_milliseconds() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

pub fn get_response_json(data: Bytes) -> Option<JsonValue> {
    let parse_data = match String::from_utf8(data.to_vec()) {
        Ok(data) => data,
        _ => return None,
    };

    json::parse(&parse_data).ok()
}