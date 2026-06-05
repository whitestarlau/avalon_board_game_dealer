use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    Json,
};
use rand::Rng;

use crate::models::{
    role::Role,
    state::{
        AppState, CurrentGameResp, HistoryEntry, HistoryResp, NewGameReq, NewGameResp,
        PlayerInfo, PollRoleReq, PollRoleResp, ReadyReq, ReadyResp,
    },
};

pub async fn health_handler() -> Html<&'static str> {
    println!("some one call health check api.");
    Html("<h1>Goods server health ok.</h1>")
}

pub async fn new_game(
    State(app_state): State<AppState>,
    Query(query_params): Query<NewGameReq>,
) -> Result<Json<NewGameResp>, (StatusCode, String)> {
    let mut ready_set = app_state.player_ready_set.write().await;
    let mut role_map = app_state.player_role_map.write().await;
    let mut unassigned = app_state.unassigned_role.write().await;
    let mut history = app_state.history_role_map.write().await;
    let mut counter = app_state.game_counter.write().await;

    // Save current game to history if any players have roles
    if !role_map.is_empty() {
        history.push(role_map.clone());
    }

    let count = query_params.count.max(5).min(10);

    // Reset state
    ready_set.clear();
    role_map.clear();
    unassigned.clear();
    unassigned.extend(Role::role_pool(count));
    *app_state.user_count.write().await = count;

    *counter += 1;

    Ok(Json(NewGameResp {
        des: format!("new game with {} players", count),
    }))
}

pub async fn player_ready(
    State(app_state): State<AppState>,
    Query(query_params): Query<ReadyReq>,
) -> Result<Json<ReadyResp>, (StatusCode, String)> {
    let number = query_params.number;

    let user_count = *app_state.user_count.read().await;
    if user_count == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "game not started, create a new game first".to_string(),
        ));
    }

    if number < 1 || number > user_count as i32 {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("number must be between 1 and {}", user_count),
        ));
    }

    let ready_set = app_state.player_ready_set.read().await;
    if ready_set.contains(&number) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("player {} already ready", number),
        ));
    }
    drop(ready_set);

    // Check if game is already over
    {
        let map = app_state.player_role_map.read().await;
        if map.len() >= user_count {
            return Err((
                StatusCode::BAD_REQUEST,
                "game already over, start a new game".to_string(),
            ));
        }
    }

    // Mark as ready
    {
        let mut set = app_state.player_ready_set.write().await;
        set.insert(number);
    }

    gen_player_role(number, &app_state).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Check if all players are ready — if so, auto-save to history
    {
        let map = app_state.player_role_map.read().await;
        if map.len() == *app_state.user_count.read().await {
            let counter = *app_state.game_counter.read().await;
            let history = app_state.history_role_map.read().await;
            save_game_to_history(&map, counter, &history).await;
        }
    }

    Ok(Json(ReadyResp {
        number,
        ready: true,
    }))
}

async fn gen_player_role(num: i32, app_state: &AppState) -> Result<i32, String> {
    let mut map = app_state.player_role_map.write().await;
    let mut unassigned_role = app_state.unassigned_role.write().await;
    let history = app_state.history_role_map.read().await;

    if unassigned_role.is_empty() {
        return Err("no roles left to assign".to_string());
    }

    // Check if player had a role in the last game for future weighted selection
    let _last_faction: Option<&str> = history
        .last()
        .and_then(|last_map| last_map.get(&num))
        .map(|r| r.faction());

    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..unassigned_role.len());
    let role = unassigned_role.remove(index);
    map.insert(num, role.clone());

    Ok(0)
}

pub async fn poll_player_role(
    State(app_state): State<AppState>,
    Query(query_params): Query<PollRoleReq>,
) -> Result<Json<PollRoleResp>, (StatusCode, String)> {
    let user_count = *app_state.user_count.read().await;
    let map = app_state.player_role_map.read().await;
    let ready_size = map.len();

    if ready_size < user_count {
        return Ok(Json(PollRoleResp {
            ready: false,
            role: String::new(),
            role_des: String::new(),
            skill_des: String::new(),
        }));
    }

    let role = map.get(&query_params.number)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "player not found".to_string()))?;

    let resp = build_poll_role_resp(role, &app_state).await;
    Ok(Json(resp))
}

async fn build_poll_role_resp(role: &Role, state: &AppState) -> PollRoleResp {
    let map = state.player_role_map.read().await;

    let (role_name, skill_des) = match role {
        Role::Merlin => {
            let mut des = "邪恶方玩家有： ".to_string();
            for (num, p_role) in map.iter() {
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
            for (num, p_role) in map.iter() {
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
            for (num, p_role) in map.iter() {
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
            for (num, p_role) in map.iter() {
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
            for (num, p_role) in map.iter() {
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
            for (num, p_role) in map.iter() {
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
        skill_des,
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
) -> Result<Json<CurrentGameResp>, (StatusCode, String)> {
    let user_count = *app_state.user_count.read().await;
    let map = app_state.player_role_map.read().await;
    let ready_set = app_state.player_ready_set.read().await;

    let game_over = map.len() == user_count;

    let mut players: Vec<PlayerInfo> = Vec::new();
    let mut sorted_numbers: Vec<i32> = map.keys().cloned().collect();
    sorted_numbers.sort();

    for num in sorted_numbers {
        let role = &map[&num];
        players.push(PlayerInfo {
            number: num,
            role: role.name_cn().to_string(),
            faction: role.faction().to_string(),
        });
    }

    let all_numbers: Vec<i32> = (1..=user_count as i32).collect();
    let unready: Vec<i32> = all_numbers.into_iter()
        .filter(|n| !ready_set.contains(n))
        .collect();

    Ok(Json(CurrentGameResp {
        game_over,
        player_count: user_count,
        players,
        unready_numbers: unready,
    }))
}

pub async fn admin_history(
    State(_app_state): State<AppState>,
) -> Result<Json<HistoryResp>, (StatusCode, String)> {
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
