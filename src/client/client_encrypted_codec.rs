use crate::util::crypto::codec_util::*;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;
use std::io;
use tfserver::async_trait::async_trait;
use tfserver::codec::codec_trait::TfCodec;
use tfserver::structures::temp_transport::TempTransport;
use tfserver::structures::transport::Transport;
use tfserver::tokio::io::{AsyncReadExt, AsyncWriteExt};
use tfserver::tokio_util::bytes::{Bytes, BytesMut};
use tfserver::tokio_util::codec::{Decoder, Encoder, Framed, LengthDelimitedCodec};
use tfserver::futures_util::{SinkExt, StreamExt};
#[derive(Clone)]
pub struct ClientEncryptedCodec {
    login: String,
    password_hash: Vec<u8>,
    state: CryptoState,
    base_codec: LengthDelimitedCodec,
}

impl ClientEncryptedCodec {
    pub fn new(login: String, password_hash: Vec<u8>) -> Self {
        ClientEncryptedCodec {
            login,
            password_hash,
            state: CryptoState::Uninitialized,
            base_codec: LengthDelimitedCodec::new(),
        }
    }
}

#[async_trait]
impl TfCodec for ClientEncryptedCodec {
    async fn initial_setup(&mut self, transport: &mut Transport) -> bool {
        let tmp_transport = TempTransport::new(transport);
        let mut framed = Framed::new(tmp_transport, LengthDelimitedCodec::new());

        let l_bytes_org = self.login.as_bytes();

        if framed.send(Bytes::copy_from_slice(l_bytes_org)).await.is_err() {
            return false;
        }

        let msg = match framed.next().await {
            Some(Ok(res)) => res,
            _ => return false,
        };
        if msg.len() < 144 {
            return false;
        }
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&msg[msg.len() - 12..msg.len()]);
        let ciphertext = &msg[..msg.len() - 12];

        let handshake_key = derive_handshake_key(self.password_hash.as_slice());
        let cipher = match Aes256Gcm::new_from_slice(&handshake_key) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("{}", err.to_string());
                return false;
            }
        };

        let plaintext = match cipher.decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref()) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("{}", err.to_string());
                return false;
            }
        };

        if framed.send(Bytes::from(plaintext)).await.is_err() {
            return false;
        }

        let mut client_nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut client_nonce);
        if framed.send(Bytes::copy_from_slice(&client_nonce)).await.is_err() {
            return false;
        }

        let server_nonce_msg = match framed.next().await {
            Some(Ok(res)) => res,
            _ => return false,
        };
        if server_nonce_msg.len() != 12 {
            return false;
        }
        let mut server_nonce = [0u8; 12];
        server_nonce.copy_from_slice(&server_nonce_msg);

        let traffic_key =
            derive_traffic_key(self.password_hash.as_slice(), &client_nonce, &server_nonce);

        self.state = CryptoState::Established {
            cipher: Aes256Gcm::new_from_slice(&traffic_key).unwrap(),
            send_ctr: 0,
            recv_ctr: 0,
        };
        true
    }
}

impl Decoder for ClientEncryptedCodec {
    type Item = BytesMut;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let CryptoState::Established {
            cipher, recv_ctr, ..
        } = &mut self.state
        else {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"));
        };

        let Some(data) = self.base_codec.decode(src)? else {
            return Ok(None);
        };

        let nonce = make_nonce(*recv_ctr, NONCE_SERVER_TO_CLIENT);
        *recv_ctr += 1;

        let decrypted = cipher
            .decrypt(&nonce, data.as_ref())
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"))?;

        Ok(Some(BytesMut::from(Bytes::from(decrypted))))
    }
}

impl Encoder<Bytes> for ClientEncryptedCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let CryptoState::Established {
            cipher, send_ctr, ..
        } = &mut self.state
        else {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"));
        };

        let nonce = make_nonce(*send_ctr, NONCE_CLIENT_TO_SERVER);
        *send_ctr += 1;

        let encrypted = cipher
            .encrypt(&nonce, item.as_ref())
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"))?;

        self.base_codec.encode(Bytes::from(encrypted), dst)
    }
}
