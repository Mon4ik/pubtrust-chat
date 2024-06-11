use std::{error, fmt};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};

use base64::Engine;
use base64::engine::general_purpose;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private, Public};
use openssl::rsa::Rsa;
use openssl::sign::{Signer, Verifier};

use crate::utils::{ClientSettings, DatabaseFile};

#[derive(Debug)]
struct VerifyError;

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid signature")
    }
}

impl error::Error for VerifyError {}

pub struct DataClient {
    client_settings: ClientSettings,

    pub rsa: Rsa<Private>,
    pub database_file: DatabaseFile,
}

impl DataClient {
    pub fn new(client_settings: ClientSettings) -> Self {
        let path = &client_settings.profile;
        let display = path.display();

        let (database_file, rsa) = if !path.exists() {
            let rsa = Rsa::generate(2048).unwrap();
            let mut file = match File::create(path) {
                Err(why) => {
                    panic!("couldn't create {}: {}", display, why);
                }
                Ok(file) => file,
            };

            let data = DatabaseFile {
                alias: "Guest".to_string(),
                private_key: String::from_utf8(rsa.private_key_to_pem().unwrap()).unwrap(),
                saved_aliases: HashMap::new(),
            };

            file.write(serde_json::to_string(&data).unwrap().as_bytes()).expect("bebra nuh");

            (data, rsa)
        } else {
            let file = match File::open(&path) {
                Err(why) => {
                    panic!("couldn't open {}: {}", display, why);
                }
                Ok(file) => file,
            };

            // Read the file contents into a string, returns `io::Result<usize>`
            let reader = BufReader::new(file);
            let database_file: DatabaseFile = serde_json::from_reader(reader).unwrap();

            let rsa = Rsa::private_key_from_pem(&database_file.private_key.as_bytes()).expect("why");

            (database_file, rsa)
        };

        // Open the path in read-only mode, returns `io::Result<File>`

        Self {
            rsa,
            database_file,
            client_settings,
        }
    }

    pub fn save_changes(&self) -> serde_json::Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.client_settings.profile)
            .unwrap();

        serde_json::to_writer(file, &self.database_file)
    }

    pub fn change_alias(&mut self, new_alias: String) -> serde_json::Result<()> {
        self.database_file.alias = new_alias;
        self.save_changes()
    }

    pub fn pubkey(&self) -> String {
        let pub_key: Vec<u8> = self.rsa.public_key_to_pem().unwrap();

        return String::from_utf8(pub_key).unwrap();
    }

    pub fn signature(&self, message: &String, timestamp: &u64) -> String {
        let mut signer = Signer::new(MessageDigest::sha256(), &self.get_pubkey()).unwrap();
        signer.update(format!("{};{}", message, timestamp).as_bytes()).unwrap();

        general_purpose::STANDARD.encode(&signer.sign_to_vec().unwrap())
    }

    pub fn try_verify(message: &String, timestamp: &u64, signature: &String, pubkey: &PKey<Public>) -> bool {
        match Verifier::new(MessageDigest::sha256(), pubkey) {
            Err(_e) => return false,
            Ok(mut verifier) => {
                verifier.update(format!("{};{}", message, timestamp).as_bytes()).unwrap();
                match general_purpose::STANDARD.decode(signature) {
                    Err(_e) => return false,
                    Ok(signature_bytes) => {
                        verifier.verify(&signature_bytes).unwrap_or(false)
                    }
                }
            }
        }
    }

    fn get_pubkey(&self) -> PKey<Private> {
        let keypair = self.rsa.clone();
        let keypair = PKey::from_rsa(keypair).unwrap();

        keypair
    }
}