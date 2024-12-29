use poem::{
    handler,
    http::StatusCode,
    post,
    web::{headers::ContentType, TypedHeader},
    Response, Result, Route,
};
use serde::Deserialize;
use serde_with::serde_as;

#[derive(Debug, Deserialize)]
struct Metadata {
    #[serde(default)]
    orders: Vec<Order>,
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct Order {
    item: String,
    #[serde_as(deserialize_as = "serde_with::DefaultOnError")]
    #[serde(default)]
    quantity: Option<u32>,
}

#[handler]
async fn manifest(
    data: String,
    TypedHeader(content_type): TypedHeader<ContentType>,
) -> Result<Response> {
    let manifest = match content_type.to_string().to_lowercase().as_str() {
        "application/json" => {
            serde_json::from_str::<cargo_manifest::Manifest<Metadata>>(&data).ok()
        }
        "application/toml" => toml::from_str(&data).ok(),
        "application/yaml" => serde_yml::from_str(&data).ok(),
        _ => {
            return Ok(Response::builder()
                .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
                .body(()))
        }
    };
    let Some(cargo_manifest::Manifest { package, .. }) = manifest else {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Invalid manifest"));
    };
    let metadata = match package {
        Some(package)
            if package
                .keywords
                .clone()
                .and_then(|k| k.as_local())
                .is_some_and(|k| k.iter().any(|s| s == "Christmas 2024")) =>
        {
            package.metadata
        }
        _ => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Magic keyword not provided"))
        }
    };

    let Some(Metadata { orders }) = metadata else {
        return Ok(StatusCode::NO_CONTENT.into());
    };

    let order_receipt = orders
        .iter()
        .filter_map(|Order { item, quantity }| {
            quantity.map(|quantity| format!("{item}: {quantity}"))
        })
        .collect::<Vec<String>>();

    if order_receipt.is_empty() {
        return Ok(StatusCode::NO_CONTENT.into());
    }

    Ok(order_receipt.join("\n").into())
}

pub fn day_five() -> Route {
    Route::new().at("/manifest", post(manifest))
}
