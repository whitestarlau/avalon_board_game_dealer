#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Merlin,
    Percival,
    LoyalServant(i32),
    Morgana,
    Assassin,
    Oberon,
}

#[allow(dead_code)]
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
