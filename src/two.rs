use std::{
    net::{Ipv4Addr, Ipv6Addr},
    ops::BitXor,
};

use poem::{get, handler, web::Query, Response, Route};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Destination {
    from: String,
    key: String,
}

#[handler]
fn dest(Query(Destination { from, key: ip }): Query<Destination>) -> Response {
    let from_octets = from.parse::<Ipv4Addr>().unwrap().octets();
    let key_octets = ip.parse::<Ipv4Addr>().unwrap().octets();

    Ipv4Addr::from(
        from_octets
            .iter()
            .zip(key_octets.iter())
            .fold(0_u32, |x, (a, b)| (x << 8) + b.overflowing_add(*a).0 as u32),
    )
    .to_string()
    .into()
}

#[handler]
fn dest_v6(Query(Destination { from, key: ip }): Query<Destination>) -> Response {
    let from_segments = from.parse::<Ipv6Addr>().unwrap().segments();
    let key_segments = ip.parse::<Ipv6Addr>().unwrap().segments();

    Ipv6Addr::from(
        from_segments
            .iter()
            .zip(key_segments.iter())
            .fold(0_u128, |x, (a, b)| (x << 16) + b.bitxor(*a) as u128),
    )
    .to_string()
    .into()
}

#[derive(Debug, Deserialize)]
struct Key {
    from: String,
    to: String,
}

#[handler]
fn key(Query(Key { from, to }): Query<Key>) -> Response {
    let from_octets = from.parse::<Ipv4Addr>().unwrap().octets();
    let to_octets = to.parse::<Ipv4Addr>().unwrap().octets();

    Ipv4Addr::from(
        from_octets
            .iter()
            .zip(to_octets.iter())
            .fold(0_u32, |x, (a, b)| (x << 8) + b.overflowing_sub(*a).0 as u32),
    )
    .to_string()
    .into()
}

#[handler]
fn key_v6(Query(Key { from, to }): Query<Key>) -> Response {
    let from_segments = from.parse::<Ipv6Addr>().unwrap().segments();
    let to_segments = to.parse::<Ipv6Addr>().unwrap().segments();

    Ipv6Addr::from(
        from_segments
            .iter()
            .zip(to_segments.iter())
            .fold(0_u128, |x, (a, b)| (x << 16) + b.bitxor(*a) as u128),
    )
    .to_string()
    .into()
}

pub fn day_two() -> Route {
    Route::new()
        .at("/dest", get(dest))
        .at("/key", get(key))
        .nest(
            "/v6",
            Route::new()
                .at("/dest", get(dest_v6))
                .at("/key", get(key_v6)),
        )
}
