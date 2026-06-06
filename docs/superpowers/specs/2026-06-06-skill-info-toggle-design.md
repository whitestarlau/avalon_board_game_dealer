# 技能信息展示开关设计

## 概述

添加管理员开关"是否展示技能信息"。关闭时，玩家仍能看到角色名称和描述，但不会看到技能信息（如梅林不再获知邪恶方号码、派西维尔不再看到梅林和莫甘娜号码等）。

## 方案

后端状态 + API 传递（方案 A）。

## 后端变更

### AppState 新增字段

```rust
pub show_skill_info: Arc<RwLock<bool>>,  // 默认 true
```

### 新增 API

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/admin/skill_info_status` | 获取当前开关状态 |
| POST | `/api/admin/toggle_skill_info` | 切换开关，返回新状态 |

### 响应结构

```rust
#[derive(Serialize)]
pub struct SkillInfoStatusResp {
    pub show_skill_info: bool,
}
```

### 修改 build_poll_role_resp

- 接收 `AppState` 参数（已有）
- 读取 `show_skill_info` 状态
- 如果 `false`，将 `skill_des` 设为空字符串
- `role_name` 和 `role_des` 正常返回

### new_game 重置

- 在 `new_game` handler 中将 `show_skill_info` 重置为 `true`

## 前端变更

### AdminView.vue

- 在当前游戏区域添加开关按钮"展示技能信息"
- 页面加载时调用 `GET /api/admin/skill_info_status` 获取初始状态
- 点击开关调用 `POST /api/admin/toggle_skill_info` 并更新本地状态

### RoleCard.vue

- 无需改动（已有 `v-if="role.skill_des"` 条件渲染）

## 数据流

1. 管理员在 AdminView 点击开关
2. 前端调用 `POST /api/admin/toggle_skill_info`
3. 后端更新 `show_skill_info` 状态
4. 玩家下次 `poll_player_role` 时，若开关关闭则 `skill_des` 为空
5. RoleCard 因 `skill_des` 为空不渲染技能信息区域
