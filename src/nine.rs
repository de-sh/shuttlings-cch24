use std::{sync::Arc, time::Duration};

use leaky_bucket::RateLimiter;
use poem::{
    handler,
    http::StatusCode,
    post,
    web::{headers::ContentType, Data, Json, TypedHeader},
    EndpointExt, IntoEndpoint, Response, Route,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum Metric {
    Liters(f32),
    Gallons(f32),
    Litres(f32),
    Pints(f32),
}

impl Metric {
    fn convert(self) -> Self {
        match self {
            Self::Gallons(g) => Self::Liters(g / 0.264172),
            Self::Liters(l) => Self::Gallons(l * 0.264172),
            Self::Litres(l) => Self::Pints(l / 0.568261),
            Self::Pints(p) => Self::Litres(p * 0.568261),
        }
    }
}

fn fresh_rates() -> RateLimiter {
    RateLimiter::builder()
        .initial(5)
        .max(5)
        .interval(Duration::from_secs(1))
        .build()
}

#[handler]
async fn milk(
    Data(rate_limiter): Data<&Arc<RwLock<RateLimiter>>>,
    content_type: Option<TypedHeader<ContentType>>,
    body: Option<Json<Metric>>,
) -> Response {
    let response = if rate_limiter.read().await.try_acquire(1) {
        (StatusCode::OK, "Milk withdrawn\n").into()
    } else {
        return (StatusCode::TOO_MANY_REQUESTS, "No milk available\n").into();
    };

    match content_type {
        Some(TypedHeader(content_type)) if content_type == ContentType::json() => {
            if let Some(Json(request)) = body {
                let response = request.convert();
                serde_json::to_string(&response).unwrap().into()
            } else {
                (StatusCode::BAD_REQUEST).into()
            }
        }
        _ => response,
    }
}

#[handler]
async fn refill(Data(rate_limiter): Data<&Arc<RwLock<RateLimiter>>>) -> Response {
    let mut guard = rate_limiter.write().await;
    *guard = fresh_rates();
    StatusCode::OK.into()
}

pub fn day_nine() -> impl IntoEndpoint {
    Route::new()
        .at("/milk", post(milk))
        .at("/refill", post(refill))
        .data(Arc::new(RwLock::new(fresh_rates())))
}
