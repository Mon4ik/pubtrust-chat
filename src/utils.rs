use std::collections::HashMap;
use std::path::PathBuf;

use openssl::pkey::{PKey, Public};
use openssl::sha;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[derive(Clone, Debug)]
pub struct ChatClient {
    pub alias: String,
    pub pubkey: PKey<Public>,
}

impl ChatClient {
    pub fn get_pubkey_hash(&self) -> Option<String> {
        let pubkey_bytes = self.pubkey.public_key_to_pem();
        if pubkey_bytes.is_err() { return None; }

        let mut hasher = sha::Sha1::new();
        hasher.update(&pubkey_bytes.unwrap());

        let pubkey_hash_finish = hasher.finish();

        Some(hex::encode(pubkey_hash_finish)[..6].to_string())
    }
}

#[derive(Debug)]
pub enum UIMessage {
    System(String),
    SystemError(String),
    Chat(ChatClient, String),
    DM(ChatClient, ChatClient, String),
}

#[derive(Clone, Debug)]
pub struct UIHelpCommand {
    pub(crate) name: String,
    pub(crate) description: String,
}

// pub enum UIActionType {
//     ChangeAlias,
//     ChangeTopic,
//     SendMessage,
//     SendDM,
// }

#[derive(Debug)]
pub enum UIAction {
    ChangeAlias(String),
    ChangeTopic(String),
    SendMessage(String),
    SendDM(String, String),
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseFile {
    pub alias: String,
    pub private_key: String,

    /// H
    #[serde_as(as = "Vec<(_, _)>")]
    pub saved_aliases: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct ClientSettings {
    pub host: String,
    pub port: u16,
    pub topic: String,
    pub profile: PathBuf,
}