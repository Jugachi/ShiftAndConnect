use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{broadcast, Mutex};
use tower_http::cors::{Any, CorsLayer};

#[derive(Serialize, Deserialize, Clone)]
struct CreateGameReq {
    mode: String,
    is_private: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct GameRes {
    room_code: String,
    password: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct GameListItem {
    room_code: String,
    game_mode: String,
    is_private: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct JoinGameReq {
    password: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GameAction {
    action: String,
    row: Option<usize>,
    col: Option<usize>,
    direction: Option<String>,
    player: i32,
}

#[derive(Serialize, Deserialize, Clone)]
struct GameUpdate {
    board: Vec<Vec<i32>>,
    current_player: i32,
    winner: Option<i32>,
    is_closed: bool,
    player_count: usize,
}

struct AppState {
    db_pool: PgPool,
    rooms: Mutex<HashMap<String, broadcast::Sender<String>>>,
    lobby_tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    let (lobby_tx, _) = broadcast::channel(100);

    let shared_state = Arc::new(AppState {
        db_pool: pool.clone(),
        rooms: Mutex::new(HashMap::new()),
        lobby_tx: lobby_tx.clone(),
    });

    let cleanup_pool = pool.clone();
    let cleanup_tx = lobby_tx.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let result = sqlx::query!(
                "DELETE FROM games WHERE last_activity < NOW() - INTERVAL '5 minutes'"
            )
            .execute(&cleanup_pool)
            .await;
            if let Ok(res) = result {
                if res.rows_affected() > 0 {
                    println!(
                        "🧹 Cleanup: {} inaktive Spiele gelöscht.",
                        res.rows_affected()
                    );
                    let _ = cleanup_tx.send("update".to_string());
                }
            }
        }
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/games", get(list_games).post(create_game))
        .route("/api/games/:room_code/join", post(join_game))
        .route("/ws/lobby", get(ws_lobby_handler))
        .route("/ws/games/:room_code", get(ws_handler))
        .with_state(shared_state)
        .layer(cors);

    println!("🚀 Backend läuft auf http://127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


async fn list_games(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<GameListItem>>, StatusCode> {
    let games = sqlx::query_as!(
        GameListItem,
        "SELECT room_code, game_mode, is_private FROM games"
    )
    .fetch_all(&state.db_pool)
    .await
    .unwrap();
    Ok(Json(games))
}

async fn create_game(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateGameReq>,
) -> Result<Json<GameRes>, StatusCode> {
    let room_code: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect::<String>()
        .to_uppercase();
    let password = if payload.is_private {
        Some(
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(4)
                .map(char::from)
                .collect::<String>()
                .to_uppercase(),
        )
    } else {
        None
    };
    let empty_board = vec![vec![0; 7]; 7];

    let result = sqlx::query!(
        "INSERT INTO games (room_code, game_mode, board, current_player, is_private, password) VALUES ($1, $2, $3, $4, $5, $6)",
        room_code, payload.mode, serde_json::to_value(&empty_board).unwrap(), 1, payload.is_private, password
    ).execute(&state.db_pool).await;

    match result {
        Ok(_) => {
            let _ = state.lobby_tx.send("update".to_string());
            Ok(Json(GameRes {
                room_code,
                password,
            }))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn join_game(
    Path(room_code): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<JoinGameReq>,
) -> Result<StatusCode, StatusCode> {
    let game = sqlx::query!(
        "SELECT is_private, password FROM games WHERE room_code = $1",
        room_code
    )
    .fetch_optional(&state.db_pool)
    .await
    .unwrap();
    if let Some(g) = game {
        if g.is_private {
            if g.password == payload.password {
                Ok(StatusCode::OK)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        } else {
            Ok(StatusCode::OK)
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn ws_lobby_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |mut socket| async move {
        let mut rx = state.lobby_tx.subscribe();
        while let Ok(msg) = rx.recv().await {
            if socket.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    })
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(room_code): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, room_code, state))
}

async fn handle_socket(socket: WebSocket, room_code: String, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let tx = {
        let mut rooms = state.rooms.lock().await;
        rooms
            .entry(room_code.clone())
            .or_insert_with(|| broadcast::channel(100).0)
            .clone()
    };
    let mut rx = tx.subscribe();

    let state_init = state.clone();
    let room_init = room_code.clone();
    let tx_init = tx.clone();
    tokio::spawn(async move {
        let game = sqlx::query!("SELECT board, current_player FROM games WHERE room_code = $1", room_init).fetch_one(&state_init.db_pool).await;
        if let Ok(g) = game {
             let update = GameUpdate { 
                board: serde_json::from_value(g.board).unwrap(), 
                current_player: g.current_player, 
                winner: None, 
                is_closed: false,
                player_count: tx_init.receiver_count()
             };
             let _ = tx_init.send(serde_json::to_string(&update).unwrap());
        }
    });

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let tx_clone = tx.clone();
    let state_clone = state.clone();
    let room_code_task = room_code.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {

            if tx_clone.receiver_count() < 2 {
                println!("🚫 Zug ignoriert: Warten auf zweiten Spieler in Raum {}", room_code_task);
                continue; 
            }

            if let Ok(action) = serde_json::from_str::<GameAction>(&text) {
                let game = sqlx::query!(
                    "SELECT board, current_player FROM games WHERE room_code = $1",
                    room_code_task
                )
                .fetch_one(&state_clone.db_pool)
                .await;
                if let Ok(g) = game {
                    let mut board: Vec<Vec<i32>> = serde_json::from_value(g.board).unwrap();
                    let mut current_player = g.current_player;

                    process_action(&mut board, &action);
                    let winner = check_win(&board);
                    current_player = if current_player == 1 { 2 } else { 1 };

                    if winner.is_some() {
                        let _ =
                            sqlx::query!("DELETE FROM games WHERE room_code = $1", room_code_task)
                                .execute(&state_clone.db_pool)
                                .await;
                        let _ = state_clone.lobby_tx.send("update".to_string());
                    } else {
                        let _ = sqlx::query!("UPDATE games SET board = $1, current_player = $2, last_activity = NOW() WHERE room_code = $3", serde_json::to_value(&board).unwrap(), current_player, room_code_task).execute(&state_clone.db_pool).await;
                    }

                    let update = GameUpdate {
                        board,
                        current_player,
                        winner,
                        is_closed: winner.is_some(),
                        player_count: tx_clone.receiver_count()
                    };
                    let _ = tx_clone.send(serde_json::to_string(&update).unwrap());
                }
            }
        }
    });

    tokio::select! { _ = (&mut send_task) => recv_task.abort(), _ = (&mut recv_task) => send_task.abort() };
}

fn process_action(board: &mut Vec<Vec<i32>>, action: &GameAction) {
    match action.action.as_str() {
        "place" => {
            if let (Some(r), Some(c)) = (action.row, action.col) {
                if board[r][c] == 0 {
                    board[r][c] = action.player;
                }
            }
        }
        "shift_row" => {
            if let (Some(r), Some(dir)) = (action.row, &action.direction) {
                if dir == "right" {
                    let last = board[r].pop().unwrap();
                    board[r].insert(0, last);
                } else if dir == "left" {
                    let first = board[r].remove(0);
                    board[r].push(first);
                }
            }
        }
        "shift_col" => {
            if let (Some(c), Some(dir)) = (action.col, &action.direction) {
                if dir == "down" {
                    let last = board[6][c];
                    for i in (1..7).rev() {
                        board[i][c] = board[i - 1][c];
                    }
                    board[0][c] = last;
                } else if dir == "up" {
                    let first = board[0][c];
                    for i in 0..6 {
                        board[i][c] = board[i + 1][c];
                    }
                    board[6][c] = first;
                }
            }
        }
        _ => {}
    }
}
fn check_win(board: &Vec<Vec<i32>>) -> Option<i32> {
    for r in 0..7 {
        for c in 0..7 {
            let p = board[r][c];
            if p == 0 {
                continue;
            }
            let dirs = [(0, 1), (1, 0), (1, 1), (1, -1)];
            for (dr, dc) in dirs {
                let mut count = 1;
                for i in 1..5 {
                    let nr = r as i32 + dr * i;
                    let nc = c as i32 + dc * i;
                    if nr >= 0
                        && nr < 7
                        && nc >= 0
                        && nc < 7
                        && board[nr as usize][nc as usize] == p
                    {
                        count += 1;
                    } else {
                        break;
                    }
                }
                if count == 5 {
                    return Some(p);
                }
            }
        }
    }
    None
}
