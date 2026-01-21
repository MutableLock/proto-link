use crate::client::api::auth_api::AuthApi;
use hkdf::Hkdf;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tfserver::client::ClientConnect;

use crate::structures::protolink_stype::RegisterRequestStruct;

pub struct AuthModel {
    auth_api: AuthApi,
}

impl AuthModel {
    pub fn new(conn: Arc<ClientConnect>) -> Self {
        Self {
            auth_api: AuthApi::new(conn),
        }
    }

    pub async fn create_user(&self, username: &str, login: &str, password: &str) -> bool {
        let hk = Hkdf::<Sha256>::new(None, password.as_bytes());

        let mut key = [0u8; 32];
        hk.expand(b"aes-256-key", &mut key).unwrap();

        let mut request = RegisterRequestStruct::new();

        request.name = username.to_string();
        request.login = login.to_string();
        request.password_hash_sha256_hkdf = key.to_vec();
        let res = self.auth_api.create_user(request).await.await;
        res.success
    }
}
