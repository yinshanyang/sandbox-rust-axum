use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
struct Message {
    id: Uuid,
    message: String,
}

#[derive(Debug, Deserialize)]
struct Payload {
    // without `Option` type, it’s a required field and we get response codes for free
    message: String,
    // // we can write it as an option too
    // message: Option<String>,
}

async fn list(State(db): State<Db>) -> impl IntoResponse {
    let messages = db.read().unwrap().values().cloned().collect::<Vec<_>>();

    Json(messages)
}

async fn get(Path(id): Path<Uuid>, State(db): State<Db>) -> Result<impl IntoResponse, StatusCode> {
    let message = db
        .read()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(message))
}

async fn post(State(db): State<Db>, Json(payload): Json<Payload>) -> impl IntoResponse {
    let message = Message {
        id: Uuid::new_v4(),
        message: payload.message,
    };

    db.write().unwrap().insert(message.id, message.clone());

    (StatusCode::CREATED, Json(message))
}

async fn update(
    Path(id): Path<Uuid>,
    State(db): State<Db>,
    Json(payload): Json<Payload>,
) -> Result<impl IntoResponse, StatusCode> {
    let message = db
        .read()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    let message = Message {
        id: message.id,
        message: payload.message,
    };

    db.write().unwrap().insert(message.id, message.clone());

    Ok(Json(message))
}

async fn delete(Path(id): Path<Uuid>, State(db): State<Db>) -> impl IntoResponse {
    if db.write().unwrap().remove(&id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

type Db = Arc<RwLock<HashMap<Uuid, Message>>>;

#[tokio::main]
async fn main() {
    let db = Db::default();

    let app = Router::with_state(db)
        .route("/", routing::get(list).post(post))
        .route("/:id", routing::get(get).patch(update).delete(delete));

    let address = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
