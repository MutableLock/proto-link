use crate::server::db::users_db;
use crate::server::db::users_db::UsersDb;
use crate::structures::protolink_stype::{
    AuthRequestStruct, AuthResponse, ProtoLinkSType, RegisterRequestStruct,
};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::MysqlConnection;
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use tfserver::async_trait::async_trait;
use tfserver::codec::length_delimited::LengthDelimitedCodec;
use tfserver::server::handler::Handler;
use tfserver::structures::s_type;
use tfserver::structures::s_type::StructureType;
use tfserver::structures::traffic_proc::TrafficProcessorHolder;
use tfserver::structures::transport::Transport;
use tfserver::tokio::sync::Mutex;
use tfserver::tokio_util::bytes::BytesMut;
use tfserver::tokio_util::codec::{Framed};

pub struct AuthHandler {
    db_connection: Arc<Mutex<Pool<ConnectionManager<MysqlConnection>>>>,
}
impl AuthHandler {
    pub fn new(db_connection: Arc<Mutex<Pool<ConnectionManager<MysqlConnection>>>>) -> Self {
        Self { db_connection }
    }
    

    async fn register_request(&self, request: RegisterRequestStruct) -> AuthResponse {
        let mut conn = self.db_connection.lock().await.get().unwrap();
        if let Ok(exists) = users_db::UsersDb::is_user_exists(&mut conn, request.login.as_str()) {
            if exists {
                return AuthResponse {
                    s_type: ProtoLinkSType::AuthResponse,
                    success: false,
                    message: "User already exists".into(),
                };
            }
            if let Ok(res) = UsersDb::create_user(
                &mut conn,
                request.login,
                request.name,
                request.password_hash_sha256_hkdf
            ) {
                return AuthResponse {
                    s_type: ProtoLinkSType::AuthResponse,
                    success: true,
                    message: "".into(),
                };
            }
        }
        AuthResponse {
            s_type: ProtoLinkSType::AuthResponse,
            success: false,
            message: "internal database error".into(),
        }
    }
}
#[async_trait]
impl Handler for AuthHandler {
    type Codec = LengthDelimitedCodec;

    async fn serve_route(
        &mut self,
        client_meta: (
            SocketAddr,
            &mut Option<
                tfserver::tokio::sync::oneshot::Sender<
                    Arc<Mutex<(dyn Handler<Codec = LengthDelimitedCodec> + 'static)>>,
                >,
            >,
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
            ProtoLinkSType::RegisterRequest => {
                let request = s_type::from_slice::<RegisterRequestStruct>(data.as_mut());
                if request.is_err() {
                    return Err("Malformed request".into());
                } else {
                    let request = request.unwrap();
                    let resp = self.register_request(request).await;
                    return Ok(s_type::to_vec(&resp).unwrap());
                }
            }
            
            _ => {
                return Err("Malformed request".into());
            }
        }
    }

    async fn accept_stream(
        &mut self,
        add: SocketAddr,
        stream: (
            Framed<Transport, LengthDelimitedCodec>,
            TrafficProcessorHolder<LengthDelimitedCodec>,
        ),
    ) {
        todo!()
    }
}
