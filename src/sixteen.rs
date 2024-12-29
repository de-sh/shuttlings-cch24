use std::collections::HashSet;

use jsonwebtoken::{
    decode, encode, errors::ErrorKind, get_current_timestamp, Algorithm, DecodingKey, EncodingKey,
    Header, Validation,
};
use poem::{
    get, handler,
    http::StatusCode,
    post,
    web::{headers::Cookie, Json, TypedHeader},
    Response, Route,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const SECRET: &[u8] = b"not-so-secret";
const SANTAS_PUB_KEY: &[u8] = include_bytes!("../assets/day16_santa_public_key.pem");

#[derive(Serialize, Deserialize)]
struct Claims {
    payload: Value,
    exp: u64,
}

#[handler]
async fn wrap(Json(payload): Json<Value>) -> Response {
    let Ok(gift) = encode(
        &Header::default(),
        &Claims {
            payload,
            exp: get_current_timestamp(),
        },
        &EncodingKey::from_secret(SECRET),
    ) else {
        return StatusCode::BAD_REQUEST.into();
    };
    Response::builder()
        .header("Set-Cookie", format!("gift={gift}"))
        .body(())
}

#[handler]
async fn unwrap(TypedHeader(jwt): TypedHeader<Cookie>) -> Response {
    let Some(gift) = jwt.get("gift") else {
        return StatusCode::BAD_REQUEST.into();
    };
    let Ok(data) = decode::<Claims>(
        gift,
        &DecodingKey::from_secret(SECRET),
        &Validation::default(),
    ) else {
        return StatusCode::BAD_REQUEST.into();
    };

    data.claims.payload.to_string().into()
}

#[handler]
async fn decode_handler(token: String) -> Response {
    let mut validation = Validation::default();
    validation.algorithms = vec![Algorithm::RS256, Algorithm::RS512]; // algorithms were tracked down with help of some prompting
    validation.required_spec_claims = HashSet::default(); // don't adhere to spec

    match decode::<Value>(
        &token,
        &DecodingKey::from_rsa_pem(SANTAS_PUB_KEY).expect("Public key corrupted"),
        &validation,
    ) {
        Ok(data) => data.claims.to_string().into(),
        Err(e) if *e.kind() == ErrorKind::InvalidSignature => StatusCode::UNAUTHORIZED.into(),
        _ => StatusCode::BAD_REQUEST.into(),
    }
}

pub fn day_sixteen() -> Route {
    Route::new()
        .at("/wrap", post(wrap))
        .at("/unwrap", get(unwrap))
        .at("/decode", post(decode_handler))
}
