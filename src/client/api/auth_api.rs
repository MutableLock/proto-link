use crate::client::api::api_consumer::{process_response_oneshot};
use crate::structures::protolink_stype::{AuthResponse, ProtoLinkSType, RegisterRequestStruct};
use std::sync::Arc;
use tfserver::client::{ClientConnect, ClientRequest, DataRequest, HandlerInfo};
use tfserver::structures::s_type;
use tfserver::structures::s_type::{StrongType, StructureType};
use tfserver::tokio::sync::{oneshot, Mutex};
use tfserver::tokio::sync::oneshot::{Receiver, Sender};
use tfserver::tokio_util::bytes::BytesMut;

pub struct AuthApi {
    handler_info: HandlerInfo,
    conn: Arc<ClientConnect>,
}

impl AuthApi {
    pub fn new(conn: Arc<ClientConnect>) -> Self {
        Self {
            handler_info: HandlerInfo::new_named("AUTH_HANDLER".to_string()),
            conn
           ,
        }
    }

    async fn build_request(
        &self,
        data: Vec<u8>,
        on_received: Sender<BytesMut>,
        s_type: Box<dyn StructureType>,
        id: u64,
    ) -> ClientRequest {
        let mut res = ClientRequest {
            req: DataRequest {
                handler_info: self.handler_info.clone(),
                data,
                s_type,
            },
            consumer: on_received,
            payload_id: id,
        };

        res
    }

    pub async fn create_user(
        &self,
        request: RegisterRequestStruct,
    ) -> impl std::future::Future<Output = AuthResponse> {
        let (tx, rx) = oneshot::channel();

        let req = self
            .build_request(
                s_type::to_vec(&request).unwrap(),
                tx,
                Box::new(ProtoLinkSType::RegisterRequest),
                0
            )
            .await;
        self.conn
            .dispatch_request(req)
            .await
            .expect("Failed to dispatch request");
        process_response_oneshot(rx)
    }


}
