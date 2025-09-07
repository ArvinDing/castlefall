use axum::{
    extract::{Query, State},
    response::{sse::{Event, Sse}, IntoResponse},
    routing::get,
    Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap},
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

#[derive(Clone)]
struct Room {
    names: Vec<String>,
    round_num: i32,
    words: Vec<String>,
    assignments: Assignments,
    tx: broadcast::Sender<String>,
}

impl Room {
    fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self {
            names: vec![],
            round_num: 0,
            words: vec![],
            assignments: Assignments::new(),
            tx,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
struct Assignments {
    pairs: Vec<(String, String)>, // (player, word)
}

use rand::seq::SliceRandom;
use rand::thread_rng;

impl Assignments {
    fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    fn assign_random_words(&mut self, words: &[String], players: &[String]) {
        if words.len() < 2 {
            panic!("Need at least 2 words to assign players");
        }

        let mut rng = thread_rng();

        // Pick two random words
        let chosen_words: Vec<&String> = words.choose_multiple(&mut rng, 2).collect();
        let word1 = chosen_words[0].clone();
        let word2 = chosen_words[1].clone();

        // Shuffle players
        let mut shuffled_players = players.to_vec();
        shuffled_players.shuffle(&mut rng);

        let half = (shuffled_players.len() + 1) / 2; // round up if odd number

        // Assign first half to word1, second half to word2
        for player in shuffled_players.iter().take(half) {
            self.pairs.push((player.clone(), word1.clone()));
        }
        for player in shuffled_players.iter().skip(half) {
            self.pairs.push((player.clone(), word2.clone()));
        }
    }

    fn remove_player(&mut self, player: &str) {
        self.pairs.retain(|(p, _)| p != player);
    }
}


#[derive(Clone)]
struct AppState {
    rooms: Arc<Mutex<HashMap<String, Room>>>, // global mutex
}

#[derive(Serialize)]
struct RoomStateView {
    round_num: i32,
    names: Vec<String>,
    word_list: Vec<String>,
    assignments: Assignments,
}

#[derive(Deserialize)]
struct JoinParams {
    name: String,
    room: String,
}

#[derive(Deserialize)]
struct KickParams {
    name: String,
    room: String,
}

#[derive(Deserialize)]
struct NextRoundParams {
    room: String,
}

use axum::Json;

// ========================= Handlers =========================

async fn join_handler(
    State(state): State<AppState>,
    Query(params): Query<JoinParams>,
) -> Json<RoomStateView> {
    let mut rooms = state.rooms.lock().unwrap();
    let room = rooms.entry(params.room.clone()).or_insert_with(Room::new);

    if !room.names.contains(&params.name) {
        room.names.push(params.name.clone());
    }

    let _ = room.tx.send(format!("JOIN, {}, {}", params.name, params.room));

    let view = RoomStateView {
        round_num: room.round_num,
        names: room.names.clone(),
        word_list: room.words.clone(),
        assignments: room.assignments.clone(),
    };

    Json(view)
}

async fn kick_handler(
    State(state): State<AppState>,
    Query(params): Query<KickParams>,
) -> impl IntoResponse {
    let mut rooms = state.rooms.lock().unwrap();

    if let Some(room) = rooms.get_mut(&params.room) {
        if let Some(pos) = room.names.iter().position(|n| n == &params.name) {
            room.names.remove(pos);
            room.assignments.remove_player(&params.name);

            let _ = room.tx.send(format!("KICK, {}, {}", params.name, params.room));

            return format!(
                "{} removed. Room {} now has members: {:?}",
                params.name, params.room, room.names
            );
        } else {
            return format!("{} is not in room {}", params.name, params.room);
        }
    }

    format!("Room {} does not exist", params.room)
}

async fn start_round_handler(
    State(state): State<AppState>,
    Query(params): Query<NextRoundParams>,
) -> impl IntoResponse {
    let mut rooms = state.rooms.lock().unwrap();

    if let Some(room) = rooms.get_mut(&params.room) {
        room.round_num += 1;

        room.words = vec![
            "apple", "banana", "cherry", "date", "elderberry", "fig", "grape",
            "honeydew", "kiwi", "lemon", "mango", "nectarine", "orange", "papaya",
            "quince", "raspberry", "strawberry", "tangerine"
        ]
        .into_iter()
        .map(String::from)
        .collect();

        room.assignments = Assignments::new();
        room.assignments.assign_random_words(&room.words, &room.names);
        let _ = room.tx.send(format!("ROUND_START, {}", params.room));

        format!("Round started in room {}! Round number: {} Assignments: {:?}", params.room, room.round_num, room.assignments)
    } else {
        format!("Room {} does not exist", params.room)
    }
}

async fn events_handler(
    State(state): State<AppState>,
    Query(params): Query<JoinParams>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let rooms = state.rooms.lock().unwrap();
    let room = rooms.get(&params.room).expect("Room must exist");

    let mut rx = room.tx.subscribe();

    let stream = async_stream::stream! {
        while let Ok(msg) = rx.recv().await {
            yield Ok(Event::default().data(msg));
        }
    };

    Sse::new(stream)
}

// ========================= Main =========================

#[tokio::main]
async fn main() {
    let state = AppState {
        rooms: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/join", get(join_handler))
        .route("/kick", get(kick_handler))
        .route("/events", get(events_handler))
        .route("/next_round", get(start_round_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}/", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
