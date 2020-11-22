use uuid::Uuid;
use std::convert::TryInto;
use sha2::{Sha256, Digest};
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
    Response { id : Uuid, index : i64, scores : Vec<PlayerScore> },
    Error(String)
}

#[derive(Serialize, Deserialize)]
pub struct RenameScoreRequest {
    pub id : Uuid,
    pub name: String
}

pub fn rand128<T : RngCore + CryptoRng>(rng : &mut T) -> [u8; 16] {
    let mut result = [0u8; 16];
    rng.fill_bytes(&mut result);
    return result;
}

pub fn rand256<T : RngCore + CryptoRng>(rng : &mut T) -> [u8; 32] {
    let mut result = [0u8; 32];
    rng.fill_bytes(&mut result);
    return result;
}

pub fn validate_proof_of_work(session_id : [u8; 32], proof_of_work : [u8; 32], degree : usize) -> (bool, [u8; 32]) {
    let sha256 = Sha256::new()
        .chain(session_id)
        .chain(proof_of_work)
        .finalize();

    let num_bytes = degree / 8;
    let num_bits = degree % 8;
    let mask = !(0xffu8 << num_bits >> num_bits);

    for i in 0..num_bytes {
        if sha256[i] != 0 {
            return (false, sha256.as_slice().try_into().expect("wrong size"));
        }
    }

    if sha256[num_bytes] & mask != 0 {
        return (false, sha256.as_slice().try_into().expect("wrong size"));
    }

    return (true, sha256.as_slice().try_into().expect("wrong size"));
}

pub fn proof_of_work(session_id : [u8; 32], seed : u64, degree : usize) -> [u8; 32] {
    let mut rng = StdRng::seed_from_u64(seed);

    loop {
        let test = rand256(&mut rng);

        if validate_proof_of_work(session_id, test, degree).0 {
            return test;
        }
    }
}

