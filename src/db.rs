use std::str::FromStr;

use log::{error, info};

use rand::Rng;
use sqlx::{sqlite::SqliteConnectOptions, Row, SqlitePool};

use crate::{
    configs::DATABASE_URL,
    enums::Status,
    structs::{Reason, User},
    utils,
};

lazy_static::lazy_static! {
    static ref POOL: SqlitePool = {
        SqlitePool::connect_lazy_with(SqliteConnectOptions::from_str(DATABASE_URL).unwrap().create_if_missing(true))
    };
}

fn gen_rand_key() -> String {
    let mut rng = rand::thread_rng();

    let mut key = String::new();

    for _ in 0..32 {
        let ch = rng.gen_range('a'..='z');
        let upper = rng.gen_bool(0.5);

        key.push(if upper { ch.to_ascii_uppercase() } else { ch });
    }

    key
}

pub async fn gen_key(lvl: i8, role: &str) -> Option<String> {
    let key = gen_rand_key();

    let mut db = POOL.begin().await.unwrap();

    let sql = r#"INSERT INTO keys (admin_key, lvl, role) VALUES ($1, $2, $3)"#;

    let ret = sqlx::query(sql)
        .bind(&key)
        .bind(lvl)
        .bind(role)
        .execute(&mut db)
        .await;

    match ret {
        Ok(_) => {
            info!("Successfully generated {role}(lvl:{lvl}) admin key: {key}");
            db.commit().await.unwrap();
            Some(key)
        }
        Err(e) => {
            error!("Cannot generate {role}(lvl:{lvl}) admin key with error: {e}");
            db.rollback().await.unwrap();
            None
        }
    }
}

pub async fn check_admin_key_with_lvl(key: &str, lvl: i8) -> bool {
    let mut db = POOL.acquire().await.unwrap();

    let sql = r#"SELECT lvl FROM keys WHERE admin_key = $1"#;

    let ret = sqlx::query(sql).bind(key).fetch_optional(&mut db).await;

    match ret {
        Ok(Some(r)) => {
            let l: i8 = r.get(0);
            l >= lvl
        }
        _ => false,
    }
}

pub async fn check_admin_key(key: &str) -> bool {
    check_admin_key_with_lvl(key, 0).await
}

pub async fn get_admin_key_role(key: &str) -> Option<String> {
    let mut db = POOL.acquire().await.unwrap();

    let sql = r#"SELECT role FROM keys WHERE admin_key = $1"#;

    let ret = sqlx::query(sql).bind(key).fetch_optional(&mut db).await;

    match ret {
        Ok(Some(r)) => Some(r.get(0)),
        _ => None,
    }
}

async fn gen_admin_key() {
    let mut db = POOL.acquire().await.unwrap();

    let sql = r#"SELECT admin_key FROM keys WHERE lvl = 127"#;

    let ret = sqlx::query(sql).fetch_optional(&mut db).await;

    match ret {
        Ok(Some(row)) => {
            let key: String = row.get(0);
            info!("Admin key already exists: {}", key)
        }
        _ => {
            gen_key(127, "admin").await;
        }
    };
}

pub async fn revoke_admin_key_by_role(role: &str) {
    let mut db = POOL.begin().await.unwrap();

    let sql = r#"DELETE FROM keys WHERE role = $1"#;

    let ret = sqlx::query(sql).bind(role).execute(&mut db).await;

    match ret {
        Ok(_) => {
            info!("Successfully revoked admin key of {role}");
            db.commit().await.unwrap();
        }
        Err(e) => {
            error!("Cannot revoke admin key of {role} with error: {e}");
            db.rollback().await.unwrap();
        }
    }
}

pub async fn revoke_admin_key_by_key(key: &str) {
    let mut db = POOL.begin().await.unwrap();

    let sql = r#"DELETE FROM keys WHERE admin_key = $1"#;

    let ret = sqlx::query(sql).bind(key).execute(&mut db).await;

    match ret {
        Ok(_) => {
            info!("Successfully revoked admin key: {key}");
            db.commit().await.unwrap();
        }
        Err(e) => {
            error!("Cannot revoke admin key: {key} with error: {e}");
            db.rollback().await.unwrap();
        }
    }
}

pub async fn prepare() {
    info!("Start prepare database");

    {
        let mut db = POOL.acquire().await.unwrap();

        let sql = r#"CREATE TABLE IF NOT EXISTS users
        (
            uid         BIGINT PRIMARY KEY,
            status      SMALLINT NOT NULL DEFAULT 0,
            last_reason TEXT
        )"#;

        sqlx::query(sql).execute(&mut db).await.unwrap();

        let sql = r#"CREATE TABLE IF NOT EXISTS reasons
        (
            id      INTEGER PRIMARY KEY AUTOINCREMENT,
            uid     BIGINT   NOT NULL,
            op      SMALLINT NOT NULL DEFAULT 0,
            op_role TEXT     NOT NULL DEFAULT "admin",
            reason  TEXT,
            op_time BIGINT   NOT NULL DEFAULT 0
        )"#;

        sqlx::query(sql).execute(&mut db).await.unwrap();

        let sql = r#"CREATE TABLE IF NOT EXISTS keys
        (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            admin_key VARCHAR(32) NOT NULL,
            lvl        SMALLINT    NOT NULL DEFAULT 1,
            role       Text        NOT NULL
        )"#;

        sqlx::query(sql).execute(&mut db).await.unwrap();
    }

    gen_admin_key().await;

    info!("Finish prepare database");
}

pub async fn get_user_by_id(uid: i64) -> User {
    let mut db = POOL.acquire().await.unwrap();

    let sql = r#"SELECT * FROM users WHERE uid = $1;"#;

    let row = sqlx::query(sql).bind(uid).fetch_optional(&mut db).await;

    match row {
        Ok(Some(r)) => User {
            uid,
            status: Status::from(r.get(1)),
            last_reason: r.try_get(2).unwrap_or(None),
        },
        _ => User {
            uid,
            status: Status::None,
            last_reason: None,
        },
    }
}

pub async fn get_last_reason(uid: i64) -> Option<Reason> {
    let mut db = POOL.acquire().await.unwrap();

    let sql = r#"SELECT * FROM (SELECT * FROM reasons ORDER BY ID DESC) WHERE uid = $1 LIMIT 1"#;

    let ret = sqlx::query(sql).bind(uid).fetch_optional(&mut db).await;

    match ret {
        Ok(Some(r)) => Some(Reason {
            uid,
            op: Status::from(r.get(2)),
            op_role: r.try_get(3).unwrap_or("无".to_owned()),
            reason: r.try_get(4).unwrap_or("无".to_owned()),
            op_time: r.get(5),
        }),
        _ => None,
    }
}

pub async fn count_black_times(uid: i64) -> i64 {
    let mut db = POOL.acquire().await.unwrap();

    let sql = r#"SELECT COUNT(*) FROM reasons WHERE uid = $1 AND op = 1"#;

    let ret = sqlx::query(sql).bind(uid).fetch_optional(&mut db).await;

    match ret {
        Ok(Some(r)) => {
            let i: i64 = r.get(0);
            i
        }
        _ => 0,
    }
}

pub async fn do_op(uid: i64, op: &Status, op_role: &str, reason: &str) {
    {
        let mut db = POOL.begin().await.unwrap();

        let sql = r#"INSERT OR REPLACE INTO users (uid, status, last_reason) VALUES ($1, $2, $3)"#;

        let ret = sqlx::query(sql)
            .bind(uid)
            .bind(op.into())
            .bind(reason)
            .execute(&mut db)
            .await;

        match ret {
            Ok(_) => {
                info!("User {uid} is {} now", op.display());
                db.commit().await.unwrap();
            }
            Err(e) => {
                error!("Cannot {} user {uid} with error: {e}", op.display());
                db.rollback().await.unwrap();
                return;
            }
        }
    }

    {
        let mut db = POOL.begin().await.unwrap();

        let sql = r#"INSERT INTO reasons (uid, op, op_role, reason, op_time) VALUES ($1, $2, $3, $4, $5)"#;

        let ret = sqlx::query(sql)
            .bind(uid)
            .bind(op.into())
            .bind(op_role)
            .bind(reason)
            .bind(utils::current_milliseconds())
            .execute(&mut db)
            .await;

        match ret {
            Ok(_) => {
                info!(
                    "Successfully added records where uid={uid}, op={}, reason={reason}",
                    op.display()
                );
                db.commit().await.unwrap();
            }
            Err(e) => {
                error!(
                    "Cannot add records where uid={uid}, op={}, reason={reason} with error: {e}",
                    op.display(),
                );
                db.rollback().await.unwrap();
            }
        }
    }
}
