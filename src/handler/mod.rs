pub mod contract;
pub mod user;

use jwt_simple::prelude::*;
use spin_sdk::variables;

fn whoami<T>(req: &http::Request<T>) -> anyhow::Result<String> {
    let token = req.headers().get("Authorization");
    if token.is_none() || token.unwrap().to_str().unwrap_or_default().is_empty() {
        return Err(anyhow::anyhow!(""));
    }

    let token = token
        .unwrap()
        .to_str()
        .unwrap()
        .strip_prefix("Bearer ")
        .unwrap_or_default();
    let key = HS256Key::from_bytes(variables::get("jwt_secret").unwrap().as_bytes());
    if let Ok(claims) = key.verify_token::<NoCustomClaims>(token, None) {
        return Ok(claims.audiences.unwrap().into_string().unwrap());
    }

    return Err(anyhow::anyhow!(""));
}

fn json<T: serde::Serialize>(data: &T) -> anyhow::Result<spin_sdk::http::Response> {
    Ok(spin_sdk::http::Response::builder()
        .status(http::StatusCode::OK)
        .header("content-type", "application/json")
        .header("Access-Control-Allow-Headers", "*")
        .header("Access-Control-Allow-Origin", "*") // cors
        .body(serde_json::to_string(data).unwrap())
        .build())
}

fn ok() -> anyhow::Result<spin_sdk::http::Response> {
    json(&())
}

fn not_found() -> anyhow::Result<spin_sdk::http::Response> {
    json(&())
}
