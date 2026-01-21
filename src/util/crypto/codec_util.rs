use aes_gcm::{Aes256Gcm, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;
use tfserver::sha2::digest::consts::U12;

pub const NONCE_CLIENT_TO_SERVER: [u8; 4] = [0, 0, 0, 1];
pub const NONCE_SERVER_TO_CLIENT: [u8; 4] = [0, 0, 0, 2];
#[derive(Clone)]
pub enum CryptoState {
    Uninitialized,
    Established {
        cipher: Aes256Gcm,
        send_ctr: u64,
        recv_ctr: u64,
    },
}


pub fn derive_traffic_key(
    password_hash: &[u8],
    client_nonce: &[u8; 12],
    server_nonce: &[u8; 12],
) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, password_hash);

    let mut info = Vec::with_capacity(7 + 12 + 12);
    info.extend_from_slice(b"traffic");
    info.extend_from_slice(client_nonce);
    info.extend_from_slice(server_nonce);

    let mut key = [0u8; 32];
    hk.expand(&info, &mut key).unwrap();
    key
}

pub fn derive_handshake_key(password_hash: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, password_hash);
    let mut key = [0u8; 32];
    hk.expand(b"handshake-key", &mut key).unwrap();
    key
}

pub fn make_nonce(counter: u64, dir: [u8; 4]) -> Nonce<U12> {
    let mut nonce = [0u8; 12];
    nonce[..4].copy_from_slice(&dir);
    nonce[4..].copy_from_slice(&counter.to_be_bytes());
    Nonce::from_slice(&nonce).clone()
}