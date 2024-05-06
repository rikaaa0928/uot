use anyhow::Error;
use serde::{Deserialize, Serialize};
use rand::Rng;
use rand::distributions::Alphanumeric;

#[derive(Serialize, Deserialize)]
pub(crate) struct Auth {
    pub pw: String,
    rand: String,
}

pub(crate) fn generate_auth(pw: String) -> String {
    let rand = generate_random_string(255);
    let auth = Auth { pw, rand };
    serde_json::to_string(&auth).unwrap()
}

pub(crate) fn parse_auth(data: &str) -> Result<Auth, Error> {
    let auth: Auth = serde_json::from_str(data)?;
    Ok(auth)
}

pub(crate) fn encrypt(data: &[u8], mut key: u8) -> Vec<u8> {
    if key == 0 {
        key = 128;
    }
    let mut encrypted_data = Vec::with_capacity(data.len());
    for byte in data {
        encrypted_data.push(byte.wrapping_add(key));
    }
    encrypted_data
}

pub(crate) fn decrypt(data: &[u8], mut key: u8) -> Vec<u8> {
    if key == 0 {
        key = 128;
    }
    let mut decrypted_data = Vec::with_capacity(data.len());
    for byte in data {
        decrypted_data.push(byte.wrapping_sub(key));
    }
    decrypted_data
}

pub(crate) fn generate_random_string(max_length: usize) -> String {
    let length = rand::thread_rng().gen_range(1..max_length);
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    s
}

#[tokio::test]
async fn test_test() -> Result<(), Error> {
    let data: [u8; 4] = [0, 1, 255, 254];
    let mut encrypted_data = encrypt(&data, 1);
    let mut decrypted_data = decrypt(&encrypted_data, 1);
    println!("{:?} {:?}", encrypted_data, decrypted_data);

    encrypted_data = encrypt(&data, 2);
    decrypted_data = decrypt(&encrypted_data, 2);
    println!("{:?} {:?}", encrypted_data, decrypted_data);
    encrypted_data = encrypt(&data, 255);
    decrypted_data = decrypt(&encrypted_data, 255);
    println!("{:?} {:?}", encrypted_data, decrypted_data);
    encrypted_data = encrypt(&data, 0);
    decrypted_data = decrypt(&encrypted_data, 0);
    println!("{:?} {:?}", encrypted_data, decrypted_data);
    println!("{:?} {:?}", generate_random_string(10), generate_random_string(255));
    Ok(())
}