use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct PlayerScore {
    pub player : String,
    pub score : i64
}
