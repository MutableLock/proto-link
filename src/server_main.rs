use crate::server::handlers::auth_handler::AuthHandler;
use crate::structures::protolink_stype::ProtoLinkSType;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{r2d2, Connection, MysqlConnection};
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use tfserver::server::server_router::TcpServerRouter;
use tfserver::server::tcp_server::TcpServer;
use tfserver::tokio;
use tfserver::tokio::sync::Mutex;
use tfserver::codec::length_delimited::LengthDelimitedCodec;
use crate::server::server_encrypted_codec::ServerEncryptedTrafficProc;

mod util;


mod server;
mod structures;

async fn init_auth_server(
    pool: Arc<Mutex<Pool<ConnectionManager<MysqlConnection>>>>,
    codec_pool: Pool<ConnectionManager<MysqlConnection>>
) -> TcpServer<ServerEncryptedTrafficProc> {
    let enc_codec = ServerEncryptedTrafficProc::new(codec_pool);

    let auth_handler = Arc::new(Mutex::new(AuthHandler::new(pool.clone())));
    let mut router: TcpServerRouter<ServerEncryptedTrafficProc> =
        TcpServerRouter::new(Box::new(ProtoLinkSType::AuthResponse));
    router.add_route(
        auth_handler,
        "AUTH_HANDLER".to_string(),
        vec![
            Box::new(ProtoLinkSType::RegisterRequest),
            Box::new(ProtoLinkSType::AuthRequest),
        ],
    );
    router.commit_routes();
    let router = Arc::new(router);
    TcpServer::new(
        "0.0.0.0:8080".to_string(),
        router,
        None,
        enc_codec,
        None,
    )
    .await
}

async fn init_server(pool: Arc<Mutex<Pool<ConnectionManager<MysqlConnection>>>>) {
    
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
    let server_pool = 
        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
 

    
    let mut auth_server = init_auth_server(auth_pool, server_pool).await;
    auth_server.start().await;
    
    
    
    
    loop {}
}
