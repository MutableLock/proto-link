use tfserver::tokio;
use crate::client::api::auth_api::AuthApi;
use crate::client::api::init_client_api;
use crate::structures::protolink_stype::{RegisterRequestStruct};

mod client;
mod structures;

pub struct TestReceiver;


#[tokio::main]
async fn main() {
    let conn = init_client_api( "127.0.0.1:8080".to_string(), "127.0.0.1".to_string()).await;
    let auth_api = AuthApi::new(conn);
    let mut request = RegisterRequestStruct::new();
    request.login = "hello_aiden".parse().unwrap();
    request.name = "aiden".parse().unwrap();
    request.password_hash_sha256 = "hello_hash".parse().unwrap();
    let res = auth_api.create_user(request).await.await;
    println!("{:?}", res);
    loop {}
}