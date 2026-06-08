use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, watch};

use super::role::Role;

#[derive(Clone, Debug)]
pub struct AppState {
    pub user_count: Arc<RwLock<usize>>,
    pub player_role_map: Arc<RwLock<HashMap<i32, Role>>>,
    pub player_ready_set: Arc<RwLock<HashSet<i32>>>,
    pub unassigned_role: Arc<RwLock<Vec<Role>>>,
    #[allow(dead_code)]
    pub history_role_map: Arc<RwLock<Vec<HashMap<i32, Role>>>>,
    #[allow(dead_code)]
    pub game_counter: Arc<RwLock<i32>>,
    pub show_skill_info: Arc<RwLock<bool>>,
    pub game_complete_tx: Arc<watch::Sender<()>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NewGameReq {
    pub count: usize,
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
    pub player_count: usize,
    pub players: Vec<PlayerInfo>,
    pub unready_numbers: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerInfo {
    pub number: i32,
    pub role: String,
    pub faction: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoryEntry {
    pub id: i32,
    pub timestamp: String,
    pub players: Vec<PlayerInfo>,
}

#[derive(Serialize, Debug, Clone)]
pub struct HistoryResp {
    pub games: Vec<HistoryEntry>,
}

#[derive(Serialize, Debug, Clone)]
pub struct SkillInfoStatusResp {
    pub show_skill_info: bool,
}
