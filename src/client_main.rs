use tfserver::tokio;
use crate::client::api::auth_api::AuthApi;
use crate::client::api::init_client_api;
use crate::client::model::auth_model::AuthModel;
use crate::structures::protolink_stype::{RegisterRequestStruct};

pub mod client;
pub mod structures;
pub mod util;


pub struct TestReceiver;


#[tokio::main]
async fn main() {
    let conn = init_client_api( "127.0.0.1:8080".to_string(), "127.0.0.1".to_string()).await;
    let auth_model = AuthModel::new(conn);
    auth_model.create_user("hello", "hell_nah", "hello_larry!").await;
    
}
