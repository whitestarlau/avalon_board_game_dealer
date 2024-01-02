use core::num;
use std::{
    collections::{hash_set, HashMap, HashSet},
    f32::consts::E,
    sync::Arc,
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    Json,
};
use rand::{
    distributions::{Distribution, WeightedIndex},
    seq::SliceRandom,
    Rng,
};
use serde_json::to_string;
use tokio::sync::{RwLock, RwLockReadGuard};
use tower_http::follow_redirect::policy::PolicyExt;

use crate::models::{
    role::Role,
    state::{AppState, NewGameResp, PollRoleReq, PollRoleResp, ReadyReq, ReadyResp},
};

pub async fn health_handler() -> Html<&'static str> {
    println!("some one call health check api.");
    Html("<h1>Goods server health ok.</h1>")
}

pub async fn new_game(
    State(app_state): State<AppState>,
) -> Result<axum::Json<NewGameResp>, (axum::http::StatusCode, String)> {
    let mut set = app_state.player_ready_set.write().await;
    let mut map = app_state.player_role_map.write().await;
    let mut unassigned_role = app_state.unassigned_role.write().await;

    if app_state.user_count == 7 {
        unassigned_role.clear();
        unassigned_role.push(Role::Merlin);
        unassigned_role.push(Role::Percival);
        unassigned_role.push(Role::LS_of_Arthur(1));
        unassigned_role.push(Role::LS_of_Arthur(2));
        unassigned_role.push(Role::Morgana);
        unassigned_role.push(Role::Assassin);
        unassigned_role.push(Role::Oberon);

        map.clear()
    } else {
        //暂不支持
    }

    return Ok(axum::Json(NewGameResp {
        des: "".to_string(),
    }));
}

/**
 * 在Ready的一刻就计算好当前玩家的角色，但是我们要等到所有玩家都准备好之后才会放出对应的结果。
 */
pub async fn player_ready(
    State(app_state): State<AppState>,
    Query(query_params): Query<ReadyReq>,
) -> Result<axum::Json<ReadyResp>, (axum::http::StatusCode, String)> {
    let number = query_params.number;
    let mut map = app_state.player_role_map.write().await;

    {
        let mut set = app_state.player_ready_set.write().await;
        if set.contains(&number) {
            //重新分配
        } else {
            set.insert(query_params.number);
        }
    }

    let mut map = app_state.player_role_map.write().await;
    let values = map.values();
    for value in values {
        match value {
            Role::Merlin => todo!(),
            Role::Percival => todo!(),
            Role::LS_of_Arthur(_) => todo!(),
            Role::Morgana => todo!(),
            Role::Assassin => todo!(),
            Role::Oberon => todo!(),
        }
    }
    // map.insert(number, v);

    let resp = ReadyResp {
        number: query_params.number,
    };

    return Ok(axum::Json(resp));
    // return Err((StatusCode::INTERNAL_SERVER_ERROR, "aaa".to_string()));
}

async fn gen_player_role(num: i32, app_state: &AppState) {
    let mut map = app_state.player_role_map.write().await;
    let mut unassigned_role = app_state.unassigned_role.write().await;

    let mut rng = rand::thread_rng();

    //TODO 使用伪随机权重，上局游戏打过的角色或者阵营，这一把将降低概率。如果连续两把随机到同一个阵营，将大幅度降低其概率。
    // let weights = [2, 10, 1]; // 这里的权重值越大，对应的元素被选中的概率就越高
    // let dist = WeightedIndex::new(&weights).unwrap();
    // let random_index = dist.sample(&mut rng);
    // let random_enum = unassigned_role.get(random_index);

    let len = unassigned_role.len();
    let index = rng.gen_range(0..len);

    let random_role_opt = unassigned_role.get(index);

    if let Some(r) = random_role_opt {
        map.insert(num, r.clone());
        unassigned_role.remove(index);
    }
}

pub async fn poll_player_role(
    State(app_state): State<AppState>,
    Query(query_params): Query<PollRoleReq>,
) -> Result<axum::Json<PollRoleResp>, (axum::http::StatusCode, String)> {
    let map: RwLockReadGuard<'_, HashMap<i32, Role>> = app_state.player_role_map.read().await;
    let ready_size = map.values().len();
    if ready_size < app_state.user_count {
        //还有玩家没准备，不返回结果。
        let error = (
            StatusCode::INTERNAL_SERVER_ERROR,
            "some body dont ready".to_string(),
        );
        return Err(error);
    } else {
        let roleOpt = map.get(&query_params.number);
        if let Some(role) = roleOpt {
            //
            let resp = build_poll_role_resp(role, &app_state).await;
            return Ok(axum::Json(resp));
        } else {
            //不应该出现这种情况
            let error = (
                StatusCode::INTERNAL_SERVER_ERROR,
                "your number is not ready".to_string(),
            );
            return Err(error);
        }
    }
}

/**
 * 构建轮训的返回
 */
async fn build_poll_role_resp(role: &Role, state: &AppState) -> PollRoleResp {
    let map = state.player_role_map.read().await;

    let resp = match role {
        Role::Merlin => {
            //找到所有的邪恶方玩家，提供给ta
            let mut h_roles: Vec<i32> = Vec::new();
            let mut skill_des = "邪恶方玩家有： ".to_string();
            for (num, p_role) in map.clone().into_iter() {
                match p_role {
                    Role::Morgana | Role::Assassin | Role::Oberon => {
                        skill_des = format!("{} {}号", skill_des, num);
                        h_roles.push(num)
                    }
                    _ => {}
                }
            }
            PollRoleResp {
                role: "Merlin".to_string(),
                role_des: "你是梅林，是正义方的首领，知晓邪恶方的号码。注意，请不要暴露自己。"
                    .to_string(),
                skill_des: skill_des,
            }
        }
        Role::Percival => {
            //找到梅林和莫甘娜提供给ta
            let mut h_roles: Vec<i32> = Vec::new();
            let mut skill_des = "梅林和莫甘娜是：".to_string();
            for (num, p_role) in map.clone().into_iter() {
                match p_role {
                    Role::Morgana | Role::Merlin => {
                        skill_des = format!("{} {}号", skill_des, num);
                        h_roles.push(num)
                    }
                    _ => {}
                }
            }
            PollRoleResp {
                role: "Percival".to_string(),
                role_des: "你是派。".to_string(),
                skill_des: skill_des,
            }
        }
        Role::LS_of_Arthur(_) => PollRoleResp {
            role: "LS_of_Arthur".to_string(),
            role_des: "你是亚瑟的忠臣。".to_string(),
            skill_des: "".to_string(),
        },
        Role::Morgana => {
            let mut skill_des = "刺客是：".to_string();
            for (num, p_role) in map.clone().into_iter() {
                match p_role {
                    Role::Assassin => {
                        skill_des = format!("{} {}号", skill_des, num);
                    }
                    _ => {}
                }
            }
            PollRoleResp {
                role: "Morgana".to_string(),
                role_des: "你是莫甘娜。".to_string(),
                skill_des: skill_des.to_string(),
            }
        }
        Role::Assassin => {
            let mut skill_des = "莫甘娜是：".to_string();
            for (num, p_role) in map.clone().into_iter() {
                match p_role {
                    Role::Morgana => {
                        skill_des = format!("{} {}号", skill_des, num);
                    }
                    _ => {}
                }
            }
            PollRoleResp {
                role: "Assassin".to_string(),
                role_des: "你是刺客。".to_string(),
                skill_des: skill_des.to_string(),
            }
        }
        Role::Oberon => PollRoleResp {
            role: "Oberon".to_string(),
            role_des: "你是奥伯伦".to_string(),
            skill_des: "".to_string(),
        },
    };

    return resp;
}

pub fn map_ok_result<T>(r: T) -> axum::Json<T> {
    axum::Json(r)
}
