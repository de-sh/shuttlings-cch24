use askama_escape::{escape, Html};
use hex::decode;
use poem::{
    get, handler,
    http::StatusCode,
    post,
    web::{Multipart, Path},
    Response, Route,
};
use serde::Deserialize;

#[handler]
fn star() -> Response {
    r#"<div id="star" class="lit"></div>"#.into()
}

#[handler]
fn present(Path(color): Path<String>) -> Response {
    let next = match color.as_str() {
        "red" => "blue",
        "blue" => "purple",
        "purple" => "red",
        _ => return StatusCode::IM_A_TEAPOT.into(),
    };

    format!(
        r#"<div class="present {color}" hx-get="/23/present/{next}" hx-swap="outerHTML"><div class="ribbon"></div><div class="ribbon"></div><div class="ribbon"></div><div class="ribbon"></div></div>"#
    )
    .into()
}

#[handler]
fn ornament(Path((state, n)): Path<(String, String)>) -> Response {
    let next_state = match state.as_str() {
        "on" => "off",
        "off" => "on",
        _ => return StatusCode::IM_A_TEAPOT.into(),
    };
    let n = escape(&n, Html);

    format!(
        r#"<div class="ornament{}" id="ornament{n}" hx-trigger="load delay:2s once" hx-get="/23/ornament/{next_state}/{n}" hx-swap="outerHTML"></div>"#,
        match state.as_str() {
            "on" => " on",
            _ => "",
        }
    )
    .into()
}

#[derive(Deserialize, Debug)]
struct Lockfile {
    #[serde(rename = "package")]
    packages: Vec<Package>,
}

#[derive(Deserialize, Debug)]
struct Package {
    checksum: Option<String>,
}

#[handler]
async fn lockfile(mut multipart: Multipart) -> Response {
    let mut output = String::default();
    let Ok(Some(field)) = multipart.next_field().await else {
        return StatusCode::BAD_REQUEST.into();
    };
    let Ok(data) = field.bytes().await else {
        return StatusCode::BAD_REQUEST.into();
    };
    let Ok(data) = std::str::from_utf8(&data) else {
        return StatusCode::BAD_REQUEST.into();
    };
    let Ok(Lockfile { packages }) = toml::from_str::<Lockfile>(data) else {
        return StatusCode::BAD_REQUEST.into();
    };
    for package in packages {
        let Some(checksum) = &package.checksum else {
            continue;
        };
        if checksum.len() < 10 {
            return StatusCode::UNPROCESSABLE_ENTITY.into();
        }
        if decode(checksum).is_err() {
            return StatusCode::UNPROCESSABLE_ENTITY.into();
        }
        let color = &checksum[0..6];
        let top = u32::from_str_radix(&checksum[6..8], 16).expect("Hex code");
        let left = u32::from_str_radix(&checksum[8..10], 16).expect("Hex code");

        output.push_str(&format!(
            "<div style=\"background-color:#{color};top:{top}px;left:{left}px;\"></div>"
        ));
    }

    output.push('\n');
    output.into()
}

pub fn day_twentythree() -> Route {
    Route::new()
        .at("/star", get(star))
        .at("/present/:color", get(present))
        .at("/ornament/:state/:n", get(ornament))
        .at("/lockfile", post(lockfile))
}
