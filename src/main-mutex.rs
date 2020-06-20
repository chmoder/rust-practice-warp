extern crate redis;
extern crate r2d2_redis;

use argon2::{self, Config};
use rand::Rng;
use serde::Deserialize;
use warp::{http::StatusCode, Filter};
use std::sync::{Arc, Mutex};
use r2d2_redis::{r2d2, RedisConnectionManager};
use r2d2_redis::redis::Commands;

#[derive(Debug, Deserialize)]
struct User {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() {
    let manager = RedisConnectionManager::new("redis://127.0.0.1/").unwrap();
    let pool = r2d2::Pool::builder()
        .build(manager)
        .unwrap();
    // let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    // let con: redis::Connection = client.get_connection().map_err(|_| ()).unwrap();
    let db = Arc::new(Mutex::new(pool));
    let db = warp::any().map(move || Arc::clone(&db));

    let register = warp::post()
        .and(warp::path("register"))
        .and(warp::body::json())
        .and(db.clone())
        .and_then(register);
    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::body::json())
        .and(db.clone())
        .and_then(login);

    let routes = register.or(login);
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

async fn register(
    new_user: User,
    redis_pool: Arc<Mutex<r2d2::Pool<RedisConnectionManager>>>,
) -> Result<impl warp::Reply, warp::Rejection> {

    let  pool = redis_pool.lock().unwrap();
    let mut con = pool.get().unwrap();
    let exists: bool = con.exists(new_user.username.clone()).unwrap();
    // redis::cmd("SET").arg(&["key2", "bar"]).query_async(&mut con);

    if exists {
        return Ok(StatusCode::BAD_REQUEST);
    }

    let hashed_pass = hash(new_user.password.as_bytes());

    let _ : () = con.set(new_user.username.clone(), hashed_pass.clone()).unwrap();
    Ok(StatusCode::CREATED)
}

async fn login(
    credentials: User,
    redis_pool: Arc<Mutex<r2d2::Pool<RedisConnectionManager>>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let pool = redis_pool.lock().unwrap();
    let mut con = pool.get().unwrap();
    let hashed_password: String = con.get(credentials.username.clone()).unwrap_or_default();

    if hashed_password.len() > 0 {
        if verify(&hashed_password, credentials.password.as_bytes()) {
            return Ok(StatusCode::OK);
        } else {
            return Ok(StatusCode::UNAUTHORIZED);
        }
    } else {
        return Ok(StatusCode::BAD_REQUEST);
    }
}

pub fn hash(password: &[u8]) -> String {
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config).unwrap()
}

pub fn verify(hash: &str, password: &[u8]) -> bool {
    argon2::verify_encoded(hash, password).unwrap_or(false)
}

