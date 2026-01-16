use std::sync::Arc;
use tfserver::client::ClientConnect;
use tfserver::tokio_util::codec::LengthDelimitedCodec;

mod auth_api;

pub async fn init_client_api(
    server_dest: String,
    server_name: String,
) -> Arc<ClientConnect> {
    let client = ClientConnect::new(server_name, server_dest, None, LengthDelimitedCodec::new(), None, 16).await.unwrap();
    Arc::new(client)
}
