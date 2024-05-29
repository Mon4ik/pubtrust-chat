use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[derive(Debug)]
pub enum UIMessageType {
    System,
    SystemError,
    Chat,
    DM,
}

#[derive(Debug)]
pub struct UIMessage {
    pub message_type: UIMessageType,
    pub author: String,
    pub message: String,
}

impl UIMessage {
    pub fn system(message: String) -> Self {
        Self {
            message_type: UIMessageType::System,
            author: String::new(),
            message
        }
    }

    pub fn system_error(message: String) -> Self {
        Self {
            message_type: UIMessageType::SystemError,
            author: String::new(),
            message
        }
    }


    pub fn dm(client1: String, client2: String, message: String) -> Self {
        Self {
            message_type: UIMessageType::DM,
            author: format!("{} → {}", client1, client2),
            message
        }
    }

    pub fn chat(author: String, message: String) -> Self {
        Self {
            message_type: UIMessageType::Chat,
            author,
            message
        }
    }
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