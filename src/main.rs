use five::day_five;
use nine::day_nine;
use nineteen::{day_nineteen, setup_table};
use poem::{
    endpoint::StaticFileEndpoint,
    get, handler,
    http::{HeaderValue, StatusCode},
    Endpoint, Response, Route,
};
use shuttle_poem::ShuttlePoem;
use sixteen::day_sixteen;
use twelve::day_twelve;
use twentythree::day_twentythree;
use two::day_two;

mod five;
mod nine;
mod nineteen;
mod sixteen;
mod twelve;
mod twentythree;
mod two;

#[handler]
fn hello_world() -> &'static str {
    "Hello, bird!"
}

#[handler]
fn redirect() -> Response {
    let mut resp = Response::default();
    resp.set_status(StatusCode::FOUND);
    resp.headers_mut().insert(
        "Location",
        HeaderValue::from_static("https://www.youtube.com/watch?v=9Gc4QTqslN4"),
    );

    resp
}

#[shuttle_runtime::main]
async fn poem(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://user_1PhqNcttxB0h:cGFMftVcgtpyAuIXBzLsRAGbpic2Fgac@sharedpg-rds.shuttle.dev:5432/db_1PhqNcttxB0h"
    )]
    pool: sqlx::PgPool,
) -> ShuttlePoem<impl Endpoint> {
    setup_table(&pool).await;

    let app = Route::new()
        .at("/", get(hello_world))
        .at("/-1/seek", get(redirect))
        .nest("/2", day_two())
        .nest("/5", day_five())
        .nest("/9", day_nine())
        .nest("/12", day_twelve())
        .nest("/16", day_sixteen())
        .nest("/19", day_nineteen(pool))
        .nest("/23", day_twentythree())
        .nest(
            "/assets/23.html",
            StaticFileEndpoint::new("./assets/23.html"),
        );

    Ok(app.into())
}
