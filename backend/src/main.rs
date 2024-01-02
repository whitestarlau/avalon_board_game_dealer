use std::{
    collections::{HashMap, HashSet},
    env,
    sync::Arc,
    vec,
};

// #[macro_use]
// extern crate lazy_static;

use axum::{
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

use crate::{
    handler::rest::{health_handler, player_ready},
    models::{role::Role, state::AppState},
};

#[path = "./models/mod.rs"]
mod models;

#[path = "./handler/mod.rs"]
mod handler;

fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        println!("start web_server in main.");
        web_server().await;
    });
}

async fn web_server() {
    dotenv().ok();

    //目前暂时只支持7人模式,后续通过环境变量来配置
    // let user_count_str = env::var("All_Player_Count").expect("All_Player_Count should be set.");
    // let user_count: usize = user_count_str.parse().unwrap();
    let user_count: usize = 7;

    let play_role_map: Arc<RwLock<HashMap<i32, Role>>> = Arc::new(RwLock::new(HashMap::new()));
    let ready_player_set: Arc<RwLock<HashSet<i32>>> = Arc::new(RwLock::new(HashSet::new()));
    let history_player_role: Arc<RwLock<Vec<HashMap<i32, Role>>>> =
        Arc::new(RwLock::new(Vec::new()));
    let unassigned_role: Arc<RwLock<Vec<Role>>> = Arc::new(RwLock::new(vec![
        Role::Merlin,
        Role::Percival,
        Role::LS_of_Arthur(1),
        Role::LS_of_Arthur(2),
        Role::Morgana,
        Role::Assassin,
        Role::Oberon,
    ]));

    let app_state = AppState {
        user_count: user_count,
        player_role_map: play_role_map,
        player_ready_set: ready_player_set,
        history_role_map: history_player_role,
        unassigned_role: unassigned_role,
    };

    let health_check_path = "/health_check";

    // build our application with a route
    let rest = Router::new()
        .route(health_check_path, get(health_handler))
        .route("/ready", get(player_ready))
        // .route("/poll_player_role", post(get_goods_detail))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // run it
    let addr = "127.0.0.1:3004";
    println!("listening on {}", addr);

    //向consul中心注册自己
    // tokio::spawn(register_consul(&addr, health_check_path));

    axum::Server::bind(&addr.parse().unwrap())
        .serve(rest.into_make_service())
        .await
        .unwrap();
}
