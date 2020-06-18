// based on https://github.com/joshua-cooper/warp-auth-server
extern crate r2d2_redis;

use argon2::{self, Config};
use rand::Rng;
use serde::Deserialize;
use warp::{http::StatusCode, Filter};
use r2d2_redis::{r2d2, RedisConnectionManager};
use r2d2_redis::redis::Commands;

#[derive(Debug, Deserialize)]
struct User {
    username: String,
    password: String,
}

/// This is where our warp app is defined and started.  You can POST
/// to either /register or /login like this:
/// ```
/// HTTP
/// POST /register HTTP/1.1
/// Host: 192.168.1.2:3030
///
/// {
///     "username": "tcross",
///     "password": "password"
/// }
/// ```
#[tokio::main]
async fn main() {
    let manager = RedisConnectionManager::new("redis://127.0.0.1/").unwrap();
    let pool = r2d2::Pool::builder()
        .build(manager)
        .unwrap();
    let warp_pool = warp::any().map(move || pool.clone());

    let register = warp::post()
        .and(warp::path("register"))
        .and(warp::body::json())
        .and(warp_pool.clone())
        .and_then(register);
    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::body::json())
        .and(warp_pool.clone())
        .and_then(login);

    let routes = register.or(login);
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

/// This method will take the user credentials filled into this type by
/// warp.  Then hash the password and save it to redis.
async fn register(
    new_user: User,
    pool: r2d2::Pool<RedisConnectionManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut con = pool.get().unwrap();
    let exists: bool = con.exists(new_user.username.clone()).unwrap();

    if exists {
        return Ok(StatusCode::BAD_REQUEST);
    }

    let hashed_pass = hash(new_user.password.as_bytes());

    let _ : () = con.set(new_user.username.clone(), hashed_pass.clone()).unwrap();
    Ok(StatusCode::CREATED)
}

/// A User object was created using warp filter.  It has username and password
/// from the JSON request body.
/// This method checks checks that the password matches the hashed
/// password that was stored in redis by `register`
async fn login(
    credentials: User,
    pool: r2d2::Pool<RedisConnectionManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
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
