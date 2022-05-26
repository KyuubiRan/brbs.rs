use actix_web::{
    get,
    http::header::ContentType,
    web::{post, Bytes, Path, self},
    App, HttpResponse, HttpServer,
};

use json::object;
use log::debug;

use crate::{
    bili_requests, configs, db,
    enums::{self, Status},
    structs::User,
    utils::get_response_json,
};

fn make_json_http(json: String) -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(ContentType::json())
        .body(json)
}

fn act_success() -> HttpResponse {
    let s = object! {
        code: 200,
        msg: "执行成功"
    }
    .dump();
    make_json_http(s)
}

fn invalid_param() -> HttpResponse {
    let s = object! {
        code: 400,
        msg: "非法参数"
    }
    .dump();
    make_json_http(s)
}

fn internal_error() -> HttpResponse {
    let s = object! {
        code: 500,
        msg: "内部错误"
    }
    .dump();
    make_json_http(s)
}

async fn make_op(data: Bytes, op: enums::Status) -> HttpResponse {
    let json = match get_response_json(data) {
        Some(json) => json,
        _ => return invalid_param(),
    };

    let id = match json["uid"].as_i64() {
        Some(id) => id,
        None => return invalid_param(),
    };

    let key = match json["key"].as_str() {
        Some(key) => key,
        None => return invalid_param(),
    };

    let exec_role = match db::get_admin_key_role(key).await {
        Some(role) => role,
        None => return invalid_param(),
    };

    let reason = match json["reason"].as_str() {
        Some(key) => key,
        None => return invalid_param(),
    };

    debug!("Recv make black uid={id} key={key} reason={reason}");

    db::do_op(id, &op, &exec_role, reason).await;

    act_success()
}

fn query_result(user: User) -> HttpResponse {
    let ret = match user.status {
        Status::None => object! {
            code: 200,
            msg: "查询成功",
            data: {
                uid: user.uid,
                status: 0
            }
        }
        .dump(),
        Status::Black => object! {
            code: 200,
            msg: "查询成功",
            data: {
                uid: user.uid,
                status: 1,
                reason: user.last_reason.unwrap_or("无".to_owned())
            }
        }
        .dump(),
        Status::White => object! {
            code: 200,
            msg: "查询成功",
            data: {
                uid: user.uid,
                status: 2,
                reason: user.last_reason.unwrap_or("无".to_owned())
            }
        }
        .dump(),
    };

    make_json_http(ret)
}

/** 请求部分 **/

/*
GET /query/status/uid=123456
Response: {"code": 200, "data": {"status": 1, "reason": "评论区发送解析链接"}}
Status: 0: none, 1: black, 2: white
*/
#[get("/query/status/uid={uid}")]
async fn query_by_id(params: Path<String>) -> HttpResponse {
    let id = params.into_inner();

    let id = id.parse::<i64>();

    match id {
        Ok(id) => {
            debug!("Recv query by uid={id}");

            let user = db::get_user_by_id(id).await;

            query_result(user)
        }
        _ => invalid_param(),
    }
}

#[get("/query/status/key={key}")]
async fn query_by_key(params: Path<String>) -> HttpResponse {
    let key = params.into_inner();

    debug!("Recv query by key={key}");

    let uid = match bili_requests::get_uid_by_access_key(&key).await {
        Some(uid) => uid,
        _ => return invalid_param(),
    };

    let user = db::get_user_by_id(uid).await;

    query_result(user)
}

/*
Response: {"code": 200, "msg": "查询成功", "data": {"blackTimes": 3}}
*/
#[get("/query/times/uid={uid}")]
async fn query_black_times_by_id(params: Path<String>) -> HttpResponse {
    let id = params.into_inner();

    let id = id.parse::<i64>();

    match id {
        Ok(id) => {
            debug!("Recv query black times by uid={id}");

            let times = db::count_black_times(id).await;
            let ret = object! {
                code: 200,
                msg: "查询成功",
                data: { blackTimes: times }
            }
            .dump();
            make_json_http(ret)
        }
        _ => invalid_param(),
    }
}

/*
Response: {"code": 200, "msg": "查询成功", "data": {"blackTimes": 3}}
*/
#[get("/query/times/key={key}")]
async fn query_black_times_by_key(params: Path<String>) -> HttpResponse {
    let key = params.into_inner();

    let id = bili_requests::get_uid_by_access_key(&key).await;

    match id {
        Some(id) => {
            debug!("Recv query black times by key={key}({id})");

            let times = db::count_black_times(id).await;
            let ret = object! {
                code: 200,
                msg: "查询成功",
                data: { blackTimes: times }
            }
            .dump();
            make_json_http(ret)
        }
        _ => invalid_param(),
    }
}

/*
Request: {"uid": 123456, "key": "...", "reason": "..."}
Response: {"code": 200, "msg": "操作成功"}
*/
async fn make_black(data: Bytes) -> HttpResponse {
    make_op(data, enums::Status::Black).await
}

/*
Request: {"uid": 123456, "key": "...", "reason": "..."}
Response: {"code": 200, "msg": "操作成功"}
*/
async fn make_white(data: Bytes) -> HttpResponse {
    make_op(data, enums::Status::White).await
}

/*
Request: {"uid": 123456, "key": "...", "reason": "..."}
Response: {"code": 200, "msg": "操作成功"}
*/
async fn make_none(data: Bytes) -> HttpResponse {
    make_op(data, enums::Status::None).await
}

/*
Request: {"lvl": [0-127], "key": "...", "role": "..."}
Response: {"code": 200, "msg": "生成成功", "data": {"key":"..."}}
*/
async fn key_gen(data: Bytes) -> HttpResponse {
    let json = match get_response_json(data) {
        Some(json) => json,
        _ => return invalid_param(),
    };

    let key = match json["key"].as_str() {
        Some(s) => s,
        None => return invalid_param(),
    };

    let role = match json["role"].as_str() {
        Some(s) => s,
        None => return invalid_param(),
    };

    let lvl = json["lvl"].as_i8().unwrap_or(1);

    if !db::check_admin_key_with_lvl(key, 127).await {
        return invalid_param();
    }

    debug!("Recv key gen key={key}, role={role} lvl={lvl}");

    match db::gen_key(lvl, role).await {
        Some(k) => {
            let ret = object! {
                code: 200,
                msg: "生成成功",
                data: { key: k }
            }
            .dump();
            make_json_http(ret)
        }
        _ => internal_error(),
    }
}

/*
Request: {"key": "...", "role": "..."} or {"key": "...", "revokeKey": "..."}
Response: {"code": 200, "msg": "操作成功"}
*/
async fn key_revoke(data: Bytes) -> HttpResponse {
    let json = match get_response_json(data) {
        Some(json) => json,
        _ => return invalid_param(),
    };

    if let Some(key) = json["key"].as_str() {
        if let Some(rev) = json["revokeKey"].as_str() {
            if !db::check_admin_key_with_lvl(key, 127).await {
                return invalid_param();
            }
            db::revoke_admin_key_by_key(rev).await;
            debug!("Recv revoke key: {key}, revoked key: {rev}");
            return act_success();
        }

        if let Some(role) = json["role"].as_str() {
            if !db::check_admin_key_with_lvl(key, 127).await {
                return invalid_param();
            }
            db::revoke_admin_key_by_role(role).await;
            debug!("Recv revoke key: {key}, role: {role}");
            return act_success();
        }
    }
    invalid_param()
}

/*
Request: {"key": "..."}
Response: {"code": 200, "msg": "重新生成成功", "data": {"key":"..."}}
*/
async fn owner_key_regen(data: Bytes) -> HttpResponse {
    let json = match get_response_json(data) {
        Some(json) => json,
        _ => return invalid_param(),
    };

    match json["key"].as_str() {
        Some(s) => {
            if !db::check_admin_key_with_role_and_lvl(s, "owner", 127).await {
                return invalid_param();
            }
            let k = db::regen_owner_key(s).await;
            match k {
                Some(k) => {
                    debug!("Recv regen owner key: {s}, new key: {k}");
                    let ret = object! {
                        code: 200,
                        msg: "重新生成成功",
                        data: { key: k }
                    }
                    .dump();
                    make_json_http(ret)
                }
                _ => internal_error(),
            }
        }
        _ => invalid_param(),
    }
}

/*
Request: {"uid": 123456, "key": "..."}
Response: {"code": 200, "msg":"查询成功", "data": {"status": 1, "reason": "评论区发送解析链接", "opRole": "admin", "timestamp": 1653490177054}}
*/
async fn last_reason(data: Bytes) -> HttpResponse {
    let json = match get_response_json(data) {
        Some(json) => json,
        _ => return invalid_param(),
    };

    let id = match json["uid"].as_i64() {
        Some(id) => id,
        None => return invalid_param(),
    };

    let key = match json["key"].as_str() {
        Some(key) => key,
        None => return invalid_param(),
    };

    if !db::check_admin_key(key).await {
        return invalid_param();
    }

    debug!("Recv get last reason by key: {key}, uid: {id}");

    let r = match db::get_last_reason(id).await {
        Some(reason) => reason,
        _ => {
            let ret = object! {
                code: 200,
                msg: "无结果"
            }
            .dump();
            return make_json_http(ret);
        }
    };

    let op = &r.op;
    let op = op.into();
    let op_role = r.op_role;
    let reason = r.reason;
    let ts = r.op_time;

    let ret = object! {
        code: 200,
        msg: "查询成功",
        data: {
            status: op,
            opRole: op_role,
            reason: reason,
            timestamp: ts
        }
    }
    .dump();

    make_json_http(ret)
}

/*
Request: {"key": "..."}
Response: {"code": 200, "msg": "查询成功", "data": {"blackCount": 1000, "whiteCount": 10}}
*/
pub async fn statistics(data: Bytes) -> HttpResponse {
    let json = match get_response_json(data) {
        Some(json) => json,
        _ => return invalid_param(),
    };

    let key = match json["key"].as_str() {
        Some(key) => key,
        None => return invalid_param(),
    };

    if !db::check_admin_key(key).await {
        return invalid_param();
    }

    let black = db::count_total_by_status(&Status::Black).await;
    let white = db::count_total_by_status(&Status::White).await;

    debug!("Recv get statistics key: {key}");

    let ret = object! {
        code: 200,
        msg: "查询成功",
        data: {
            blackCount: black,
            whiteCount: white
        }
    }
    .dump();

    make_json_http(ret)
}

async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().body("")
}

pub async fn run_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(query_by_id)
            .service(query_by_key)
            .service(query_black_times_by_id)
            .service(query_black_times_by_key)
            .route("/admin/black", post().to(make_black))
            .route("/admin/white", post().to(make_white))
            .route("/admin/none", post().to(make_none))
            .route("/admin/last", post().to(last_reason))
            .route("/admin/statistics", post().to(statistics))
            .route("/owner/keygen", post().to(key_gen))
            .route("/owner/keyrevoke", post().to(key_revoke))
            .route("/owner/keyregen", post().to(owner_key_regen))
            .default_service(web::route().to(not_found))
    })
    .bind(("127.0.0.1", configs::SERVER_PORT))?
    .run()
    .await
}
