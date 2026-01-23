use std::net::SocketAddr;
use std::sync::Arc;
use diesel::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use tfserver::async_trait::async_trait;
use tfserver::server::handler::Handler;
use tfserver::structures::s_type::StructureType;
use tfserver::structures::traffic_proc::TrafficProcessorHolder;
use tfserver::structures::transport::Transport;
use tfserver::tokio::sync::Mutex;
use tfserver::tokio::sync::oneshot::Sender;
use tfserver::tokio_util::bytes::BytesMut;
use tfserver::tokio_util::codec::Framed;
use crate::server::server_encrypted_codec::ServerEncriptedCodec;
use crate::structures::protolink_stype::{ChatHandlerResponseStruct, CreateChatRequestStruct};

pub struct ChatHandler {
    db_connection: Arc<Mutex<Pool<ConnectionManager<MysqlConnection>>>>,
}


impl ChatHandler {
    pub fn new(db_connection: Arc<Mutex<Pool<ConnectionManager<MysqlConnection>>>>) -> Self {
        Self { db_connection }
    }
    
    async fn create_chat_request(req: CreateChatRequestStruct) -> ChatHandlerResponseStruct {
        ChatHandlerResponseStruct{}
    }
}

#[async_trait]
impl Handler for ChatHandler {
    type Codec = ServerEncriptedCodec;

    async fn serve_route(&mut self, client_meta: (SocketAddr, &mut Option<Sender<Arc<Mutex<dyn Handler<Codec=Self::Codec>>>>>), s_type: Box<dyn StructureType>, data: BytesMut) -> Result<Vec<u8>, Vec<u8>> {
        todo!()
    }

    async fn accept_stream(&mut self, add: SocketAddr, stream: (Framed<Transport, Self::Codec>, TrafficProcessorHolder<Self::Codec>)) {
        todo!()
    }
}