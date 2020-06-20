use uuid::Uuid;
use serde::{Serialize, Deserialize};
use rand::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerScore {
    pub index : i64,
    pub name : String,
    pub score : i64
}

#[derive(Serialize, Deserialize)]
pub struct AddScoreRequest {
    pub name : String,
    pub score : i64
}

#[derive(Serialize, Deserialize)]
pub struct ListScoresRequest {
    pub limit : Option<i64>
}

#[derive(Serialize, Deserialize)]
pub struct ListScoresResponse {
    pub scores : Vec<PlayerScore>
}

#[derive(Serialize, Deserialize)]
pub struct NewScoreRequest {
    pub score : i64,
    pub session_id : String,
    pub proof_of_work : String,
    pub limit : i64
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NewScoreResponse {
    Response { id : Uuid, scores : Vec<PlayerScore> },
    Error(String)
}

#[derive(Serialize, Deserialize)]
pub struct RenameScoreRequest {
    pub id : Uuid,
    pub name: String
}

pub fn rand128() -> [u8; 16] {
    let mut result = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut result);
    return result;
}

pub fn rand256() -> [u8; 32] {
    let mut result = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut result);
    return result;
}
