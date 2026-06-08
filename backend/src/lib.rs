use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::{
    body::Body,
    http::{header, Request},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;
use std::sync::Mutex;
use tokio::sync::{RwLock, watch};
use tower_http::cors::CorsLayer;

pub mod handler;
pub mod models;

use crate::{
    handler::rest::{
        admin_current_game, admin_history, admin_skill_info_status,
        admin_toggle_skill_info, health_handler, new_game, player_ready,
        poll_player_role, sse_handler,
    },
    models::state::{AppState, GameState},
};

// Global shutdown signal: stored outside tokio runtime for JNI access
static SHUTDOWN_SENDER: Lazy<Mutex<Option<tokio::sync::oneshot::Sender<()>>>> =
    Lazy::new(|| Mutex::new(None));

#[derive(RustEmbed)]
#[folder = "static"]
struct Assets;

async fn serve_embedded(req: Request<Body>) -> impl IntoResponse {
    let path = req.uri().path().trim_start_matches('/').to_string();
    let filename = if path.is_empty() || path.starts_with("api/") {
        "index.html".to_string()
    } else {
        path
    };

    let content_type = |path: &str| -> &'static str {
        if path.ends_with(".html") {
            "text/html; charset=utf-8"
        } else if path.ends_with(".js") {
            "application/javascript"
        } else if path.ends_with(".css") {
            "text/css"
        } else if path.ends_with(".svg") {
            "image/svg+xml"
        } else if path.ends_with(".png") {
            "image/png"
        } else if path.ends_with(".ico") {
            "image/x-icon"
        } else if path.ends_with(".json") {
            "application/json"
        } else {
            "text/plain"
        }
    };

    match Assets::get(&filename) {
        Some(content) => {
            let ct = content_type(&filename);
            ([(header::CONTENT_TYPE, ct)], content.data)
        }
        None => match Assets::get("index.html") {
            Some(c) => ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], c.data),
            None => (
                [(header::CONTENT_TYPE, "text/plain")],
                std::borrow::Cow::Owned(b"Not found".to_vec()),
            ),
        },
    }
}

pub fn build_app() -> Router {
    let game_state = GameState {
        user_count: 0,
        player_role_map: HashMap::new(),
        player_ready_set: HashSet::new(),
        unassigned_role: Vec::new(),
        history_role_map: Vec::new(),
        game_counter: 0,
        show_skill_info: true,
    };
    let (game_complete_tx, _) = watch::channel(());
    let app_state = AppState {
        inner: Arc::new(RwLock::new(game_state)),
        game_complete_tx: Arc::new(game_complete_tx),
    };

    let api_routes = Router::new()
        .route("/health_check", get(health_handler))
        .route("/ready", get(player_ready))
        .route("/sse", get(sse_handler))
        .route("/poll_player_role", post(poll_player_role))
        .route("/new_game", post(new_game))
        .route("/admin/current_game", get(admin_current_game))
        .route("/admin/history", get(admin_history))
        .route("/admin/skill_info_status", get(admin_skill_info_status))
        .route("/admin/toggle_skill_info", post(admin_toggle_skill_info))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    Router::new()
        .nest("/api", api_routes)
        .fallback(serve_embedded)
}

pub fn get_local_ip() -> String {
    for (name, ip) in local_ip_address::list_afinet_netifas().unwrap_or_default() {
        if !name.starts_with("lo")
            && !name.starts_with("docker")
            && !name.starts_with("utun")
        {
            if let std::net::IpAddr::V4(ipv4) = ip {
                if ipv4.is_private() {
                    return ipv4.to_string();
                }
            }
        }
    }
    "127.0.0.1".to_string()
}

/// Run the server on the CURRENT thread (blocking). Returns only after shutdown.
async fn run_server_forever(port: u16) {
    let app = build_app();
    let addr = format!("0.0.0.0:{}", port);

    let listener = std::net::TcpListener::bind(&addr).expect("bind failed");
    listener.set_nonblocking(true).ok();

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    {
        let mut sender = SHUTDOWN_SENDER.lock().unwrap();
        *sender = Some(tx);
    }

    axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            rx.await.ok();
        })
        .await
        .ok();
}

pub fn stop_server() {
    let mut sender = SHUTDOWN_SENDER.lock().unwrap();
    if let Some(tx) = sender.take() {
        let _ = tx.send(());
    }
}

// ── Desktop / CLI entry ──

pub fn start_server_blocking(port: u16) -> String {
    let ip = get_local_ip();
    let rt = tokio::runtime::Runtime::new().unwrap();
    // Spawn the server on an extra thread so ctrl-c handler can still fire
    std::thread::spawn(move || {
        rt.block_on(run_server_forever(port));
    });
    format!("http://{}:{}", ip, port)
}

// ── JNI exports (Android only) ──

#[cfg(target_os = "android")]
pub mod android {
    use super::*;
    use jni::objects::JClass;
    use jni::sys::jint;
    use jni::JNIEnv;

    #[no_mangle]
    pub extern "system" fn Java_com_avalon_dealer_Server_startServer(
        env: JNIEnv,
        _class: JClass,
        port: jint,
    ) -> jni::sys::jstring {
        let ip = get_local_ip();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(run_server_forever(port as u16));
        });

        let url = format!("http://{}:{}", ip, port);
        let output = env.new_string(&url).expect("Failed to create string");
        output.into_raw()
    }

    #[no_mangle]
    pub extern "system" fn Java_com_avalon_dealer_Server_stopServer(
        _env: JNIEnv,
        _class: JClass,
    ) {
        stop_server();
    }
}
