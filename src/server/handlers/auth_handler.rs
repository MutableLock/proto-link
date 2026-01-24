use crate::server::db::{challenges_db, tokens_db, users_db};
use crate::server::server_encrypted_codec::ServerEncriptedCodec;
use crate::structures::protolink_stype::{
    AuthChallenge, AuthRequestStruct, AuthResponse, ProtoLinkSType,
};
use crate::util::crypto::challenge_util::{generate_challenge, verify_challenge};

use aes_gcm::{Aes256Gcm, KeyInit};
use chrono::{Duration, Utc};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::MysqlConnection;

use std::net::SocketAddr;
use std::sync::Arc;

use tfserver::async_trait::async_trait;
use tfserver::server::handler::Handler;
use tfserver::structures::s_type;
use tfserver::structures::s_type::StructureType;
use tfserver::structures::traffic_proc::TrafficProcessorHolder;
use tfserver::structures::transport::Transport;
use tfserver::tokio::sync::{oneshot::Sender, Mutex};
use tfserver::tokio_util::bytes::BytesMut;
use tfserver::tokio_util::codec::Framed;

pub struct AuthHandler {
    db: Arc<Mutex<Pool<ConnectionManager<MysqlConnection>>>>,
}

impl AuthHandler {
    pub fn new(db: Arc<Mutex<Pool<ConnectionManager<MysqlConnection>>>>) -> Self {
        Self { db }
    }

    async fn conn(&self) -> diesel::r2d2::PooledConnection<ConnectionManager<MysqlConnection>> {
        self.db.lock().await.get().expect("DB connection failed")
    }

    fn empty_challenge() -> AuthChallenge {
        AuthChallenge::new(vec![], [0u8; 12], String::new())
    }

    pub async fn auth_request(&self, req: AuthRequestStruct) -> AuthChallenge {
        let mut conn = self.conn().await;

        let user = match users_db::UsersDb::find_user_by_login(&mut conn, &req.login) {
            Ok(user) => user,
            Err(_) => return Self::empty_challenge(),
        };

        let nonce = rand::random::<[u8; 12]>();
        let cipher = match Aes256Gcm::new_from_slice(&user.password_hash) {
            Ok(c) => c,
            Err(_) => return Self::empty_challenge(),
        };

        let (solution, challenge) = generate_challenge(&cipher, &nonce, 128, 256);

        if challenges_db::ChallengesDb::create_challenge(
            &mut conn,
            user.id,
            challenge.clone(),
            solution,
            nonce.to_vec(),
        )
        .is_err()
        {
            return Self::empty_challenge();
        }

        AuthChallenge::new(challenge, nonce, user.login)
    }

    pub async fn auth_challenge(&self, req: AuthChallenge) -> AuthResponse {
        let mut conn = self.conn().await;

        let user = match users_db::UsersDb::find_user_by_login(&mut conn, &req.login) {
            Ok(user) => user,
            Err(_) => {
                return AuthResponse::error("user not found");
            }
        };

        let challenges =
            match challenges_db::ChallengesDb::find_challenges_by_user_id(&mut conn, user.id) {
                Ok(c) if !c.is_empty() => c,
                _ => {
                    return AuthResponse::error("challenge not found");
                }
            };

        let chal = &challenges[0];

        if !verify_challenge(&chal.solution, &req.challenge) {
            let _ = challenges_db::ChallengesDb::delete_challenge(&mut conn, chal.id);
            return AuthResponse::error("incorrect");
        }

        let token = match tokens_db::TokensDb::create_token(
            &mut conn,
            user.id,
            (Utc::now() + Duration::hours(2)).naive_utc(),
        ) {
            Ok(token) => token,
            Err(_) => {
                return AuthResponse::error("token creation failed");
            }
        };

        AuthResponse {
            success: true,
            s_type: ProtoLinkSType::AuthResponse,
            message: token.to_string(),
        }
    }
}

impl AuthResponse {
    fn error(msg: &str) -> Self {
        Self {
            success: false,
            s_type: ProtoLinkSType::AuthResponse,
            message: msg.to_string(),
        }
    }
}

#[async_trait]
impl Handler for AuthHandler {
    type Codec = ServerEncriptedCodec;

    async fn serve_route(
        &mut self,
        _client_meta: (
            SocketAddr,
            &mut Option<Sender<Arc<Mutex<dyn Handler<Codec = Self::Codec>>>>>,
        ),
        s_type: Box<dyn StructureType>,
        mut data: BytesMut,
    ) -> Result<Vec<u8>, Vec<u8>> {
        let s_type = s_type
            .as_any()
            .downcast_ref::<ProtoLinkSType>()
            .unwrap()
            .clone();
        match s_type {
            ProtoLinkSType::AuthRequest => {
                let req = s_type::from_slice::<AuthRequestStruct>(data.as_mut())?;
                let resp = self.auth_request(req).await;
                Ok(s_type::to_vec(&resp).unwrap())
            }
            ProtoLinkSType::AuthChallenge => {
                let chal = s_type::from_slice::<AuthChallenge>(data.as_mut())?;
                let resp = self.auth_challenge(chal).await;
                Ok(s_type::to_vec(&resp).unwrap())
            }
            _ => Err("Malformed request".into()),
        }
    }

    async fn accept_stream(
        &mut self,
        _addr: SocketAddr,
        _stream: (
            Framed<Transport, Self::Codec>,
            TrafficProcessorHolder<Self::Codec>,
        ),
    ) {
        todo!()
    }
}
