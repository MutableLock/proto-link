use crate::util::crypto::codec_util::*;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;
use std::io;
use tfserver::async_trait::async_trait;
use tfserver::codec::codec_trait::TfCodec;
use tfserver::structures::transport::Transport;
use tfserver::tokio::io::{AsyncReadExt, AsyncWriteExt};
use tfserver::tokio_util::bytes::{Bytes, BytesMut};
use tfserver::tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};
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
        let mut login = [0u8; 256];
        let l_bytes_org = self.login.as_bytes();
        login[..l_bytes_org.len()].copy_from_slice(l_bytes_org);

        match transport.write_all(&login).await {
            Ok(_) => {}
            Err(_) => {
                return false;
            }
        }

        // 2. Receive encrypted challenge
        let (ciphertext, nonce) = match read_challenge_message(transport).await {
            Some(res) => res,
            None => {
                return false;
            }
        };

        // 3. Setup handshake cipher
        let handshake_key = derive_handshake_key(self.password_hash.as_slice());
        let cipher = match Aes256Gcm::new_from_slice(&handshake_key){
            Ok(res) => res,
            Err(err) => {
                eprintln!("{}", err.to_string());
                return false;
            }
        };

        // 4. Decrypt challenge
        let plaintext = match cipher.decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref()) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("{}", err.to_string());
                return false;
            }
        };

        // 5. Send plaintext challenge back
        if transport.write_all(&plaintext).await.is_err() {
            return false;
        }

        // 6. Generate and send client traffic nonce
        let mut client_nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut client_nonce);
        if transport.write_all(&client_nonce).await.is_err() {
            return false;
        }

        // 7. Receive server traffic nonce
        let mut server_nonce = [0u8; 12];
        if transport.read_exact(&mut server_nonce).await.is_err() {
            return false;
        }

        // 8. Derive traffic key
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

async fn read_challenge_message(transport: &mut Transport) -> Option<(Vec<u8>, [u8; 12])> {
    let mut buf = [0u8; 144];
    transport.read_exact(&mut buf).await.ok()?;

    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&buf[132..]);

    let ciphertext = buf[..132].to_vec();
    Some((ciphertext, nonce))
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
