use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

/**
 * 暂时只做7人局
 */
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    //梅林，下发时告诉ta所有邪恶方的号码（除莫德雷德）
    Merlin,
    //派西维尔，下发时告诉ta梅林和莫甘娜的号码
    Percival,
    //亚瑟的忠臣（Loyal Servant of Arthur)。7人局有两个忠臣。
    LS_of_Arthur(i32),

    //莫甘娜，下发时告诉ta刺客的号码
    Morgana,
    //刺客，下发时告诉ta莫甘娜的号码
    Assassin,
    //奥伯伦，邪恶方闭眼玩家
    Oberon,
    //以下暂不支持
    //莫德雷德，梅林无法看到他，不知道玩家中谁是莫德雷德。
    //Mordred,
    //莫德雷德的爪牙，无特殊能力的角色，但是知道邪恶阵营的其他人是谁
    // Minion_of_Mordred
}
