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
    pub player_role_map: Arc<RwLock<HashMap<i32,Role>>>,
    pub player_ready_set: Arc<RwLock<HashSet<i32>>>,
    pub unassigned_role: Arc<RwLock<Vec<Role>>>,
    //记录一下之前每一轮用户的角色信息，以便进行伪随机
    pub history_role_map: Arc<RwLock<Vec<HashMap<i32,Role>>>>,
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
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PollRoleReq {
    pub number: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PollRoleResp {
    pub role: String,
    pub role_des: String,
    pub skill_des: String,
}
