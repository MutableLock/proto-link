use std::sync::Arc;
use hkdf::Hkdf;
use sha2::Sha256;
use tfserver::client::ClientConnect;
use tfserver::codec::length_delimited::LengthDelimitedCodec;
use crate::client::client_encrypted_codec::ClientEncryptedCodec;

pub mod auth_api;
pub mod api_consumer;

pub async fn init_client_api(
    server_dest: String,
    server_name: String,
) -> Arc<ClientConnect> {
    let hk = Hkdf::<Sha256>::new(None, "hello_larry!".as_bytes());

    let mut key = [0u8; 32];
    hk.expand(b"aes-256-key", &mut key).unwrap();

    let codec = ClientEncryptedCodec::new("la11y".parse().unwrap(), key.to_vec());

    let client = ClientConnect::new(server_name, server_dest, None, codec, None, 16).await.unwrap();
    Arc::new(client)
}
