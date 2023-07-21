use chrono::{DateTime, Utc};
use nostr::{Timestamp, SECP256K1};
use secp256k1::schnorr::Signature;
use secp256k1::{Message, XOnlyPublicKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct NostrEvent {
    pub kind: i32,
    pub created_at: i64,
    pub public_key: String,
    pub signature: String,
    pub url: String,
    pub tags: Vec<Vec<String>>,
}
#[derive(Debug, Deserialize)]
pub struct NostrNip98Event {
    pub id: String,
    pub kind: i32,
    pub created_at: i64,
    pub pubkey: String,
    pub content: String,
    pub sig: String,
    pub tags: Vec<Vec<String>>,
}

// impl NostrNip98Event {
//     pub fn verify(&self) -> bool {
//         let id = &self.id;
//         let message = Message::from_slice(id.as_bytes())?;
//         SECP256K1
//             .verify_schnorr(&self.sig, &message, &self.pubkey)
//             .map_err(|_| Error::InvalidSignature)
//     }
// }
