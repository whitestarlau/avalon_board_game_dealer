# Avalon Board Game Dealer — Redesign Spec

## Overview
Rewrite the prototype into a usable LAN card-dealing tool with admin panel, single-server deployment, and game history persistence.

## Architecture

```
Axum Server (:3004)
├── /api/*           → REST API for game logic
├── /                → Static files (Vite build output in backend/static/)
└── /admin           → Admin panel (Vue Router route)
```

Build pipeline:
- `frontend/` is the Vue 3 source
- `vite build` outputs to `backend/static/`
- Axum uses `ServeDir` to serve static files
- Single `cargo run` starts everything on `127.0.0.1:3004`

## Game Flow

1. Player opens `http://127.0.0.1:3004/`
2. Selects player number (1-7), clicks "Ready"
3. Frontend calls `GET /api/ready?number=N`
4. All 7 players ready → game is over (dead letters, no gameplay phases)
5. Each player sees their role card
6. Admin (`/admin`) sees all roles for current game + history
7. Host clicks "New Game" → state reset, history appended

### State Machine

```
Lobby → (readying) → GameOver → (new_game) → Lobby
```

- `Lobby`: `/ready` accepted; `/poll_player_role` returns `{"ready": false}`
- `GameOver`: `/ready` rejected; `/poll_player_role` returns the role

## Backend API

### Existing (adjusted)

| Method | Path | Changes |
|--------|------|---------|
| `GET` | `/api/ready?number=N` | Return 400 if already readied; return 400 if game is over |
| `POST` | `/api/poll_player_role?number=N` | Unchanged behavior |
| `GET` | `/api/health_check` | Unchanged |

### New

| Method | Path | Response |
|--------|------|----------|
| `POST` | `/api/new_game` | Reset state, append current game to history file |
| `GET` | `/api/admin/current_game` | `{ game_over: bool, players: [{number, role_name}], unready: [numbers] }` |
| `GET` | `/api/admin/history` | Raw contents of `game_history.json` |

### History File Format (`game_history.json`)

```json
[
  {
    "id": 1,
    "timestamp": "2025-06-05T10:30:00+08:00",
    "players": [
      {"number": 1, "role": "Merlin", "faction": "good"},
      {"number": 2, "role": "Assassin", "faction": "evil"}
    ]
  }
]
```

## Frontend Pages

### `/` (HomeView.vue) — Player interface
- 7 numbered buttons to select player number
- "Ready" button (disabled until number selected)
- After ready: shows "Waiting for other players..." with auto-poll (3s interval)
- When game over: shows role card with name, description, and visible connections

### `/admin` (AdminView.vue) — Admin panel
- **Current Game section**: shows all players + roles when game is over
- **History section**: table of all past games
- **New Game button**: calls `POST /api/new_game`

### `/about` (AboutView.vue) — Unchanged

### Components
- `RoleCard.vue` — Name, faction, description, who they see
- `PlayerSlot.vue` — Number badge + role name (admin view)
- `HistoryTable.vue` — Table of past games

## Fixes Checklist

| Issue | Fix |
|-------|-----|
| Duplicate ready breaks role pool | Check `player_ready_set` in handler; return error if already ready |
| `new_game` not wired | Register `POST /api/new_game` route |
| Pseudo-random weight | Implement with `history_role_map` — if last game was evil, bias toward good |
| `jwtStr` ReferenceError | Remove dead code |
| Hardcoded player number 1 | UI selection + route param |
| Compilation warnings | Clean up unused imports, fix naming |
| LS_of_Arthur camelCase | Rename to `LordOfArthur` or keep `LoyalServant` |
| Zero tests | Add integration tests for game lifecycle |
| Mordred / Minion commented out | Leave as-is (scope) |
| All boilerplate Vue components | Remove unused files |

## Non-Goals
- Authentication / login
- Mordred, Minion roles (not part of 7-player config)
- Game play phases (voting, quests, assassination)
- Multi-machine play (LAN only, same machine)
- Database (file-based persistence only)

## Implementation Order
1. Backend: fix warnings, fix duplicate-ready, wire new_game, add admin APIs, add game history
2. Frontend: rewrite HomeView, build AdminView, build components
3. Integration: Vite build → static directory, test full flow
4. Documentation: update readme
