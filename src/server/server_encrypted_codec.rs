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
use rand::RngCore;
use sha2::Sha256;
use std::io;
use std::io::Error;
use tfserver::async_trait::async_trait;
use tfserver::codec::codec_trait::TfCodec;
use tfserver::structures::transport::Transport;
use tfserver::tokio::io::{AsyncReadExt, AsyncWriteExt};
use tfserver::tokio_util::bytes::{Bytes, BytesMut};
use tfserver::tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};
#[derive(Clone)]
pub struct ServerEncryptedTrafficProc {
    pool: Pool<ConnectionManager<MysqlConnection>>,
    crypto: CryptoState,
    base_codec: LengthDelimitedCodec,
}

impl ServerEncryptedTrafficProc {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        ServerEncryptedTrafficProc {
            pool,
            crypto: CryptoState::Uninitialized,
            base_codec: LengthDelimitedCodec::new(),
        }
    }
}
#[async_trait]
impl TfCodec for ServerEncryptedTrafficProc {
    async fn initial_setup(&mut self, transport: &mut Transport) -> bool {
        let login = match read_null_terminated_string(transport).await {
            Some(v) => v,
            None => return false,
        };

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

        let (challenge, ciphertext) = generate_challenge(&cipher, &nonce, 116, 116);

        let msg = make_challenge_message(nonce, ciphertext);
        match transport.write_all(&msg).await {
            Ok(_) => {}
            Err(_) => {
                return false;
            }
        }

        let mut response = vec![0u8; challenge.len()];
        match transport.read_exact(&mut response).await {
            Ok(_) => {}
            Err(_) => {
                return false;
            }
        }

        if !verify_challenge(&challenge, &response) {
            return false;
        }

        let (client_nonce, server_nonce) = match exchange_traffic_nonces(transport).await {
            Some(v) => v,
            None => return false,
        };

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

async fn read_null_terminated_string(transport: &mut Transport) -> Option<String> {
    let mut buffer = [0u8; 256];
    transport.read_exact(&mut buffer).await.ok()?;
    let mut end = 0;
    for i in 0..256 {
        if buffer[i] == 0 {
            end = i;
            break;
        }
    }
    if end == 0 {
        return None;
    }
    return Some(String::from_utf8_lossy(&buffer[..end]).to_string());
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

fn make_challenge_message(nonce: [u8; 12], challenge: Vec<u8>) -> [u8; 144] {
    let mut res = [0u8; 144];
    res[..132].copy_from_slice(&challenge.as_slice()[..132]);
    res[132..].copy_from_slice(&nonce);
    res
}

impl Decoder for ServerEncryptedTrafficProc {
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

impl Encoder<Bytes> for ServerEncryptedTrafficProc {
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
