# rust-practice-warp
Example warp rust server with redis


### Setup:
1) Install redis server
2) Install rustup https://rustup.rs/
3) clone this repo
4) `cargo run --package warp-server-redis --bin warp-server-redis --release`

### Usage:
```
POST /register HTTP/1.1
Host: 192.168.1.2:3030
Content-Type: application/json

{
    "username": "tcross",
    "password": "password"
}
```

```
POST /login HTTP/1.1
Host: 192.168.1.2:3030
Content-Type: application/json

{
    "username": "tcross",
    "password": "password"
}
```
