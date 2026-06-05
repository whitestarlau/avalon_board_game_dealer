use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::{
    http::StatusCode,
    routing::{get, get_service, post},
    Router,
};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::{
    handler::rest::{
        admin_current_game, admin_history, health_handler, new_game, player_ready,
        poll_player_role,
    },
    models::{role::Role, state::AppState},
};

mod handler;
mod models;

fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        println!("start web_server in main.");
        web_server().await;
    });
}

async fn web_server() {
    let user_count: usize = 7;

    let play_role_map: Arc<RwLock<HashMap<i32, Role>>> = Arc::new(RwLock::new(HashMap::new()));
    let ready_player_set: Arc<RwLock<HashSet<i32>>> = Arc::new(RwLock::new(HashSet::new()));
    let history_player_role: Arc<RwLock<Vec<HashMap<i32, Role>>>> =
        Arc::new(RwLock::new(Vec::new()));
    let unassigned_role: Arc<RwLock<Vec<Role>>> = Arc::new(RwLock::new(vec![
        Role::Merlin,
        Role::Percival,
        Role::LoyalServant(1),
        Role::LoyalServant(2),
        Role::Morgana,
        Role::Assassin,
        Role::Oberon,
    ]));
    let game_counter: Arc<RwLock<i32>> = Arc::new(RwLock::new(0));

    let app_state = AppState {
        user_count,
        player_role_map: play_role_map,
        player_ready_set: ready_player_set,
        history_role_map: history_player_role,
        unassigned_role,
        game_counter,
    };

    let api_routes = Router::new()
        .route("/health_check", get(health_handler))
        .route("/ready", get(player_ready))
        .route("/poll_player_role", post(poll_player_role))
        .route("/new_game", post(new_game))
        .route("/admin/current_game", get(admin_current_game))
        .route("/admin/history", get(admin_history))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback(
            get_service(ServeDir::new("../backend/static"))
                .handle_error(|error: std::io::Error| async move {
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("{error}"))
                }),
        );

    let addr = "127.0.0.1:3004";
    println!("listening on {}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
