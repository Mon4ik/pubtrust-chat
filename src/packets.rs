use serde::{Deserialize, Serialize};

#[repr(u8)]
pub enum PacketType {
    ReqAnnouncement = 001u8,
    Announcement    = 002u8,
    ChatMessage     = 003u8,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ReqAnnouncement {
    pub version: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Announcement {
    pub alias: String,
    pub pub_key: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ChatMessage {
    pub message: String,
    pub signature: String,
    pub timestamp: u64,
}

/// Initial packet for DM start
///  - `for_hash` - hash of public key of another client
///  - `shared_secret` - secret, encrypted with another client's public key
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DirectMessageShare {
    pub for_hash: String,
    pub shared_secret: String,
}

/// Packet of DirectMessage
///  - `for_hash` - hash of public key of another client
///  - `message` - encrypted message with `shared_secret`
///  - `signature` - signature of decrypted message
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DirectMessage {
    pub to_hash: String,
    pub message: String,
    pub signature: String,
    pub timestamp: u64,
}

