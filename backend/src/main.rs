use axum::{
    extract::{Query, State},
    response::{sse::{Event, Sse}, IntoResponse},
    routing::get,
    Router,
};
use futures::stream::{Stream, StreamExt};
use serde::Deserialize;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

#[derive(Debug)]
struct Room {
    names: Vec<String>,
}

#[derive(Clone)]
struct AppState {
    rooms: Arc<Mutex<HashMap<String, Room>>>,
    tx: broadcast::Sender<String>, // broadcaster
}

#[derive(Deserialize)]
struct JoinParams {
    name: String,
    room: String,
}

#[tokio::main]
async fn main() {
    let (tx, _rx) = broadcast::channel(100);

    let state = AppState {
        rooms: Arc::new(Mutex::new(HashMap::new())),
        tx,
    };

    let app = Router::new()
        .route("/join", get(join_handler))
        .route("/events", get(events_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}/", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn join_handler(
    State(state): State<AppState>,
    Query(params): Query<JoinParams>,
) -> impl IntoResponse {
    let mut rooms = state.rooms.lock().unwrap();

    let room = rooms.entry(params.room.clone()).or_insert(Room { names: vec![] });
    if !room.names.contains(&params.name) {
        room.names.push(params.name.clone());
    }

    let msg = format!("{} joined room {}", params.name, params.room);
    println!("{}", msg);

    // Broadcast to all listening clients
    let _ = state.tx.send(msg.clone());

    format!("Room {} now has members: {:?}", params.room, room.names)
}

async fn events_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let mut rx = state.tx.subscribe();

    let stream = async_stream::stream! {
        while let Ok(msg) = rx.recv().await {
            yield Ok(Event::default().data(msg));
        }
    };

    Sse::new(stream)
}

// player join
// round start
// start the shot clock
// kick player
// reset score for everyone