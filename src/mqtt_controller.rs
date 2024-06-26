use std::fmt::Debug;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use nanoid::nanoid;
use openssl::pkey::PKey;
use rumqttc::{Client, ClientError, Event, Incoming, MqttOptions, Publish, QoS};
use serde::Serialize;

use crate::data_client::DataClient;
use crate::packets::{self, PacketType};
use crate::utils::{ChatClient, ClientSettings, UIAction, UIMessage};

pub struct MqttController {
    client: Client,
    chat_clients: Vec<ChatClient>,

    client_settings: ClientSettings,

    ui_message_sender: Sender<UIMessage>,
    ui_action_receiver: Receiver<UIAction>,
    event_receiver: Receiver<Event>,
    data_client: DataClient,
}

impl MqttController {
    pub fn new(client_settings: ClientSettings, ui_message_sender: Sender<UIMessage>, ui_action_receiver: Receiver<UIAction>) -> Self {
        let mut mqttoptions = MqttOptions::new(
            nanoid!(10),
            &client_settings.host,
            client_settings.port,
        );
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let (mut client, mut connection) = Client::new(mqttoptions, 10);
        let data_client = DataClient::new(client_settings.clone());

        let (event_sender, event_receiver) = mpsc::channel::<Event>();
        let _ui_message_sender = ui_message_sender.clone();
        std::thread::spawn(move || {
            for (i, notification) in connection.iter().enumerate() {
                match notification {
                    Ok(event) => event_sender.send(event).unwrap(),
                    Err(err) => {
                        _ui_message_sender.send(UIMessage::SystemError(
                            format!("MQTT Error: {}", err)
                        )).unwrap();

                        break;
                    }
                };
            }
        });

        Self {
            client,
            client_settings,
            ui_message_sender,
            ui_action_receiver,
            event_receiver,
            data_client,
            chat_clients: Vec::new(),
        }
    }

    pub fn start_mqtt() {}

    pub fn start(&mut self) {
        self.client.subscribe(&self.client_settings.topic, QoS::AtMostOnce).unwrap();

        let packet = packets::ReqAnnouncement {
            version: "1.0.0".to_string(),
        };

        self.publish_packet(PacketType::ReqAnnouncement, &packet).expect("Couldn't request announcements.");

        loop {
            match self.ui_action_receiver.try_recv() {
                Ok(action) => self.dispatch_action(action),
                _ => {}
            }

            match self.event_receiver.try_recv() {
                Ok(event) => self.handle_packet(event),
                _ => {}
            }
        }
    }

    pub fn dispatch_action(&mut self, action: UIAction) {
        match action {
            UIAction::SendMessage(message) => {
                let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
                    Ok(time) => time.as_secs(),
                    Err(_e) => 0u64
                };

                let signature = self.data_client.signature(&message, &timestamp);
                let chat_message = packets::ChatMessage {
                    message,
                    timestamp,
                    signature,
                };

                self.publish_packet(PacketType::ChatMessage, &chat_message).expect("Couldn't send message.");
            }
            UIAction::ChangeAlias(alias) => {
                let res = self.data_client.change_alias(alias);
                if res.is_err() {
                    self.ui_message_sender.send(UIMessage::SystemError(format!("{:?}", res.err().unwrap()))).unwrap()
                }
                self.send_announcement();

                self.ui_message_sender.send(
                    UIMessage::System("Restart client to apply changes".to_string())
                ).unwrap()
            }

            _ => self.ui_message_sender.send(UIMessage::SystemError("unimplemented".to_string())).unwrap()
        }
    }

    fn publish_packet<T>(&mut self, packet_type: PacketType, packet: &T) -> Result<(), ClientError>
        where
            T: Serialize + ?Sized + Debug
    {
        let mut packet_serialized = rmp_serde::to_vec(packet).unwrap();
        let mut publish_data = vec![packet_type as u8];

        publish_data.append(&mut packet_serialized);

        self.client.publish(
            &self.client_settings.topic,
            QoS::AtLeastOnce,
            false,
            publish_data,
        )
    }

    fn handle_packet(&mut self, event: Event) {
        match event {
            Event::Incoming(incoming) => {
                match incoming {
                    Incoming::Publish(publish) => self.handle_publish_packet(publish),
                    Incoming::ConnAck(_conn) => {
                        self.ui_message_sender.send(UIMessage::System(
                            format!("Connected to room \"{}\"", self.client_settings.topic)
                        )).unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn handle_publish_packet(&mut self, publish: Publish) {
        if publish.payload.len() < 2 { return; }
        let packet_type = publish.payload[0];
        let packet_data = publish.payload.slice(1..publish.payload.len());


        match packet_type {
            001u8 => {
                let packet_res = rmp_serde::from_slice(&packet_data) as Result<packets::ReqAnnouncement, _>;

                match packet_res {
                    Ok(packet) => self.deal_req_announcement(packet),
                    Err(err) => println!("Bad packet: {}", err)
                }
            }
            002u8 => {
                let packet_res = rmp_serde::from_slice(&packet_data) as Result<packets::Announcement, _>;

                match packet_res {
                    Ok(packet) => self.deal_announcement(packet),
                    Err(err) => println!("Bad packet: {}", err)
                }
            }
            003u8 => {
                let packet_res = rmp_serde::from_slice(&packet_data) as Result<packets::ChatMessage, _>;

                match packet_res {
                    Ok(packet) => self.deal_chat_message(packet),
                    Err(err) => println!("Bad packet: {}", err)
                }
            }
            _ => {}
        }
    }

    fn send_announcement(&mut self) {
        let packet = packets::Announcement {
            alias: self.data_client.database_file.alias.clone(),
            pub_key: self.data_client.pubkey().to_string(),
        };

        self.publish_packet(PacketType::Announcement, &packet).unwrap()
    }

    // Packet interaction

    fn deal_req_announcement(&mut self, _packet: packets::ReqAnnouncement) {
        self.send_announcement();
    }

    fn deal_announcement(&mut self, packet: packets::Announcement) {
        let pubkey = PKey::public_key_from_pem(packet.pub_key.as_bytes());
        if pubkey.is_err() { return; }

        self.chat_clients.push(ChatClient {
            alias: packet.alias,
            pubkey: pubkey.unwrap(),
        });
    }

    fn deal_chat_message(&mut self, packet: packets::ChatMessage) {
        let mut from_client_res: Option<ChatClient> = None;
        for (i, chat_client) in self.chat_clients.iter().enumerate() {
            let ok = DataClient::try_verify(&packet.message, &packet.timestamp, &packet.signature, &chat_client.pubkey);

            if ok {
                from_client_res = Some(self.chat_clients[i].clone());
                break;
            }
        }
        if from_client_res.is_none() { return; }


        let from_client = from_client_res.unwrap();

        self.ui_message_sender.send(
            UIMessage::Chat(from_client.clone(), packet.message)
        ).unwrap()
    }
}