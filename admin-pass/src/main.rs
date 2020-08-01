use anyhow;
use hex;
use rand::prelude::*;
use sha2::{Sha256, Digest};

pub fn rand256() -> [u8; 32] {
    let mut result = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut result);
    return result;
}

fn main() -> anyhow::Result<()> {
    let login = "admin";

    let mut password_bytes = [0u8; 9];
    rand::thread_rng().fill_bytes(&mut password_bytes);
    let password = base64::encode(password_bytes);

    // let password = "password";
    
    let password_bytes = password.as_bytes();

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);
    let salt = hex::encode_upper(salt_bytes);

    let hash_bytes = Sha256::new()
        .chain(password_bytes)
        .chain(salt_bytes)
        .finalize();

    let hash = hex::encode_upper(hash_bytes);

    println!(
        "// {}:{}\n(\"{}\", \"{}\", \"{}\")",
        login,
        password,
        login,
        hash,
        salt);

    return Ok(());
}
