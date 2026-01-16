use crate::client::api::auth_api::AuthApi;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tfserver::async_trait::async_trait;
use tfserver::client::ClientConnect;

use crate::structures::protolink_stype::{AuthResponse, RegisterRequestStruct};

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
        let mut hasher: Sha256 = Sha256::new();
        hasher.update(password);
        let mut request = RegisterRequestStruct::new();
        request.name = username.to_string();
        request.login = login.to_string();
        request.password_hash_sha256 = base64::encode(hasher.finalize());
        let res = self.auth_api.create_user(request).await.await;
        res.success
    }
}
