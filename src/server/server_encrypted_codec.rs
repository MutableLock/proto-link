use crate::server::db::users_db::UsersDb;

use crate::util::crypto::challenge_util::{generate_challenge, verify_challenge};
use crate::util::crypto::codec_util::{
    derive_handshake_key, derive_traffic_key, make_nonce, CryptoState, NONCE_CLIENT_TO_SERVER,
    NONCE_SERVER_TO_CLIENT,
};
use aes_gcm::aead::consts::U12;
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::Aead;
use aes_gcm::aes::Aes256;
use aes_gcm::{Aes256Gcm, AesGcm, KeyInit, Nonce};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::MysqlConnection;
use hkdf::Hkdf;
use rand::{Rng, RngCore};
use sha2::Sha256;
use std::io;
use std::io::Error;
use tfserver::async_trait::async_trait;
use tfserver::codec::codec_trait::TfCodec;
use tfserver::structures::temp_transport::TempTransport;
use tfserver::structures::transport::Transport;
use tfserver::tokio::io::{AsyncReadExt, AsyncWriteExt};
use tfserver::tokio_util::bytes::{Bytes, BytesMut};
use tfserver::tokio_util::codec::{Decoder, Encoder, Framed, LengthDelimitedCodec};
use tfserver::futures_util::{SinkExt, StreamExt};
#[derive(Clone)]
pub struct ServerEncriptedCodec {
    pool: Pool<ConnectionManager<MysqlConnection>>,
    crypto: CryptoState,
    base_codec: LengthDelimitedCodec,
}

impl ServerEncriptedCodec {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        ServerEncriptedCodec {
            pool,
            crypto: CryptoState::Uninitialized,
            base_codec: LengthDelimitedCodec::new(),
        }
    }
}
#[async_trait]
impl TfCodec for ServerEncriptedCodec {
    async fn initial_setup(&mut self, transport: &mut Transport) -> bool {
        let tmp_transport = TempTransport::new(transport);
        let mut framed = Framed::new(tmp_transport, LengthDelimitedCodec::new());

        let login_msg = match framed.next().await {
            Some(Ok(v)) => v,
            _ => return false,
        };

        let login = String::from_utf8_lossy(&login_msg).to_string();

        let mut conn = match self.pool.get() {
            Ok(c) => c,
            Err(_) => return false,
        };

        let user = match UsersDb::find_user_by_login(&mut conn, &login) {
            Ok(u) => u,
            Err(_) => return false,
        };

        let key = derive_handshake_key(user.password_hash.as_slice());
        let cipher = match Aes256Gcm::new_from_slice(&key) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("{}", err.to_string());
                return false;
            }
        };

        let mut nonce = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce);
        
        
        
        let (challenge, ciphertext) = generate_challenge(&cipher, &nonce, rand::rng().random_range(128..256), rand::rng().random_range(257..1024));

        let msg = make_challenge_message(nonce, ciphertext);
        if framed.send(Bytes::copy_from_slice(&msg)).await.is_err() {
            return false;
        }

        let response = match framed.next().await {
            Some(Ok(v)) => v,
            _ => return false,
        };

        if !verify_challenge(&challenge, &response) {
            return false;
        }

        let mut client_nonce = [0u8; 12];
        let client_nonce_msg = match framed.next().await {
            Some(Ok(v)) => v,
            _ => return false,
        };
        if client_nonce_msg.len() != 12 {
            return false;
        }
        client_nonce.copy_from_slice(&client_nonce_msg);

        let mut server_nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut server_nonce);
        if framed.send(Bytes::copy_from_slice(&server_nonce)).await.is_err() {
            return false;
        }

        let traffic_key =
            derive_traffic_key(user.password_hash.as_slice(), &client_nonce, &server_nonce);

        let traffic_cipher = Aes256Gcm::new_from_slice(&traffic_key).unwrap();

        self.crypto = CryptoState::Established {
            cipher: traffic_cipher,
            send_ctr: 0,
            recv_ctr: 0,
        };

        true
    }
}

fn generate_nonce() -> [u8; 12] {
    let mut n = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut n);
    n
}

async fn exchange_traffic_nonces(transport: &mut Transport) -> Option<([u8; 12], [u8; 12])> {
    // Client sends first
    let mut client_nonce = [0u8; 12];
    transport.read_exact(&mut client_nonce).await.ok()?;

    // Server responds
    let server_nonce = generate_nonce();
    transport.write_all(&server_nonce).await.ok()?;

    Some((client_nonce, server_nonce))
}

fn make_challenge_message(nonce: [u8; 12], challenge: Vec<u8>) -> Vec<u8> {
    let mut res = Vec::with_capacity(challenge.len() + nonce.len());
    res.extend_from_slice(&challenge);
    res.extend_from_slice(&nonce);
    res
}

impl Decoder for ServerEncriptedCodec {
    type Item = BytesMut;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let CryptoState::Established {
            cipher, recv_ctr, ..
        } = &mut self.crypto
        else {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"));
        };

        let Some(data) = self.base_codec.decode(src)? else {
            return Ok(None);
        };

        let nonce = make_nonce(*recv_ctr, NONCE_CLIENT_TO_SERVER);
        *recv_ctr += 1;

        let decrypted = cipher
            .decrypt(&nonce, data.as_ref())
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"))?;

        Ok(Some(BytesMut::from(Bytes::from(decrypted))))
    }
}

impl Encoder<Bytes> for ServerEncriptedCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let CryptoState::Established {
            cipher, send_ctr, ..
        } = &mut self.crypto
        else {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"));
        };

        let nonce = make_nonce(*send_ctr, NONCE_SERVER_TO_CLIENT);
        *send_ctr += 1;

        let encrypted = cipher
            .encrypt(&nonce, item.as_ref())
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"))?;

        self.base_codec.encode(Bytes::from(encrypted), dst)
    }
}
