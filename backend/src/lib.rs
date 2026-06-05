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
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

pub mod handler;
pub mod models;

use crate::{
    handler::rest::{
        admin_current_game, admin_history, health_handler, new_game, player_ready,
        poll_player_role,
    },
    models::{role::Role, state::AppState},
};

static SHUTDOWN_SENDER: Lazy<tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>> =
    Lazy::new(|| tokio::sync::Mutex::new(None));

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
    let user_count: Arc<RwLock<usize>> = Arc::new(RwLock::new(0));
    let play_role_map: Arc<RwLock<HashMap<i32, Role>>> = Arc::new(RwLock::new(HashMap::new()));
    let ready_player_set: Arc<RwLock<HashSet<i32>>> = Arc::new(RwLock::new(HashSet::new()));
    let history_player_role: Arc<RwLock<Vec<HashMap<i32, Role>>>> =
        Arc::new(RwLock::new(Vec::new()));
    let unassigned_role: Arc<RwLock<Vec<Role>>> = Arc::new(RwLock::new(Vec::new()));
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

pub async fn start_server(port: u16) -> Result<String, String> {
    let ip = get_local_ip();
    let app = build_app();
    let addr = format!("0.0.0.0:{}", port);

    let listener = std::net::TcpListener::bind(&addr)
        .map_err(|e| format!("bind failed: {}", e))?;
    listener.set_nonblocking(true).ok();

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    {
        let mut sender = SHUTDOWN_SENDER.lock().await;
        *sender = Some(tx);
    }

    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app.into_make_service())
            .with_graceful_shutdown(async {
                rx.await.ok();
            })
            .await
            .ok();
    });

    Ok(format!("http://{}:{}", ip, port))
}

pub async fn stop_server() {
    let mut sender = SHUTDOWN_SENDER.lock().await;
    if let Some(tx) = sender.take() {
        let _ = tx.send(());
    }
}

// ── JNI exports (Android only) ──

#[cfg(target_os = "android")]
pub mod android {
    use super::*;
    use jni::JNIEnv;
    use jni::objects::JClass;
    use jni::sys::jint;
    use std::sync::Mutex;
    use once_cell::sync::Lazy;

    static RUNTIME: Lazy<Mutex<Option<tokio::runtime::Runtime>>> =
        Lazy::new(|| Mutex::new(None));

    #[no_mangle]
    pub extern "system" fn Java_com_avalon_dealer_Server_startServer(
        _env: JNIEnv,
        _class: JClass,
        port: jint,
    ) -> jni::sys::jstring {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = rt.block_on(start_server(port as u16));

        let mut runtime_guard = RUNTIME.lock().unwrap();
        *runtime_guard = Some(rt);

        let output = match result {
            Ok(url) => url,
            Err(e) => format!("ERROR:{}", e),
        };

        // Return string via JNI
        let env = _env;
        let output_str = env.new_string(&output).expect("Failed to create Java string");
        output_str.into_raw()
    }

    #[no_mangle]
    pub extern "system" fn Java_com_avalon_dealer_Server_stopServer(
        _env: JNIEnv,
        _class: JClass,
    ) {
        let rt = {
            let mut guard = RUNTIME.lock().unwrap();
            guard.take()
        };
        if let Some(rt) = rt {
            rt.block_on(stop_server());
        }
    }
}
