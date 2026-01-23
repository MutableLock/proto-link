use crate::server::handlers::register_handler::AuthHandler;
use crate::server::handlers::chat_handler::ChatHandler;
use crate::server::server_encrypted_codec::ServerEncriptedCodec;
use crate::structures::protolink_stype::ProtoLinkSType;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{r2d2, Connection, MysqlConnection};
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use tfserver::codec::length_delimited::LengthDelimitedCodec;
use tfserver::server::server_router::TcpServerRouter;
use tfserver::server::tcp_server::TcpServer;
use tfserver::tokio;
use tfserver::tokio::sync::Mutex;

mod util;

mod server;
mod structures;

async fn init_auth_server(
    pool: Arc<Mutex<Pool<ConnectionManager<MysqlConnection>>>>,
) -> TcpServer<LengthDelimitedCodec> {
    let enc_codec = LengthDelimitedCodec::new();

    let auth_handler = Arc::new(Mutex::new(AuthHandler::new(pool.clone())));
    let mut router: TcpServerRouter<LengthDelimitedCodec> =
        TcpServerRouter::new(Box::new(ProtoLinkSType::AuthResponse));
    router.add_route(
        auth_handler,
        "REGISTER_HANDLER".to_string(),
        vec![
            Box::new(ProtoLinkSType::RegisterRequest),
        ],
    );
    router.commit_routes();
    let router = Arc::new(router);
    TcpServer::new("0.0.0.0:8080".to_string(), router, None, enc_codec, None).await
}

async fn init_server(
    pool: Arc<Mutex<Pool<ConnectionManager<MysqlConnection>>>>,
    codec_pool: Pool<ConnectionManager<MysqlConnection>>,
) {
    let enc_codec = ServerEncriptedCodec::new(codec_pool);
    let mut router: TcpServerRouter<ServerEncriptedCodec> =
        TcpServerRouter::new(Box::new(ProtoLinkSType::CreateChat));

    let chat_handler = Arc::new(Mutex::new(ChatHandler::new(pool.clone())));
    router.add_route(
        chat_handler,
        "CHAT_HANDLER".to_string(),
        vec![Box::new(ProtoLinkSType::CreateChat)],
    );
    router.commit_routes();
    let router = Arc::new(router);
    TcpServer::new("0.0.0.0:8090".to_string(), router, None, enc_codec, None).await;
}

#[tokio::main]
pub async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url.clone());
    let auth_pool = Arc::new(Mutex::new(
        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool."),
    ));

    let manager = ConnectionManager::<MysqlConnection>::new(database_url.clone());
    let enc_pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let manager = ConnectionManager::<MysqlConnection>::new(database_url.clone());
    let server_pool = Arc::new(Mutex::new(
        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool."),
    ));

    let mut auth_server = init_auth_server(auth_pool).await;
    let mut server = init_server(server_pool.clone(), enc_pool).await;
    auth_server.start().await.await;
}
