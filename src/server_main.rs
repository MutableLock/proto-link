use std::env;
use std::sync::{Arc};
use diesel::{r2d2, Connection, MysqlConnection};
use diesel::r2d2::ConnectionManager;
use dotenvy::dotenv;
use tfserver::server::server_router::TcpServerRouter;
use tfserver::server::tcp_server::TcpServer;
use tfserver::tokio;
use tfserver::tokio::sync::Mutex;
use tfserver::tokio_util::codec::LengthDelimitedCodec;
use crate::server::handlers::auth_handler::AuthHandler;
use crate::structures::protolink_stype::ProtoLinkSType;

mod structures;
mod server;

#[tokio::main]
pub async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    let pool =Arc::new(Mutex::new( r2d2::Pool::builder().build(manager).expect("Failed to create pool.")));
    let auth_handler =Arc::new(Mutex::new( AuthHandler::new(pool.clone())));
    let mut router: TcpServerRouter<LengthDelimitedCodec> = TcpServerRouter::new(Box::new(ProtoLinkSType::AuthResponse));
    router.add_route(auth_handler, "AUTH_HANDLER".to_string(), vec![Box::new(ProtoLinkSType::RegisterRequest), Box::new(ProtoLinkSType::AuthRequest)]);


    let router = Arc::new(router);
    let mut server = TcpServer::new("0.0.0.0:8080".to_string(), router, None, LengthDelimitedCodec::new(), None).await;
    server.start().await;
    loop {

    }
}
