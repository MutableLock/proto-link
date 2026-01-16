use std::sync::Arc;
use tfserver::async_trait::async_trait;
use tfserver::client::{ClientConnect, ClientRequest, DataConsumer, DataRequest, HandlerInfo};
use tfserver::tokio::sync::Mutex;
use tfserver::tokio_util::bytes::BytesMut;
use crate::structures::protolink_stype::{ProtoLinkSType, RegisterRequestStruct};

pub struct AuthApi{
    handler_info: HandlerInfo,
    conn: Arc<ClientConnect>,
}

impl AuthApi {
    pub fn new(conn: Arc<ClientConnect>) -> Self {
        Self{
            handler_info: HandlerInfo::new_named("AUTH_HANDLER".to_string()),
            conn
        }
    }

    async fn build_request(&self, data: Vec<u8>, on_received: Arc<Mutex<dyn DataConsumer>>, id: u64) -> ClientRequest{
        let mut res = ClientRequest{ req: DataRequest {
            handler_info: self.handler_info.clone(),
            data,
            s_type: Box::new(ProtoLinkSType::AuthRequest),
        }, consumer: on_received ,
            payload_id: id,
        };
        res
    }

    pub async fn create_user(&self, request: RegisterRequestStruct) {

        conn.dispatch_request()
    }
}