# 技能信息展示开关 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an admin toggle to control whether skill information (e.g., Merlin seeing evil player numbers) is shown to players.

**Architecture:** Add `show_skill_info` field to backend `AppState`, two new admin APIs for getting/toggling the state, modify `build_poll_role_resp` to conditionally clear `skill_des`, and add a toggle button in the frontend `AdminView`.

**Tech Stack:** Rust/Axum (backend), Vue 3 (frontend)

---

### Task 1: Add `show_skill_info` to AppState and SkillInfoStatusResp

**Files:**
- Modify: `backend/src/models/state.rs:12-21` (AppState struct)
- Modify: `backend/src/models/state.rs:79-82` (add new response struct)

- [ ] **Step 1: Add `show_skill_info` field to AppState**

In `backend/src/models/state.rs`, add after line 20 (`pub game_counter`):

```rust
    pub show_skill_info: Arc<RwLock<bool>>,
```

- [ ] **Step 2: Add `SkillInfoStatusResp` struct**

In `backend/src/models/state.rs`, add after the `HistoryResp` struct (after line 82):

```rust
#[derive(Serialize, Debug, Clone)]
pub struct SkillInfoStatusResp {
    pub show_skill_info: bool,
}
```

- [ ] **Step 3: Commit**

```bash
git add backend/src/models/state.rs
git commit -m "feat: add show_skill_info to AppState and SkillInfoStatusResp"
```

---

### Task 2: Initialize `show_skill_info` in build_app and wire new API routes

**Files:**
- Modify: `backend/src/lib.rs:81-97` (build_app function)
- Modify: `backend/src/lib.rs:22-28` (imports)
- Modify: `backend/src/lib.rs:99-107` (api_routes)

- [ ] **Step 1: Add `show_skill_info` initialization in `build_app`**

In `backend/src/lib.rs`, add after line 88 (`let game_counter`):

```rust
    let show_skill_info: Arc<RwLock<bool>> = Arc::new(RwLock::new(true));
```

Add the field to the `AppState` struct init (after `game_counter,` on line 96):

```rust
        show_skill_info,
```

- [ ] **Step 2: Add new handler imports**

In `backend/src/lib.rs`, change the import block (lines 22-28) to include the new handlers:

```rust
use crate::{
    handler::rest::{
        admin_current_game, admin_history, admin_skill_info_status,
        admin_toggle_skill_info, health_handler, new_game, player_ready,
        poll_player_role,
    },
    models::{role::Role, state::AppState},
};
```

- [ ] **Step 3: Add new API routes**

In `backend/src/lib.rs`, add after the `admin/history` route (after line 105):

```rust
        .route("/admin/skill_info_status", get(admin_skill_info_status))
        .route("/admin/toggle_skill_info", post(admin_toggle_skill_info))
```

- [ ] **Step 4: Commit**

```bash
git add backend/src/lib.rs
git commit -m "feat: wire show_skill_info state and new admin API routes"
```

---

### Task 3: Implement new admin handlers and modify build_poll_role_resp

**Files:**
- Modify: `backend/src/handler/rest.rs:11-17` (imports)
- Modify: `backend/src/handler/rest.rs:24-53` (new_game — reset show_skill_info)
- Modify: `backend/src/handler/rest.rs:168-258` (build_poll_role_resp — conditionally clear skill_des)
- Modify: `backend/src/handler/rest.rs:337-351` (add new handlers at end)

- [ ] **Step 1: Add `SkillInfoStatusResp` to imports**

In `backend/src/handler/rest.rs`, change the import block (lines 11-17) to:

```rust
use crate::models::{
    role::Role,
    state::{
        AppState, CurrentGameResp, HistoryEntry, HistoryResp, NewGameReq, NewGameResp,
        PlayerInfo, PollRoleReq, PollRoleResp, ReadyReq, ReadyResp, SkillInfoStatusResp,
    },
};
```

- [ ] **Step 2: Reset `show_skill_info` in `new_game` handler**

In `backend/src/handler/rest.rs`, add after line 46 (`*app_state.user_count.write().await = count;`):

```rust
    *app_state.show_skill_info.write().await = true;
```

- [ ] **Step 3: Modify `build_poll_role_resp` to respect `show_skill_info`**

In `backend/src/handler/rest.rs`, replace the `build_poll_role_resp` function (lines 168-258) with:

```rust
async fn build_poll_role_resp(role: &Role, state: &AppState) -> PollRoleResp {
    let map = state.player_role_map.read().await;
    let show_skill_info = *state.show_skill_info.read().await;

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
        skill_des: if show_skill_info { skill_des } else { String::new() },
    }
}
```

- [ ] **Step 4: Add `admin_skill_info_status` and `admin_toggle_skill_info` handlers**

In `backend/src/handler/rest.rs`, add after the `admin_history` function (after line 351):

```rust
pub async fn admin_skill_info_status(
    State(app_state): State<AppState>,
) -> Result<Json<SkillInfoStatusResp>, (StatusCode, String)> {
    let show = *app_state.show_skill_info.read().await;
    Ok(Json(SkillInfoStatusResp {
        show_skill_info: show,
    }))
}

pub async fn admin_toggle_skill_info(
    State(app_state): State<AppState>,
) -> Result<Json<SkillInfoStatusResp>, (StatusCode, String)> {
    let mut show = app_state.show_skill_info.write().await;
    *show = !*show;
    let new_val = *show;
    drop(show);
    Ok(Json(SkillInfoStatusResp {
        show_skill_info: new_val,
    }))
}
```

- [ ] **Step 5: Build and verify**

Run: `cd backend && cargo build`
Expected: compilation succeeds with no errors

- [ ] **Step 6: Commit**

```bash
git add backend/src/handler/rest.rs
git commit -m "feat: implement skill info toggle handlers and conditional skill_des"
```

---

### Task 4: Add toggle UI in AdminView frontend

**Files:**
- Modify: `frontend/src/views/AdminView.vue:1-53` (script setup)
- Modify: `frontend/src/views/AdminView.vue:79-105` (current-game section template)
- Modify: `frontend/src/views/AdminView.vue:120-275` (styles)

- [ ] **Step 1: Add `showSkillInfo` ref and fetch/toggle functions**

In `frontend/src/views/AdminView.vue`, add after line 10 (`const setupPlayerCount = ref(7)`):

```javascript
const showSkillInfo = ref(true)
```

Add after the `newGame` function (after line 48):

```javascript
async function fetchSkillInfoStatus() {
  try {
    const res = await fetch(`${BASE_URL}/api/admin/skill_info_status`)
    const data = await res.json()
    showSkillInfo.value = data.show_skill_info
  } catch (e) {
    console.error('Failed to fetch skill info status', e)
  }
}

async function toggleSkillInfo() {
  try {
    const res = await fetch(`${BASE_URL}/api/admin/toggle_skill_info`, {
      method: 'POST',
    })
    const data = await res.json()
    showSkillInfo.value = data.show_skill_info
  } catch (e) {
    console.error('Failed to toggle skill info', e)
  }
}
```

- [ ] **Step 2: Add `fetchSkillInfoStatus` to onMounted**

In `frontend/src/views/AdminView.vue`, change the `onMounted` (lines 50-53) to:

```javascript
onMounted(async () => {
  await Promise.all([fetchCurrentGame(), fetchHistory(), fetchSkillInfoStatus()])
  loading.value = false
})
```

- [ ] **Step 3: Add toggle button to the current-game section template**

In `frontend/src/views/AdminView.vue`, add after the section-header div (after line 85, before the `<div v-if="!currentGame.game_over">`):

```html
        <div class="skill-info-toggle">
          <span>技能信息：</span>
          <button
            :class="['toggle-btn', { active: showSkillInfo }]"
            @click="toggleSkillInfo"
          >
            {{ showSkillInfo ? '展示' : '隐藏' }}
          </button>
        </div>
```

- [ ] **Step 4: Add styles for the toggle**

In `frontend/src/views/AdminView.vue`, add before the closing `</style>` tag:

```css
.skill-info-toggle {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 1rem;
  padding: 0.5rem 0;
}

.skill-info-toggle span {
  font-size: 0.95rem;
  color: var(--color-text);
  opacity: 0.8;
}

.toggle-btn.active {
  border-color: #42b883;
  color: #42b883;
}
```

- [ ] **Step 5: Commit**

```bash
git add frontend/src/views/AdminView.vue
git commit -m "feat: add skill info toggle button in admin view"
```

---

### Task 5: Build frontend and verify end-to-end

**Files:** None (verification only)

- [ ] **Step 1: Build frontend**

Run: `cd frontend && npm run build`
Expected: build succeeds

- [ ] **Step 2: Build backend**

Run: `cd backend && cargo build`
Expected: compilation succeeds

- [ ] **Step 3: Final commit if any adjustments needed**

If any fixes were required during verification, commit them.
