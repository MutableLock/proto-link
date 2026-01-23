use crate::server::db::{challenges_db, users_db};
use crate::server::server_encrypted_codec::ServerEncriptedCodec;
use crate::structures::protolink_stype::{
    AuthChallenge, AuthRequestStruct, AuthResponse, ProtoLinkSType,
};
use crate::util::crypto::challenge_util::{generate_challenge, verify_challenge};
use aes_gcm::{Aes256Gcm, KeyInit};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::MysqlConnection;
use std::net::SocketAddr;
use std::sync::Arc;
use tfserver::async_trait;
use tfserver::async_trait::async_trait;
use tfserver::server::handler::Handler;
use tfserver::structures::s_type::StructureType;
use tfserver::structures::traffic_proc::TrafficProcessorHolder;
use tfserver::structures::transport::Transport;
use tfserver::tokio::sync::oneshot::Sender;
use tfserver::tokio::sync::Mutex;
use tfserver::tokio_util::bytes::BytesMut;
use tfserver::tokio_util::codec::Framed;

pub struct AuthHandler {
    db_connection: Arc<Mutex<Pool<ConnectionManager<MysqlConnection>>>>,
}

impl AuthHandler {
    pub fn new(db_connection: Arc<Mutex<Pool<ConnectionManager<MysqlConnection>>>>) -> Self {
        Self { db_connection }
    }

    pub async fn auth_request(&mut self, req: AuthRequestStruct) -> AuthChallenge {
        let user = users_db::UsersDb::find_user_by_login(
            &mut self.db_connection.lock().await.get().unwrap(),
            req.login.as_str(),
        );
        if let Ok(user) = user {
            let nonce = rand::random::<[u8; 12]>();
            if let Ok(cipher) = Aes256Gcm::new_from_slice(user.password_hash.as_slice()) {
                let challenge = generate_challenge(&cipher, &nonce, 128, 256);
                if let Ok(_) = challenges_db::ChallengesDb::create_challenge(
                    &mut self.db_connection.lock().await.get().unwrap(),
                    user.id,
                    challenge.1.clone(),
                    challenge.0,
                    nonce.to_vec(),
                ) {
                    return AuthChallenge::new(challenge.1, nonce, user.login);
                }
            }
            AuthChallenge::new(vec![], [0u8; 12], "".to_string())
        } else {
            AuthChallenge::new(vec![], [0u8; 12], "".to_string())
        }
    }

    pub async fn auth_challenge(&mut self, req: AuthChallenge) -> AuthResponse {
        let user = users_db::UsersDb::find_user_by_login(
            &mut self.db_connection.lock().await.get().unwrap(),
            req.login.as_str(),
        );
        if let Ok(user) = user {
            let chal = challenges_db::ChallengesDb::find_challenges_by_user_id(
                &mut self.db_connection.lock().await.get().unwrap(),
                user.id,
            );
            if let Ok(chal) = chal {
                if !chal.is_empty() {
                    let chal = chal.first().unwrap();
                    if verify_challenge(&chal.solution, &req.challenge) {
                    } else {
                        challenges_db::ChallengesDb::delete_challenge(
                            &mut self.db_connection.lock().await.get().unwrap(),
                            chal.id,
                        )
                        .unwrap();
                        return AuthResponse {
                            success: false,
                            s_type: ProtoLinkSType::AuthResponse,
                            message: "incorrect".to_string(),
                        };
                    }
                }
                return AuthResponse {
                    success: false,
                    s_type: ProtoLinkSType::AuthResponse,
                    message: "challenge not found".to_string(),
                };
            }
            return AuthResponse {
                success: false,
                s_type: ProtoLinkSType::AuthResponse,
                message: "challenge not found".to_string(),
            };
        }
        AuthResponse {
            success: false,
            s_type: ProtoLinkSType::AuthResponse,
            message: "user not found".to_string(),
        }
    }
}

#[async_trait]
impl Handler for AuthHandler {
    type Codec = ServerEncriptedCodec;

    async fn serve_route(
        &mut self,
        client_meta: (
            SocketAddr,
            &mut Option<Sender<Arc<Mutex<dyn Handler<Codec = Self::Codec>>>>>,
        ),
        s_type: Box<dyn StructureType>,
        data: BytesMut,
    ) -> Result<Vec<u8>, Vec<u8>> {
        todo!()
    }

    async fn accept_stream(
        &mut self,
        add: SocketAddr,
        stream: (
            Framed<Transport, Self::Codec>,
            TrafficProcessorHolder<Self::Codec>,
        ),
    ) {
        todo!()
    }
}
