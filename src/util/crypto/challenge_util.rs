use aes_gcm::{Aes256Gcm, Nonce};
use aes_gcm::aead::Aead;
use rand::{Rng, RngCore};

/// Generates a random-size challenge and encrypts it using an existing cipher + nonce
///
/// Returns:
///   (plaintext_challenge, ciphertext)
pub fn generate_challenge(
    cipher: &Aes256Gcm,
    nonce: &[u8; 12],
    min_size: usize,
    max_size: usize,
) -> (Vec<u8>, Vec<u8>) {
    assert!(min_size > 0 && max_size >= min_size);

    let mut rng = rand::thread_rng();
    let size = rng.gen_range(min_size..=max_size);

    let mut challenge = vec![0u8; size];
    rng.fill_bytes(&mut challenge);

    let nonce = Nonce::from_slice(nonce);

    let ciphertext = cipher
        .encrypt(nonce, challenge.as_ref())
        .expect("AES-GCM encryption failed");

    (challenge, ciphertext)
}

pub fn verify_challenge(expected_challenge: &[u8], answer: &[u8]) -> bool {
    for i in 0..expected_challenge.len(){
        if expected_challenge[i] != answer[i]{
            return false;
        }
    }
    return true;
}
