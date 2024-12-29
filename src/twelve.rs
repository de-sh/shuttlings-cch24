use std::{fmt, sync::Arc};

use poem::{
    get, handler,
    http::StatusCode,
    post,
    web::{Data, Path},
    EndpointExt, IntoEndpoint, Response, Route,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use tokio::sync::RwLock;

#[derive(Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Team {
    #[default]
    Empty,
    Cookie,
    Milk,
}

impl From<bool> for Team {
    fn from(value: bool) -> Self {
        if value {
            Self::Cookie
        } else {
            Self::Milk
        }
    }
}

#[derive(Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum Column {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "⬛",
            Self::Milk => "🥛",
            Self::Cookie => "🍪",
        })
    }
}

struct Board {
    inner: [[Team; 4]; 4],
    rng: StdRng,
}

impl Board {
    fn check_winner(&self) -> Option<Team> {
        if (0..4).any(|i| {
            // Check if any row is completely Cookie
            self.inner[i].iter().all(|j| j == &Team::Cookie)
            // Check if any column is completely Cookie
                || self.inner.iter().all(|j| j[i] == Team::Cookie)
            // Check if either diagonal is completely Cookie
        }) || (0..4).all(|i| self.inner[i][i] == Team::Cookie)
            || (0..4).all(|i| self.inner[i][3 - i] == Team::Cookie)
        {
            return Some(Team::Cookie);
        }
        // Do a similar check for Milk
        if (0..4).any(|i| {
            self.inner[i].iter().all(|j| j == &Team::Milk)
                || self.inner.iter().all(|j| j[i] == Team::Milk)
        }) || (0..4).all(|i| self.inner[i][i] == Team::Milk)
            || (0..4).all(|i| self.inner[i][3 - i] == Team::Milk)
        {
            return Some(Team::Milk);
        }

        // Check if empty slots are still available, if
        if (0..4).any(|i| self.inner[i].iter().any(|j| j == &Team::Empty)) {
            return Some(Team::Empty);
        }

        None
    }
}

impl Default for Board {
    fn default() -> Self {
        Board {
            inner: Default::default(),
            rng: StdRng::seed_from_u64(2024),
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..4 {
            writeln!(
                f,
                "⬜{}⬜",
                self.inner[row]
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<String>()
            )?;
        }

        // Print who wins if anyone does
        write!(
            f,
            "⬜⬜⬜⬜⬜⬜\n{}",
            match self.check_winner() {
                Some(Team::Milk) => "🥛 wins!\n",
                Some(Team::Cookie) => "🍪 wins!\n",
                None => "No winner.\n",
                _ => "",
            }
        )
    }
}

#[handler]
async fn board(Data(shared): Data<&Arc<RwLock<Board>>>) -> Response {
    shared.read().await.to_string().into()
}

#[handler]
async fn reset(Data(shared): Data<&Arc<RwLock<Board>>>) -> Response {
    let mut table = shared.write().await;
    *table = Board::default();
    table.to_string().into()
}

#[handler]
async fn place(
    Data(shared): Data<&Arc<RwLock<Board>>>,
    Path((team, column)): Path<(Team, Column)>,
) -> Response {
    let table = shared.read().await;
    match table.check_winner() {
        Some(Team::Milk) | Some(Team::Cookie) | None => {
            return (StatusCode::SERVICE_UNAVAILABLE, table.to_string()).into()
        }
        // Empty slots are still available, continue play
        _ => {}
    }
    for row in (0..4).rev() {
        if table.inner[row][column as usize - 1] == Team::Empty {
            drop(table);
            let mut table = shared.write().await;
            table.inner[row][column as usize - 1] = team;
            return table.to_string().into();
        }
    }

    (StatusCode::SERVICE_UNAVAILABLE, table.to_string()).into()
}

#[handler]
async fn random_board(Data(shared): Data<&Arc<RwLock<Board>>>) -> Response {
    let mut table = shared.write().await;
    for row in 0..4 {
        for column in 0..4 {
            table.inner[row][column] = table.rng.gen::<bool>().into();
        }
    }
    table.to_string().into()
}

pub fn day_twelve() -> impl IntoEndpoint {
    Route::new()
        .at("/board", get(board))
        .at("/reset", post(reset))
        .at("/place/:team/:column", post(place))
        .at("/random-board", get(random_board))
        .data(Arc::new(RwLock::new(Board::default())))
}
