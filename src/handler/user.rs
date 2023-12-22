use hex::FromHex;
use jwt_simple::prelude::*;
use siwe::{generate_nonce, Message, VerificationOpts};
use spin_sdk::http::{IntoResponse, Json};
use spin_sdk::key_value::Store;
use spin_sdk::variables;

use crate::handler;

#[derive(serde::Serialize)]
pub struct NonceResp {
    pub nonce: String,
}

// `GET /user/nonce/:wallet`
pub fn nonce(
    _: http::Request<()>,
    params: spin_sdk::http::Params,
) -> anyhow::Result<impl IntoResponse> {
    let wallet = params.get("wallet").unwrap();
    let nonce = generate_nonce();

    // save nonce to cache
    let store = Store::open_default()?;
    store.set(wallet, nonce.as_bytes())?;

    handler::json(&NonceResp { nonce })
}

#[derive(serde::Deserialize)]
pub struct LoginReq {
    pub wallet: String,
    pub message: String,
    pub signature: String,
}

#[derive(serde::Serialize)]
pub struct LoginResp {
    pub token: String,
}

// `POST /user/login`
pub async fn login(
    req: http::Request<Json<LoginReq>>,
    _: spin_sdk::http::Params,
) -> anyhow::Result<impl IntoResponse> {
    let wallet = req.body().wallet.as_str();
    let message = req.body().message.as_str();
    let signature = req.body().signature.as_str();

    // read nonce from cache
    let store = Store::open_default()?;
    let nonce = store.get(wallet)?;
    // check if nonce exists
    if nonce.is_none() {
        return Err(anyhow::anyhow!("nonce not found, please refresh it"));
    }

    // verify signature
    let m: Message = message.parse()?;
    let s = <[u8; 65]>::from_hex(signature.strip_prefix("0x").unwrap())?;
    let opt = VerificationOpts {
        // domain: Some("abiui.dev".parse().unwrap()),
        // domain: Some("localhost:5173".parse().unwrap()),
        domain: None,
        nonce: Some(String::from_utf8(nonce.unwrap()).unwrap().parse().unwrap()),
        timestamp: None,
    };
    if let Err(_) = m.verify(&s, &opt).await {
        return Err(anyhow::anyhow!("signature not correct"));
    }

    // generate jwt token
    let claims = Claims::create(Duration::from_days(1)).with_audience(wallet);
    let key = HS256Key::from_bytes(variables::get("jwt_secret")?.as_bytes());
    if let Ok(token) = key.authenticate(claims) {
        handler::json(&LoginResp { token })
    } else {
        Err(anyhow::anyhow!("server error"))
    }
}

#[derive(serde::Serialize)]
pub struct MeResp {
    wallet: String,
}

// `GET /user/me`
pub fn me(req: http::Request<()>, _: spin_sdk::http::Params) -> anyhow::Result<impl IntoResponse> {
    // if let Some(token) = req.headers().get("Authorization") {
    //     let key = HS256Key::from_bytes(variables::get("jwt_secret")?.as_bytes());
    //     if let Ok(claims) = key
    //         .verify_token::<NoCustomClaims>(token.to_str().unwrap().strip_prefix("Bearer ").unwrap(), None)
    //     {
    //         return handler::json(&MeResp {
    //             wallet: claims.audiences.unwrap().into_string().unwrap(),
    //         });
    //     }
    // }
    if let Ok(wallet) = handler::whoami(&req) {
        return handler::json(&MeResp { wallet });
    }

    handler::not_found()
}
