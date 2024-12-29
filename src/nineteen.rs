use std::{collections::HashMap, sync::Arc};

use poem::{
    delete, get, handler,
    http::StatusCode,
    post, put,
    web::{Data, Json, Path, Query},
    Endpoint, EndpointExt, Response, Route,
};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::FromRow,
    types::{
        chrono::{DateTime, Utc},
        Uuid,
    },
    PgPool,
};
use tokio::sync::RwLock;

pub async fn setup_table(pool: &PgPool) {
    if let Err(e) = sqlx::query!(
        r#"CREATE TABLE IF NOT EXISTS quotes (
        id UUID PRIMARY KEY,
        author TEXT NOT NULL,
        quote TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        version INT NOT NULL DEFAULT 1
        )"#
    )
    .execute(pool)
    .await
    {
        eprintln!("Couldn't create table: {e}");
    }
}

#[derive(Deserialize, Serialize, FromRow)]
struct Quote {
    #[serde(default = "Uuid::new_v4")]
    id: Uuid,
    author: String,
    quote: String,
    #[serde(default)]
    created_at: DateTime<Utc>,
    #[serde(default)]
    version: i32,
}

#[derive(Clone)]
struct State {
    pool: PgPool,
    pages: Arc<RwLock<HashMap<String, usize>>>,
}

#[handler]
async fn reset(Data(state): Data<&State>) -> Response {
    if sqlx::query("TRUNCATE quotes;")
        .fetch_all(&state.pool)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into();
    }
    StatusCode::OK.into()
}

#[handler]
async fn cite(Data(state): Data<&State>, Path(id): Path<Uuid>) -> Response {
    let Ok(quote) = sqlx::query_as!(Quote, "SELECT * FROM quotes where id = $1", id)
        .fetch_one(&state.pool)
        .await
    else {
        return StatusCode::NOT_FOUND.into();
    };

    serde_json::to_string(&quote)
        .expect("Should serialize")
        .into()
}

#[handler]
async fn remove(Data(state): Data<&State>, Path(id): Path<Uuid>) -> Response {
    let Ok(quote) = sqlx::query_as!(Quote, "DELETE FROM quotes where id = $1 RETURNING *", id)
        .fetch_one(&state.pool)
        .await
    else {
        return StatusCode::NOT_FOUND.into();
    };

    serde_json::to_string(&quote)
        .expect("Should serialize")
        .into()
}

#[handler]
async fn undo(
    Data(state): Data<&State>,
    Path(id): Path<Uuid>,
    Json(Quote { author, quote, .. }): Json<Quote>,
) -> Response {
    let Ok(quote) = sqlx::query_as!(
        Quote,
        "UPDATE quotes SET version = version + 1, author = $1, quote = $2 WHERE id = $3 RETURNING *",
        author,
        quote,
        id,
    )
    .fetch_one(&state.pool)
    .await
    else
    {
        return StatusCode::NOT_FOUND.into();
    };

    serde_json::to_string(&quote)
        .expect("Should serialize")
        .into()
}

#[handler]
async fn draft(
    Data(state): Data<&State>,
    Json(Quote {
        id, author, quote, ..
    }): Json<Quote>,
) -> Response {
    let Ok(quote) = sqlx::query_as!(
        Quote,
        "INSERT INTO quotes (id, author, quote) VALUES($1, $2, $3) RETURNING *",
        id,
        author,
        quote,
    )
    .fetch_one(&state.pool)
    .await
    else {
        return StatusCode::NOT_FOUND.into();
    };

    (
        StatusCode::CREATED,
        serde_json::to_string(&quote).expect("Should serialize"),
    )
        .into()
}

#[derive(Serialize)]
struct Quotes {
    quotes: Vec<Quote>,
    page: usize,
    next_token: Option<String>,
}

fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

#[derive(Deserialize)]
struct Token {
    token: Option<String>,
}

#[handler]
async fn list(Data(state): Data<&State>, Query(Token { token }): Query<Token>) -> Response {
    let page = match token {
        Some(token) => {
            let pages = state.pages.read().await;
            let Some(page) = pages.get(&token) else {
                return StatusCode::BAD_REQUEST.into();
            };
            *page
        }
        None => 0,
    };

    let Ok(quotes) = sqlx::query_as!(
        Quote,
        "SELECT * FROM quotes ORDER BY created_at LIMIT 3 OFFSET $1",
        (page * 3) as i64,
    )
    .fetch_all(&state.pool)
    .await
    else {
        return StatusCode::BAD_REQUEST.into();
    };

    #[derive(FromRow)]
    struct Response {
        count: Option<i64>,
    }
    let count = sqlx::query_as!(Response, "SELECT COUNT(*) FROM quotes")
        .fetch_one(&state.pool)
        .await
        .expect("The table should exist")
        .count
        .expect("There should be a count since table exists");

    let page = page + 1;
    let next_token = if count > page as i64 * 3 {
        let hex = generate_token();
        let mut pages = state.pages.write().await;
        pages.insert(hex.to_owned(), page);
        Some(hex)
    } else {
        None
    };

    let quotes = Quotes {
        quotes,
        page,
        next_token,
    };

    serde_json::to_string(&quotes)
        .expect("Should serialize")
        .into()
}

pub fn day_nineteen(pool: PgPool) -> impl Endpoint {
    Route::new()
        .at("/reset", post(reset))
        .at("/cite/:id", get(cite))
        .at("/remove/:id", delete(remove))
        .at("/undo/:id", put(undo))
        .at("/draft", post(draft))
        .at("/list", get(list))
        .data(State {
            pool,
            pages: Default::default(),
        })
}
