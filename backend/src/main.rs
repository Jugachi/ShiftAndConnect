use axum::{
    extract::{Path, State, WebSocketUpgrade, ws::{WebSocket, Message}},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, Mutex};
use tower_http::cors::{Any, CorsLayer};

// --- DATENMODELLE (SERDE) ---

#[derive(Serialize, Deserialize, Clone)]
struct CreateGameReq {
    mode: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct GameRes {
    room_code: String,
}

// Dies repräsentiert eine Aktion vom Frontend (z.B. Shift oder Platzieren)
#[derive(Serialize, Deserialize, Clone, Debug)]
struct GameAction {
    action_type: String, // "place", "shift_row", "shift_col"
    row: Option<usize>,
    col: Option<usize>,
    direction: Option<String>,
    player: i32,
}

// --- GLOBALER STATE ---
// Hält die Datenbank-Verbindung und die aktiven WebSocket-Kanäle pro Raum
struct AppState {
    db_pool: PgPool,
    // Ein Broadcast-Channel pro Raum-Code, um Nachrichten an beide Spieler zu senden
    rooms: Mutex<HashMap<String, broadcast::Sender<String>>>,
}

#[tokio::main]
async fn main() {
    // 1. Datenbank-Verbindung aufbauen (Daten aus docker-compose.yml)
    let database_url = "postgres://shift_user:super_secret_password@127.0.0.1:5432/shift_connect";
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Konnte nicht mit der Datenbank verbinden!");

    let shared_state = Arc::new(AppState {
        db_pool: pool,
        rooms: Mutex::new(HashMap::new()),
    });

    // 2. CORS konfigurieren (Erlaubt Anfragen von deinem Vue Frontend)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 3. Router definieren
    let app = Router::new()
        .route("/api/games", post(create_game))
        .route("/ws/games/:room_code", get(ws_handler))
        .with_state(shared_state)
        .layer(cors);

    // 4. Server starten
    println!("🚀 Backend läuft auf http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- REST API: SPIEL ERSTELLEN ---
async fn create_game(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateGameReq>,
) -> Result<Json<GameRes>, StatusCode> {
    // Zufälligen 6-stelligen Code generieren (z.B. "A8F3X1")
    let room_code: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect::<String>()
        .to_uppercase();

    // Leeres 7x7 Array erstellen
    let empty_board = vec![vec![0; 7]; 7];
    let board_json = serde_json::to_value(&empty_board).unwrap();

    // In die Postgres DB speichern
    let result = sqlx::query!(
        "INSERT INTO games (room_code, game_mode, board, current_player) VALUES ($1, $2, $3, $4)",
        room_code,
        payload.mode,
        board_json,
        1
    )
    .execute(&state.db_pool)
    .await;

    match result {
        Ok(_) => Ok(Json(GameRes { room_code })),
        Err(e) => {
            eprintln!("DB Fehler: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// --- WEBSOCKET HANDLER ---
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(room_code): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Wenn sich jemand verbindet, rüsten wir die Verbindung zu einem WebSocket hoch
    ws.on_upgrade(move |socket| handle_socket(socket, room_code, state))
}

async fn handle_socket(socket: WebSocket, room_code: String, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Raum-Channel holen oder neu erstellen
    let mut rooms = state.rooms.lock().await;
    let tx = rooms
        .entry(room_code.clone())
        .or_insert_with(|| {
            let (tx, _rx) = broadcast::channel(100);
            tx
        })
        .clone();
    drop(rooms); // Mutex sofort wieder loslassen

    let mut rx = tx.subscribe();

    // Task 1: Nachrichten vom Channel an diesen Client senden
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Task 2: Nachrichten vom Client empfangen und in den Channel broadcasten
    let tx_clone = tx.clone();
    
    // HIER IST DER FIX: Wir klonen den room_code für den Task
    let room_code_task = room_code.clone(); 
    
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Hier nutzen wir den geklonten String
            println!("Nachricht empfangen im Raum {}: {}", room_code_task, text);
            
            let _ = tx_clone.send(text);
        }
    });

    // Warten bis eine der Verbindungen abbricht
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
    
    // Hier können wir das originale room_code wieder problemlos nutzen!
    println!("Spieler hat den Raum {} verlassen.", room_code);
}