use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
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
pub struct NewScoreRequest {
    pub score : i64,
    pub limit : i64
}

#[derive(Serialize, Deserialize)]
pub struct NewScoreResponse {
    pub id : Uuid,
    pub scores : Vec<PlayerScore>
}

#[derive(Serialize, Deserialize)]
pub struct RenameScoreRequest {
    pub id : Uuid,
    pub name: String
}
