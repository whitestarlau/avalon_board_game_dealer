use std::collections::HashMap;
use std::convert::Infallible;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    response::sse::{Event, Sse},
    Json,
};
use futures::stream::{self, Stream};
use rand::Rng;
use serde::Serialize;

use crate::models::{
    role::Role,
    state::{
        AppState, CurrentGameResp, GameState, HistoryEntry, HistoryResp, NewGameReq,
        NewGameResp, PlayerInfo, PollRoleReq, PollRoleResp, ReadyReq, ReadyResp,
        SkillInfoStatusResp,
    },
};

#[derive(Serialize, Debug)]
pub struct AppError {
    pub error: String,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

pub async fn health_handler() -> Html<&'static str> {
    println!("some one call health check api.");
    Html("<h1>Goods server health ok.</h1>")
}

pub async fn new_game(
    State(app_state): State<AppState>,
    Query(query_params): Query<NewGameReq>,
) -> Result<Json<NewGameResp>, AppError> {
    let mut state = app_state.inner.write().await;

    // Save current game to history if any players have roles
    if !state.player_role_map.is_empty() {
        let snapshot = state.player_role_map.clone();
        state.history_role_map.push(snapshot);
    }

    let count = query_params.count.max(5).min(10);

    // Reset state
    state.player_ready_set.clear();
    state.player_role_map.clear();
    state.unassigned_role.clear();
    state.unassigned_role.extend(Role::role_pool(count));
    state.user_count = count;
    state.show_skill_info = true;
    state.game_counter += 1;

    Ok(Json(NewGameResp {
        des: format!("new game with {} players", count),
    }))
}

pub async fn player_ready(
    State(app_state): State<AppState>,
    Query(query_params): Query<ReadyReq>,
) -> Result<Json<ReadyResp>, AppError> {
    let number = query_params.number;

    let mut state = app_state.inner.write().await;

    if state.user_count == 0 {
        return Err(AppError {
            error: "GAME_NOT_STARTED".into(),
            message: "game not started, create a new game first".into(),
        });
    }

    if number < 1 || number > state.user_count as i32 {
        return Err(AppError {
            error: "INVALID_PLAYER_NUMBER".into(),
            message: format!("number must be between 1 and {}", state.user_count),
        });
    }

    if state.player_ready_set.contains(&number) {
        return Err(AppError {
            error: "ALREADY_READY".into(),
            message: format!("player {} already ready", number),
        });
    }

    if state.player_role_map.len() >= state.user_count {
        return Err(AppError {
            error: "GAME_ALREADY_OVER".into(),
            message: "game already over, start a new game".into(),
        });
    }

    state.player_ready_set.insert(number);
    gen_player_role(number, &mut state)
        .map_err(|e| AppError { error: "NO_ROLES_LEFT".into(), message: e })?;

    let game_complete = state.player_role_map.len() == state.user_count;
    if game_complete {
        let role_map = state.player_role_map.clone();
        let counter = state.game_counter;
        drop(state);
        save_game_to_history(&role_map, counter, &Vec::new()).await;
        let _ = app_state.game_complete_tx.send(());
    }

    Ok(Json(ReadyResp {
        number,
        ready: true,
    }))
}

fn gen_player_role(num: i32, state: &mut GameState) -> Result<i32, String> {
    if state.unassigned_role.is_empty() {
        return Err("no roles left to assign".to_string());
    }

    // Check if player had a role in the last game for future weighted selection
    let _last_faction: Option<&str> = state
        .history_role_map
        .last()
        .and_then(|last_map| last_map.get(&num))
        .map(|r| r.faction());

    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..state.unassigned_role.len());
    let role = state.unassigned_role.remove(index);
    state.player_role_map.insert(num, role.clone());

    Ok(0)
}

pub async fn poll_player_role(
    State(app_state): State<AppState>,
    Query(query_params): Query<PollRoleReq>,
) -> Result<Json<PollRoleResp>, AppError> {
    let state = app_state.inner.read().await;

    if state.player_role_map.len() < state.user_count {
        return Ok(Json(PollRoleResp {
            ready: false,
            role: String::new(),
            role_des: String::new(),
            skill_des: String::new(),
        }));
    }

    let role = state.player_role_map.get(&query_params.number)
        .ok_or_else(|| AppError { error: "PLAYER_NOT_FOUND".into(), message: "player not found".into() })?;

    let resp = build_poll_role_resp(role, &state.player_role_map, state.show_skill_info);
    Ok(Json(resp))
}

fn build_poll_role_resp(role: &Role, role_map: &HashMap<i32, Role>, show_skill_info: bool) -> PollRoleResp {
    let (role_name, skill_des) = match role {
        Role::Merlin => {
            let mut des = "邪恶方玩家有： ".to_string();
            for (num, p_role) in role_map.iter() {
                match p_role {
                    Role::Morgana | Role::Assassin | Role::Oberon | Role::MinionOfMordred(_) => {
                        des = format!("{} {}号", des, num);
                    }
                    _ => {}
                }
            }
            ("梅林".to_string(), des)
        }
        Role::Percival => {
            let mut des = "梅林和莫甘娜是：".to_string();
            for (num, p_role) in role_map.iter() {
                match p_role {
                    Role::Morgana | Role::Merlin => {
                        des = format!("{} {}号", des, num);
                    }
                    _ => {}
                }
            }
            ("派西维尔".to_string(), des)
        }
        Role::LoyalServant(_) => {
            ("忠臣".to_string(), String::new())
        }
        Role::Morgana => {
            let mut des = "邪恶同伴是：".to_string();
            for (num, p_role) in role_map.iter() {
                match p_role {
                    Role::Assassin | Role::Mordred | Role::MinionOfMordred(_) => {
                        des = format!("{} {}号", des, num);
                    }
                    _ => {}
                }
            }
            ("莫甘娜".to_string(), des)
        }
        Role::Assassin => {
            let mut des = "邪恶同伴是：".to_string();
            for (num, p_role) in role_map.iter() {
                match p_role {
                    Role::Morgana | Role::Mordred | Role::MinionOfMordred(_) => {
                        des = format!("{} {}号", des, num);
                    }
                    _ => {}
                }
            }
            ("刺客".to_string(), des)
        }
        Role::Oberon => {
            ("奥伯伦".to_string(), String::new())
        }
        Role::Mordred => {
            let mut des = "邪恶同伴是：".to_string();
            for (num, p_role) in role_map.iter() {
                match p_role {
                    Role::Morgana | Role::Assassin | Role::MinionOfMordred(_) => {
                        des = format!("{} {}号", des, num);
                    }
                    _ => {}
                }
            }
            ("莫德雷德".to_string(), des)
        }
        Role::MinionOfMordred(_) => {
            let mut des = "邪恶同伴是：".to_string();
            for (num, p_role) in role_map.iter() {
                match p_role {
                    Role::Morgana | Role::Assassin | Role::Mordred => {
                        des = format!("{} {}号", des, num);
                    }
                    _ => {}
                }
            }
            ("爪牙".to_string(), des)
        }
    };

    PollRoleResp {
        ready: true,
        role: role_name,
        role_des: role.description().to_string(),
        skill_des: if show_skill_info { skill_des } else { String::new() },
    }
}

async fn save_game_to_history(
    role_map: &HashMap<i32, Role>,
    game_id: i32,
    _history: &Vec<HashMap<i32, Role>>,
) {
    let mut players: Vec<PlayerInfo> = Vec::new();
    let mut sorted_numbers: Vec<i32> = role_map.keys().cloned().collect();
    sorted_numbers.sort();

    for num in sorted_numbers {
        let role = &role_map[&num];
        players.push(PlayerInfo {
            number: num,
            role: role.name_cn().to_string(),
            faction: role.faction().to_string(),
        });
    }

    let entry = HistoryEntry {
        id: game_id,
        timestamp: chrono::Local::now().to_rfc3339(),
        players,
    };

    // Read existing history from file, append, write back
    let path = std::path::Path::new("game_history.json");
    let mut history: Vec<HistoryEntry> = if path.exists() {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    history.push(entry);

    if let Ok(json) = serde_json::to_string_pretty(&history) {
        let _ = std::fs::write(path, json);
    }
}

pub async fn admin_current_game(
    State(app_state): State<AppState>,
) -> Result<Json<CurrentGameResp>, AppError> {
    let state = app_state.inner.read().await;

    let game_over = state.player_role_map.len() == state.user_count;

    let mut players: Vec<PlayerInfo> = Vec::new();
    let mut sorted_numbers: Vec<i32> = state.player_role_map.keys().cloned().collect();
    sorted_numbers.sort();

    for num in sorted_numbers {
        let role = &state.player_role_map[&num];
        players.push(PlayerInfo {
            number: num,
            role: role.name_cn().to_string(),
            faction: role.faction().to_string(),
        });
    }

    let all_numbers: Vec<i32> = (1..=state.user_count as i32).collect();
    let unready: Vec<i32> = all_numbers.into_iter()
        .filter(|n| !state.player_ready_set.contains(n))
        .collect();

    Ok(Json(CurrentGameResp {
        game_over,
        player_count: state.user_count,
        players,
        unready_numbers: unready,
    }))
}

pub async fn admin_history(
    State(_app_state): State<AppState>,
) -> Result<Json<HistoryResp>, AppError> {
    let path = std::path::Path::new("game_history.json");
    let games: Vec<HistoryEntry> = if path.exists() {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    Ok(Json(HistoryResp { games }))
}

pub async fn admin_skill_info_status(
    State(app_state): State<AppState>,
) -> Result<Json<SkillInfoStatusResp>, AppError> {
    let state = app_state.inner.read().await;
    Ok(Json(SkillInfoStatusResp {
        show_skill_info: state.show_skill_info,
    }))
}

pub async fn admin_toggle_skill_info(
    State(app_state): State<AppState>,
) -> Result<Json<SkillInfoStatusResp>, AppError> {
    let mut state = app_state.inner.write().await;
    state.show_skill_info = !state.show_skill_info;
    Ok(Json(SkillInfoStatusResp {
        show_skill_info: state.show_skill_info,
    }))
}

pub async fn sse_handler(
    State(app_state): State<AppState>,
    Query(query_params): Query<PollRoleReq>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let number = query_params.number;
    let state = app_state.clone();
    let mut game_rx = state.game_complete_tx.subscribe();

    let stream = stream::once(async move {
        loop {
            let is_complete = {
                let inner = state.inner.read().await;
                inner.player_role_map.len() >= inner.user_count
                    && inner.player_role_map.contains_key(&number)
            };

            if is_complete {
                let inner = state.inner.read().await;
                let role = inner.player_role_map.get(&number).unwrap();
                let resp = build_poll_role_resp(role, &inner.player_role_map, inner.show_skill_info);
                return Ok(Event::default().data(serde_json::to_string(&resp).unwrap()));
            }

            let _ = game_rx.changed().await;
        }
    });

    Sse::new(stream)
}
