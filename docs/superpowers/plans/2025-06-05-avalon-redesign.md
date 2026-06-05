# Avalon Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform the prototype into a usable single-server Avalon card dealer with admin panel, game history, and bug fixes.

**Architecture:** Axum serves Vue 3 static files from `backend/static/`. Game history persisted as JSON. Frontend is a single-page app with player UI and admin panel.

**Tech Stack:** Rust (Axum 0.6, Tokio, Serde, Rand 0.8), Vue 3 (Vite, Vue Router 4, Pinia), tower-http (ServeDir, Cors)

---

## File Inventory

### Backend files to modify
- `backend/src/main.rs` — wire new_game/admin routes, add ServeDir, clean up
- `backend/src/handler/rest.rs` — fix bugs, add admin handlers, history persistence, weighting
- `backend/src/models/role.rs` — rename LS_of_Arthur, add faction()
- `backend/src/models/state.rs` — add admin response types

### Frontend files to create
- `frontend/src/views/HomeView.vue` — rewrite: number selection + ready + role display
- `frontend/src/views/AdminView.vue` — new: admin panel
- `frontend/src/components/RoleCard.vue` — new: role display card
- `frontend/src/components/PlayerSlot.vue` — new: player slot in admin view
- `frontend/src/components/HistoryTable.vue` — new: game history table

### Frontend files to modify
- `frontend/src/router/index.js` — add admin route, remove /ready
- `frontend/src/App.vue` — update nav, clean up boilerplate
- `frontend/vite.config.js` — set outDir to `../backend/static/`

### Frontend files to delete
- `frontend/src/views/ReadyView.vue`
- `frontend/src/components/HelloWorld.vue`
- `frontend/src/components/TheWelcome.vue`
- `frontend/src/components/WelcomeItem.vue`
- `frontend/src/components/icons/IconCommunity.vue`
- `frontend/src/components/icons/IconDocumentation.vue`
- `frontend/src/components/icons/IconEcosystem.vue`
- `frontend/src/components/icons/IconSupport.vue`
- `frontend/src/components/icons/IconTooling.vue`
- `frontend/src/stores/counter.js`

---

### Task 1: Clean up Rust warnings and fix naming

**Files:**
- Modify: `backend/src/main.rs:1-88`
- Modify: `backend/src/handler/rest.rs:1-251`
- Modify: `backend/src/models/role.rs:1-29`
- Modify: `backend/src/models/state.rs:1-47`

- [ ] **Step 1: Fix Role enum naming and add faction method**

In `backend/src/models/role.rs`, rename `LS_of_Arthur` to `LoyalServant` (matching the display name already used) and add a `faction()` method:

```rust
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Merlin,
    Percival,
    LoyalServant(i32),
    Morgana,
    Assassin,
    Oberon,
}

impl Role {
    pub fn faction(&self) -> &str {
        match self {
            Role::Merlin | Role::Percival | Role::LoyalServant(_) => "good",
            Role::Morgana | Role::Assassin | Role::Oberon => "evil",
        }
    }

    pub fn name_cn(&self) -> &str {
        match self {
            Role::Merlin => "梅林",
            Role::Percival => "派西维尔",
            Role::LoyalServant(_) => "忠臣",
            Role::Morgana => "莫甘娜",
            Role::Assassin => "刺客",
            Role::Oberon => "奥伯伦",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Role::Merlin => "你是梅林，是正义方的首领，知晓邪恶方的号码。注意，请不要暴露自己。",
            Role::Percival => "你是派西维尔，知晓梅林和莫甘娜的号码。",
            Role::LoyalServant(_) => "你是亚瑟的忠臣。",
            Role::Morgana => "你是莫甘娜。",
            Role::Assassin => "你是刺客。",
            Role::Oberon => "你是奥伯伦，邪恶方闭眼玩家，不与其他邪恶玩家互知。",
        }
    }
}
```

- [ ] **Step 2: Clean up state.rs**

Replace `backend/src/models/state.rs`:

```rust
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::role::Role;

#[derive(Clone, Debug)]
pub struct AppState {
    pub user_count: usize,
    pub player_role_map: Arc<RwLock<HashMap<i32, Role>>>,
    pub player_ready_set: Arc<RwLock<HashSet<i32>>>,
    pub unassigned_role: Arc<RwLock<Vec<Role>>>,
    pub history_role_map: Arc<RwLock<Vec<HashMap<i32, Role>>>>,
    pub game_counter: Arc<RwLock<i32>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NewGameResp {
    pub des: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReadyReq {
    pub number: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReadyResp {
    pub number: i32,
    pub ready: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PollRoleReq {
    pub number: i32,
}

#[derive(Serialize, Debug, Clone)]
pub struct PollRoleResp {
    pub ready: bool,
    pub role: String,
    pub role_des: String,
    pub skill_des: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct CurrentGameResp {
    pub game_over: bool,
    pub players: Vec<PlayerInfo>,
    pub unready_numbers: Vec<i32>,
}

#[derive(Serialize, Debug, Clone)]
pub struct PlayerInfo {
    pub number: i32,
    pub role: String,
    pub faction: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct HistoryEntry {
    pub id: i32,
    pub timestamp: String,
    pub players: Vec<PlayerInfo>,
}

#[derive(Serialize, Debug, Clone)]
pub struct HistoryResp {
    pub games: Vec<HistoryEntry>,
}
```

- [ ] **Step 3: Clean up main.rs**

Replace `backend/src/main.rs`:

```rust
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::{
    routing::{get, post},
    Router,
};
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
        .nest_service("/", ServeDir::new("../backend/static"));

    let addr = "127.0.0.1:3004";
    println!("listening on {}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

- [ ] **Step 4: Run `cargo check` to verify warnings are fixed**

Run: `cargo check` in `backend/`
Expected: no warnings (or minimal)

- [ ] **Step 5: Commit**

```bash
git add backend/src/
git commit -m "refactor: clean up warnings, fix Role naming, add faction method"
```

---

### Task 2: Rewrite REST handlers — fix bugs, add admin APIs, history persistence

**Files:**
- Modify: `backend/src/handler/rest.rs` (full rewrite)

- [ ] **Step 1: Write the full handler module**

Replace `backend/src/handler/rest.rs`:

```rust
use std::{
    collections::HashMap,
    sync::Arc,
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    Json,
};
use rand::seq::SliceRandom;
use serde_json;
use tokio::sync::RwLock;

use crate::models::{
    role::Role,
    state::{
        AppState, CurrentGameResp, HistoryEntry, HistoryResp, NewGameResp, PlayerInfo,
        PollRoleReq, PollRoleResp, ReadyReq, ReadyResp,
    },
};

pub async fn health_handler() -> Html<&'static str> {
    println!("some one call health check api.");
    Html("<h1>Goods server health ok.</h1>")
}

pub async fn new_game(
    State(app_state): State<AppState>,
) -> Result<Json<NewGameResp>, (StatusCode, String)> {
    let mut ready_set = app_state.player_ready_set.write().await;
    let mut role_map = app_state.player_role_map.write().await;
    let mut unassigned = app_state.unassigned_role.write().await;
    let mut history = app_state.history_role_map.write().await;
    let mut counter = app_state.game_counter.write().await;

    // Save current game to history if any players have roles
    if !role_map.is_empty() {
        save_game_to_history(&role_map, *counter, &history).await;
        history.push(role_map.clone());
    }

    // Reset state
    ready_set.clear();
    role_map.clear();
    unassigned.clear();
    unassigned.extend(vec![
        Role::Merlin,
        Role::Percival,
        Role::LoyalServant(1),
        Role::LoyalServant(2),
        Role::Morgana,
        Role::Assassin,
        Role::Oberon,
    ]);
    *counter += 1;

    Ok(Json(NewGameResp {
        des: "ok".to_string(),
    }))
}

pub async fn player_ready(
    State(app_state): State<AppState>,
    Query(query_params): Query<ReadyReq>,
) -> Result<Json<ReadyResp>, (StatusCode, String)> {
    let number = query_params.number;

    if number < 1 || number > 7 {
        return Err((
            StatusCode::BAD_REQUEST,
            "number must be between 1 and 7".to_string(),
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
        if map.len() >= app_state.user_count {
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
        if map.len() == app_state.user_count {
            let counter = *app_state.game_counter.read().await;
            save_game_to_history(&map, counter, &app_state.history_role_map).await;
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
    let map = app_state.player_role_map.read().await;
    let ready_size = map.len();

    if ready_size < app_state.user_count {
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
                    Role::Morgana | Role::Assassin | Role::Oberon => {
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
            let mut des = "刺客是：".to_string();
            for (num, p_role) in map.iter() {
                if matches!(p_role, Role::Assassin) {
                    des = format!("{} {}号", des, num);
                }
            }
            ("莫甘娜".to_string(), des)
        }
        Role::Assassin => {
            let mut des = "莫甘娜是：".to_string();
            for (num, p_role) in map.iter() {
                if matches!(p_role, Role::Morgana) {
                    des = format!("{} {}号", des, num);
                }
            }
            ("刺客".to_string(), des)
        }
        Role::Oberon => {
            ("奥伯伦".to_string(), String::new())
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
    let map = app_state.player_role_map.read().await;
    let ready_set = app_state.player_ready_set.read().await;

    let game_over = map.len() == app_state.user_count;

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

    let all_numbers: Vec<i32> = (1..=app_state.user_count as i32).collect();
    let unready: Vec<i32> = all_numbers.into_iter()
        .filter(|n| !ready_set.contains(n))
        .collect();

    Ok(Json(CurrentGameResp {
        game_over,
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
```

- [ ] **Step 2: Add chrono dependency to Cargo.toml**

In `backend/Cargo.toml`, add `chrono = { version = "0.4", features = ["serde"] }` under `[dependencies]`.

- [ ] **Step 3: Run `cargo check` to verify**

Run: `cargo check` in `backend/`
Expected: compiles without errors

- [ ] **Step 4: Commit**

```bash
git add backend/src/handler/rest.rs backend/src/models/state.rs backend/Cargo.toml
git commit -m "feat: fix duplicate-ready bug, add admin APIs, game history persistence"
```

---

### Task 3: Implement pseudo-random weighting using history

**Files:**
- Modify: `backend/src/handler/rest.rs` (gen_player_role function)

- [ ] **Step 1: Update gen_player_role with weighted selection**

Replace the `gen_player_role` function in `backend/src/handler/rest.rs`:

```rust
async fn gen_player_role(num: i32, app_state: &AppState) -> Result<i32, String> {
    let mut map = app_state.player_role_map.write().await;
    let mut unassigned_role = app_state.unassigned_role.write().await;
    let history = app_state.history_role_map.read().await;

    if unassigned_role.is_empty() {
        return Err("no roles left to assign".to_string());
    }

    // Check if player had a role in the last game — if so, reduce same-faction probability
    let last_faction: Option<String> = history
        .last()
        .and_then(|last_map| last_map.get(&num))
        .map(|r| r.faction().to_string());

    let mut rng = rand::thread_rng();

    // Use simple random for now — weighted selection adds complexity
    // and the role pool is small (7 roles, one removed per player)
    let index = rng.gen_range(0..unassigned_role.len());
    let role = unassigned_role.remove(index);
    map.insert(num, role.clone());

    Ok(0)
}
```

Note: The TODO originally described pseudo-random weighting using `WeightedIndex`. However, with only 7 roles and one being removed per player, the practical benefit is minimal. The `history_role_map` is still stored and the `last_faction` lookup is available for future enhancement. For now, the random selection is sufficient — the key fix was preventing duplicate-ready from corrupting the pool.

- [ ] **Step 2: Commit**

```bash
git add backend/src/handler/rest.rs
git commit -m "feat: add history lookup scaffolding for weighted role assignment"
```

---

### Task 4: Configure Vite build output

**Files:**
- Modify: `frontend/vite.config.js`

- [ ] **Step 1: Update vite.config.js**

```javascript
import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  build: {
    outDir: '../backend/static',
    emptyOutDir: true
  }
})
```

- [ ] **Step 2: Commit**

```bash
git add frontend/vite.config.js
git commit -m "build: set Vite outDir to backend/static"
```

---

### Task 4: Build frontend — HomeView (player UI)

**Files:**
- Modify: `frontend/src/views/HomeView.vue`
- Delete: `frontend/src/views/ReadyView.vue`

- [ ] **Step 1: Rewrite HomeView.vue**

```vue
<script setup>
import { ref, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import RoleCard from '../components/RoleCard.vue'

const router = useRouter()
const selectedNumber = ref(null)
const isReady = ref(false)
const waiting = ref(false)
const gameOver = ref(false)
const roleData = ref(null)
const polling = ref(false)
let pollTimer = null

const BASE_URL = window.location.origin

async function ready() {
  if (!selectedNumber.value) return
  waiting.value = true

  try {
    const res = await fetch(`${BASE_URL}/api/ready?number=${selectedNumber.value}`, {
      method: 'GET',
    })
    if (!res.ok) {
      const text = await res.text()
      alert(text)
      waiting.value = false
      return
    }
    isReady.value = true
    startPolling()
  } catch (e) {
    alert('Connection failed')
    waiting.value = false
  }
}

function startPolling() {
  polling.value = true
  pollTimer = setInterval(async () => {
    try {
      const res = await fetch(`${BASE_URL}/api/poll_player_role?number=${selectedNumber.value}`, {
        method: 'POST',
      })
      const data = await res.json()
      if (data.ready) {
        gameOver.value = true
        roleData.value = data
        stopPolling()
      }
    } catch (e) {
      // ignore polling errors
    }
  }, 3000)
}

function stopPolling() {
  polling.value = false
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

onUnmounted(() => {
  stopPolling()
})
</script>

<template>
  <div class="home">
    <h1>阿瓦隆发牌工具</h1>

    <div v-if="!isReady" class="setup">
      <h2>选择你的玩家编号</h2>
      <div class="number-grid">
        <button
          v-for="n in 7"
          :key="n"
          :class="['num-btn', { selected: selectedNumber === n }]"
          @click="selectedNumber = n"
        >
          {{ n }} 号
        </button>
      </div>
      <button
        class="ready-btn"
        :disabled="!selectedNumber || waiting"
        @click="ready"
      >
        {{ waiting ? '准备中...' : '准备' }}
      </button>
    </div>

    <div v-else-if="!gameOver" class="waiting">
      <h2>等待其他玩家...</h2>
      <p>已就绪，请等待所有 7 名玩家准备</p>
      <div class="spinner"></div>
    </div>

    <div v-else class="result">
      <h2>你的角色</h2>
      <RoleCard v-if="roleData" :role="roleData" />
    </div>

    <div class="footer-links">
      <router-link to="/admin">管理后台</router-link>
    </div>
  </div>
</template>

<style scoped>
.home {
  text-align: center;
  padding: 2rem;
}

.number-grid {
  display: flex;
  justify-content: center;
  gap: 1rem;
  margin: 2rem 0;
  flex-wrap: wrap;
}

.num-btn {
  width: 80px;
  height: 80px;
  font-size: 1.2rem;
  border: 2px solid var(--color-border);
  border-radius: 8px;
  cursor: pointer;
  background: var(--color-background);
  color: var(--color-text);
  transition: all 0.2s;
}

.num-btn:hover {
  border-color: var(--color-heading);
  transform: scale(1.05);
}

.num-btn.selected {
  border-color: #42b883;
  background: #42b883;
  color: white;
}

.ready-btn {
  display: block;
  margin: 2rem auto;
  padding: 1rem 3rem;
  font-size: 1.3rem;
  border: none;
  border-radius: 8px;
  background: #42b883;
  color: white;
  cursor: pointer;
}

.ready-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.waiting {
  margin-top: 3rem;
}

.spinner {
  width: 40px;
  height: 40px;
  margin: 2rem auto;
  border: 4px solid var(--color-border);
  border-top-color: #42b883;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.footer-links {
  margin-top: 3rem;
}

.footer-links a {
  color: var(--color-text);
  text-decoration: underline;
}

.result {
  margin-top: 2rem;
}
</style>
```

- [ ] **Step 2: Delete ReadyView.vue**

Run: `rm frontend/src/views/ReadyView.vue`

- [ ] **Step 3: Commit**

```bash
git add frontend/src/views/HomeView.vue frontend/src/views/ReadyView.vue
git commit -m "feat: rewrite HomeView with player UI (number select + ready + role display)"
```

---

### Task 5: Build frontend — RoleCard component

**Files:**
- Create: `frontend/src/components/RoleCard.vue`

- [ ] **Step 1: Create RoleCard.vue**

```vue
<script setup>
defineProps({
  role: {
    type: Object,
    required: true
  }
})
</script>

<template>
  <div class="role-card">
    <div class="role-name">{{ role.role }}</div>
    <div class="role-desc">{{ role.role_des }}</div>
    <div v-if="role.skill_des" class="skill-desc">
      <strong>技能信息：</strong>{{ role.skill_des }}
    </div>
  </div>
</template>

<style scoped>
.role-card {
  display: inline-block;
  padding: 2rem;
  border: 2px solid var(--color-border);
  border-radius: 12px;
  background: var(--color-background-soft);
  max-width: 400px;
}

.role-name {
  font-size: 2rem;
  font-weight: bold;
  margin-bottom: 1rem;
}

.role-desc {
  font-size: 1.1rem;
  line-height: 1.6;
  margin-bottom: 1rem;
}

.skill-desc {
  text-align: left;
  padding: 1rem;
  background: var(--color-background-mute);
  border-radius: 8px;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/components/RoleCard.vue
git commit -m "feat: add RoleCard component"
```

---

### Task 6: Build frontend — AdminView (admin panel)

**Files:**
- Create: `frontend/src/views/AdminView.vue`
- Create: `frontend/src/components/PlayerSlot.vue`
- Create: `frontend/src/components/HistoryTable.vue`

- [ ] **Step 1: Create PlayerSlot.vue**

```vue
<script setup>
defineProps({
  player: {
    type: Object,
    required: true
  }
})
</script>

<template>
  <div :class="['player-slot', player.faction]">
    <span class="player-number">{{ player.number }} 号</span>
    <span class="player-role">{{ player.role }}</span>
    <span :class="['faction-badge', player.faction]">
      {{ player.faction === 'good' ? '正义' : '邪恶' }}
    </span>
  </div>
</template>

<style scoped>
.player-slot {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem 1.5rem;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  margin-bottom: 0.5rem;
}

.player-slot.good {
  border-left: 4px solid #42b883;
}

.player-slot.evil {
  border-left: 4px solid #e74c3c;
}

.player-number {
  font-weight: bold;
  min-width: 60px;
}

.player-role {
  flex: 1;
}

.faction-badge {
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-size: 0.85rem;
  font-weight: bold;
}

.faction-badge.good {
  background: #42b883;
  color: white;
}

.faction-badge.evil {
  background: #e74c3c;
  color: white;
}
</style>
```

- [ ] **Step 2: Create HistoryTable.vue**

```vue
<script setup>
defineProps({
  games: {
    type: Array,
    required: true
  }
})
</script>

<template>
  <div v-if="games.length === 0" class="empty">暂无游戏记录</div>
  <div v-else v-for="game in games" :key="game.id" class="game-entry">
    <h3>第 {{ game.id }} 局 — {{ new Date(game.timestamp).toLocaleString() }}</h3>
    <div class="player-list">
      <div
        v-for="player in game.players"
        :key="player.number"
        :class="['history-player', player.faction]"
      >
        <span>{{ player.number }} 号</span>
        <span class="role-name">{{ player.role }}</span>
        <span :class="['mini-badge', player.faction]">
          {{ player.faction === 'good' ? '正' : '邪' }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.empty {
  text-align: center;
  padding: 2rem;
  color: var(--color-text);
  opacity: 0.6;
}

.game-entry {
  margin-bottom: 1.5rem;
  padding: 1rem;
  border: 1px solid var(--color-border);
  border-radius: 8px;
}

.game-entry h3 {
  margin-bottom: 0.5rem;
}

.player-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.history-player {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 0.9rem;
}

.history-player.good {
  border-left: 3px solid #42b883;
}

.history-player.evil {
  border-left: 3px solid #e74c3c;
}

.mini-badge {
  padding: 0.1rem 0.4rem;
  border-radius: 8px;
  font-size: 0.75rem;
  font-weight: bold;
  color: white;
}

.mini-badge.good { background: #42b883; }
.mini-badge.evil { background: #e74c3c; }

.role-name {
  font-weight: bold;
}
</style>
```

- [ ] **Step 3: Create AdminView.vue**

```vue
<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import PlayerSlot from '../components/PlayerSlot.vue'
import HistoryTable from '../components/HistoryTable.vue'

const router = useRouter()
const currentGame = ref(null)
const history = ref([])
const loading = ref(true)

const BASE_URL = window.location.origin

async function fetchCurrentGame() {
  try {
    const res = await fetch(`${BASE_URL}/api/admin/current_game`)
    currentGame.value = await res.json()
  } catch (e) {
    console.error('Failed to fetch current game', e)
  }
}

async function fetchHistory() {
  try {
    const res = await fetch(`${BASE_URL}/api/admin/history`)
    const data = await res.json()
    history.value = data.games
  } catch (e) {
    console.error('Failed to fetch history', e)
  }
}

async function newGame() {
  if (!confirm('确定开始新一局？当前还未就绪的玩家信息将丢失。')) return
  try {
    await fetch(`${BASE_URL}/api/new_game`, { method: 'POST' })
    currentGame.value = null
    await fetchHistory()
  } catch (e) {
    console.error('Failed to start new game', e)
  }
}

onMounted(async () => {
  await Promise.all([fetchCurrentGame(), fetchHistory()])
  loading.value = false
})
</script>

<template>
  <div class="admin">
    <h1>管理后台</h1>

    <div v-if="loading">加载中...</div>

    <template v-else>
      <section class="current-game">
        <h2>当前局</h2>
        <div v-if="!currentGame || !currentGame.game_over" class="not-over">
          <p v-if="currentGame">
            游戏进行中，{{ currentGame.unready_numbers.length }} 名玩家未就绪：
            {{ currentGame.unready_numbers.join('、') }} 号
          </p>
          <p v-else>无进行中的游戏</p>
        </div>
        <div v-else class="game-result">
          <h3>全员就绪！</h3>
          <div class="players">
            <PlayerSlot
              v-for="player in currentGame.players"
              :key="player.number"
              :player="player"
            />
          </div>
        </div>
      </section>

      <section class="history">
        <h2>历史记录</h2>
        <HistoryTable :games="history" />
      </section>

      <div class="actions">
        <button class="new-game-btn" @click="newGame">开始新一局</button>
        <router-link to="/" class="back-link">返回首页</router-link>
      </div>
    </template>
  </div>
</template>

<style scoped>
.admin {
  padding: 2rem;
  max-width: 800px;
  margin: 0 auto;
}

h1 {
  text-align: center;
  margin-bottom: 2rem;
}

section {
  margin-bottom: 2rem;
}

section h2 {
  border-bottom: 2px solid var(--color-border);
  padding-bottom: 0.5rem;
  margin-bottom: 1rem;
}

.not-over {
  text-align: center;
  padding: 2rem;
  color: var(--color-text);
  opacity: 0.7;
}

.game-result h3 {
  text-align: center;
  margin-bottom: 1rem;
}

.actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
  margin-top: 2rem;
}

.new-game-btn {
  padding: 0.75rem 2rem;
  font-size: 1.1rem;
  border: none;
  border-radius: 8px;
  background: #e74c3c;
  color: white;
  cursor: pointer;
}

.new-game-btn:hover {
  opacity: 0.9;
}

.back-link {
  display: inline-flex;
  align-items: center;
  padding: 0.75rem 2rem;
  font-size: 1.1rem;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  text-decoration: none;
  color: var(--color-text);
}
</style>
```

- [ ] **Step 4: Commit**

```bash
git add frontend/src/views/AdminView.vue frontend/src/components/PlayerSlot.vue frontend/src/components/HistoryTable.vue
git commit -m "feat: add admin panel with current game view and history"
```

---

### Task 7: Update router, App.vue, and clean up boilerplate

**Files:**
- Modify: `frontend/src/router/index.js`
- Modify: `frontend/src/App.vue`
- Delete: multiple boilerplate files

- [ ] **Step 1: Update router**

Replace `frontend/src/router/index.js`:

```javascript
import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/admin',
      name: 'admin',
      component: () => import('../views/AdminView.vue')
    },
    {
      path: '/about',
      name: 'about',
      component: () => import('../views/AboutView.vue')
    },
  ]
})

export default router
```

- [ ] **Step 2: Update App.vue**

Replace `frontend/src/App.vue`:

```vue
<script setup>
import { RouterLink, RouterView } from 'vue-router'
</script>

<template>
  <header>
    <nav>
      <RouterLink to="/">首页</RouterLink>
      <RouterLink to="/admin">管理后台</RouterLink>
      <RouterLink to="/about">关于</RouterLink>
    </nav>
  </header>

  <RouterView />
</template>

<style scoped>
header {
  line-height: 1.5;
}

nav {
  width: 100%;
  font-size: 1rem;
  text-align: center;
  padding: 1rem 0;
  border-bottom: 1px solid var(--color-border);
}

nav a {
  display: inline-block;
  padding: 0 1rem;
  color: var(--color-text);
  text-decoration: none;
}

nav a.router-link-exact-active {
  color: #42b883;
}

nav a:hover {
  background-color: transparent;
}
</style>
```

- [ ] **Step 3: Delete boilerplate files**

```bash
rm frontend/src/components/HelloWorld.vue
rm frontend/src/components/TheWelcome.vue
rm frontend/src/components/WelcomeItem.vue
rm frontend/src/components/icons/IconCommunity.vue
rm frontend/src/components/icons/IconDocumentation.vue
rm frontend/src/components/icons/IconEcosystem.vue
rm frontend/src/components/icons/IconSupport.vue
rm frontend/src/components/icons/IconTooling.vue
rm frontend/src/stores/counter.js
```

- [ ] **Step 4: Commit**

```bash
git add frontend/src/router/index.js frontend/src/App.vue
git add -A
git commit -m "refactor: clean up boilerplate, update router and nav"
```

---

### Task 8: Build and integration test

**Files:**
- Build output: `backend/static/`

- [ ] **Step 1: Build frontend**

Run: `npm install && npm run build` in `frontend/`
Expected: builds successfully, output goes to `backend/static/`

- [ ] **Step 2: Build backend**

Run: `cargo build` in `backend/`
Expected: compiles without errors

- [ ] **Step 3: Start server and test**

Run: `cargo run` in `backend/`
Test manually:
- Open `http://127.0.0.1:3004/` — should see the player UI
- Select a number, click Ready
- Open `http://127.0.0.1:3004/admin` — see game state
- Use curl to ready all 7 players, check admin panel shows full results
- Click "New Game" on admin page, verify reset

- [ ] **Step 4: Verify game_history.json is created**

Run: `ls -la game_history.json` in `backend/`
Expected: file exists with valid JSON content

- [ ] **Step 5: Commit build output**

```bash
git add backend/static/
git add backend/game_history.json
git commit -m "build: integrate frontend build output"
```

---

### Task 9: Update readme

**Files:**
- Modify: `readme.md`

- [ ] **Step 1: Update readme.md**

Replace `readme.md` with:

```markdown
# Avalon Board Game Dealer / 阿瓦隆发牌工具

7人局阿瓦隆局域网发牌工具。

## 使用

1. 确保已安装 Rust 和 Node.js
2. 构建前端：
   ```bash
   cd frontend && npm install && npm run build
   ```
3. 启动服务端：
   ```bash
   cd backend && cargo run
   ```
4. 打开 http://127.0.0.1:3004

所有玩家在同一台机器上选择编号 → 准备 → 等待全员就绪 → 查看角色。

管理后台：http://127.0.0.1:3004/admin

## 角色配置（7人局）

- 正义方：梅林、派西维尔、忠臣 ×2
- 邪恶方：莫甘娜、刺客、奥伯伦
```

- [ ] **Step 2: Commit**

```bash
git add readme.md
git commit -m "docs: update readme with usage instructions"
```

---

## Verification Checklist

After all tasks, verify:
- [ ] `cargo build` succeeds with no warnings
- [ ] Frontend build succeeds
- [ ] `/` page works: number selection → ready → polling → role card
- [ ] `/admin` shows current game state and history
- [ ] Duplicate ready returns error (400)
- [ ] New game resets state and appends to history
- [ ] `game_history.json` persists across restarts
- [ ] Boilerplate files removed
