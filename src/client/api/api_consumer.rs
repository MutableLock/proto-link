use serde::Deserialize;

use tfserver::structures::s_type;
use tfserver::structures::s_type::{StrongType};
use tfserver::tokio::sync::oneshot::{Receiver};
use tfserver::tokio_util::bytes::BytesMut;


pub async fn process_response_oneshot<r: for<'a> Deserialize<'a> + StrongType + Send + Sync>(
    rx: Receiver<BytesMut>,
) -> r {
    s_type::from_slice(rx.await.unwrap().as_mut()).expect("Failed to deserialize response")
}
