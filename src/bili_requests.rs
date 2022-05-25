use crate::utils::{current_milliseconds, get_response_json};

const APP_KEY: &str = "1d8b6e7d45233436";
const APP_SEC: &str = "560c52ccd288fed045859ed18bffd973";

pub async fn get_uid_by_access_key(key: &str) -> Option<i64> {
    let ts = current_milliseconds() / 1000;
    let sign_str = format!("access_key={key}&appkey={APP_KEY}&client=android&ts={ts}{APP_SEC}");
    let sign = format!("{:x}", md5::compute(&sign_str));

    let url = format!("https://app.bilibili.com/x/v2/account/myinfo?access_key={key}&appkey={APP_KEY}&ts={ts}&client=android&sign={sign}");

    let ret = reqwest::get(url).await.ok()?;

    let ret = ret.bytes().await.ok()?;
    let json = get_response_json(ret);

    match json {
        Some(json) => json["data"]["mid"].as_i64(),
        _ => None,
    }
}
